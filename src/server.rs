use std::collections::{vec_deque::VecDeque, HashMap, HashSet};
use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::buffer::{self, Buffer, BUFFER_SIZE};
use crate::message::{Chat, Join, Leave, Message};
use crate::people::{Group, People, User};

pub struct Server<'a> {
    host: &'a str,
    port: &'a str,

    inner: Arc<ServerInner>,
}

struct ServerInner {
    group_member_lists: Mutex<HashMap<Group, HashSet<User>>>,
    pending_chat_queues: Mutex<HashMap<User, VecDeque<Arc<Chat>>>>,
}

impl<'a> Server<'a> {
    pub fn new(host: &'a str, port: &'a str) -> Self {
        Server {
            host,
            port,

            inner: Arc::new(ServerInner {
                group_member_lists: Mutex::new(HashMap::new()),
                pending_chat_queues: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn start(self) -> Result<(), Box<dyn Error>> {
        let address = [self.host, self.port].join(":");
        let listener = TcpListener::bind(address)?;

        for stream in listener.incoming() {
            let mut stream = stream?;

            let mut buf: Buffer = [0; BUFFER_SIZE];
            if stream.read(&mut buf)? != buf.len() {
                continue;
            }
            let user = bincode::deserialize(&buf[..])?;

            self.handle_stream(stream, user)?;
        }

        Ok(())
    }

    fn handle_stream(&self, stream: TcpStream, user: User) -> Result<(), Box<dyn Error>> {
        let (pulse_sender, pulse_receiver): (mpsc::Sender<()>, mpsc::Receiver<()>) =
            mpsc::channel();

        let read_stream = stream.try_clone()?;
        let read_inner = Arc::clone(&self.inner);
        thread::spawn(move || read_inner.handle_read_stream(read_stream, pulse_sender));

        let write_stream = stream;
        let write_inner = Arc::clone(&self.inner);
        thread::spawn(move || write_inner.handle_write_stream(write_stream, pulse_receiver, user));

        Ok(())
    }
}

impl ServerInner {
    fn handle_read_stream(&self, mut stream: TcpStream, _pulse_sender: mpsc::Sender<()>) {
        let mut buf: Buffer = [0; BUFFER_SIZE];
        loop {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                // disconnect
                break;
            }
            if n != buf.len() {
                // retry
                continue;
            }
            let message = bincode::deserialize(&buf[..]).unwrap();
            match message {
                Message::Chat(chat) => self.queue_chat(chat),
                Message::Join(join) => self.join_group(join),
                Message::Leave(leave) => self.leave_group(leave),
            }
        }
    }

    fn queue_chat(&self, chat: Chat) {
        match chat.get_receiver() {
            People::User(user) => {
                self.queue_user_chat(&user.clone(), Arc::new(chat));
            }
            People::Group(group) => {
                self.queue_group_chat(&group.clone(), Arc::new(chat));
            }
        };
    }

    fn join_group(&self, join: Join) {
        let mut group_member_lists = self.group_member_lists.lock().unwrap();
        let group_members = group_member_lists
            .entry(join.get_group().clone())
            .or_insert_with(|| HashSet::new());

        group_members.insert(join.get_sender().clone());
    }

    fn leave_group(&self, leave: Leave) {
        let mut group_member_lists = self.group_member_lists.lock().unwrap();
        let group_members = group_member_lists
            .entry(leave.get_group().clone())
            .or_insert_with(|| HashSet::new());

        group_members.remove(leave.get_sender());
    }

    fn queue_user_chat(&self, user: &User, chat: Arc<Chat>) {
        let mut pending_chat_queues = self.pending_chat_queues.lock().unwrap();
        let pending_chats = pending_chat_queues
            .entry(user.clone())
            .or_insert_with(|| VecDeque::new());

        pending_chats.push_back(chat);
    }

    fn queue_group_chat(&self, group: &Group, chat: Arc<Chat>) {
        let mut group_member_lists = self.group_member_lists.lock().unwrap();
        let group_members = group_member_lists
            .entry(group.clone())
            .or_insert_with(|| HashSet::new());

        for member in group_members.iter() {
            if member == chat.get_sender() {
                continue;
            }
            self.queue_user_chat(member, Arc::clone(&chat));
        }
    }
}

impl ServerInner {
    fn handle_write_stream(
        &self,
        mut stream: TcpStream,
        pulse_receiver: mpsc::Receiver<()>,
        user: User,
    ) {
        while self.is_pulsing(&pulse_receiver) {
            self.send_pending_chat(&mut stream, &user);
        }
    }

    fn send_pending_chat(&self, stream: &mut TcpStream, user: &User) {
        let mut pending_chat_queues = self.pending_chat_queues.lock().unwrap();
        let pending_chats = pending_chat_queues.get_mut(user);

        if let Some(pending_chats) = pending_chats {
            if let Some(chat) = pending_chats.front() {
                if self.write_chat(stream, chat) {
                    pending_chats.pop_front();
                }
            }
        }
    }

    fn write_chat(&self, stream: &mut TcpStream, chat: &Chat) -> bool {
        let chat = bincode::serialize(&*chat).unwrap();
        let buf = buffer::from_vec(chat);
        stream.write(&buf).unwrap() == buf.len()
    }

    fn is_pulsing(&self, pulse_receiver: &mpsc::Receiver<()>) -> bool {
        if let Err(pulse) = pulse_receiver.try_recv() {
            if let TryRecvError::Disconnected = pulse {
                return false;
            }
        }
        true
    }
}
