use std::collections::{vec_deque::VecDeque, HashMap};
use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::message::{Chat, Message};
use crate::people::{Group, People, User};

pub struct Server<'a> {
    host: &'a str,
    port: &'a str,

    inner: Arc<ServerInner>,
}

struct ServerInner {
    group_member_lists: Mutex<HashMap<Group, Vec<User>>>,
    pending_chat_queues: Mutex<HashMap<User, VecDeque<Chat>>>,
}

type Buffer = [u8; 4096];

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
            let client_username = self.read_client_username(&mut stream)?;
            self.handle_stream(stream, client_username)?;
        }

        Ok(())
    }

    fn read_client_username(&self, stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
        let mut buffer: Buffer = [0; 4096];
        stream.read(&mut buffer)?;

        Ok(bincode::deserialize(&buffer[..])?)
    }

    fn handle_stream(
        &self,
        stream: TcpStream,
        client_username: String,
    ) -> Result<(), Box<dyn Error>> {
        let (pulse_sender, pulse_receiver): (mpsc::Sender<()>, mpsc::Receiver<()>) =
            mpsc::channel();

        let read_stream = stream.try_clone()?;
        let read_inner = self.inner.clone();
        thread::spawn(move || read_inner.handle_read_stream(read_stream, pulse_sender));

        let write_stream = stream;
        let write_inner = self.inner.clone();
        thread::spawn(move || {
            write_inner.handle_write_stream(write_stream, pulse_receiver, client_username)
        });

        Ok(())
    }
}

impl ServerInner {
    fn handle_read_stream(&self, mut stream: TcpStream, _pulse_sender: mpsc::Sender<()>) {
        loop {
            let mut buffer: Buffer = [0; 4096];
            if stream.read(&mut buffer).unwrap() == 0 {
                break;
            }

            let message: Message = bincode::deserialize(&buffer[..]).unwrap();
            match message {
                Message::Chat(chat) => {
                    match chat.get_receiver() {
                        People::User(user) => {
                            self.queue_user_chat(user.clone(), chat);
                        }
                        People::Group(group) => {
                            self.queue_group_chat(group.clone(), chat);
                        }
                    };
                }
                Message::Join(join) => {
                    let mut group_member_lists = self.group_member_lists.lock().unwrap();
                    let group_members = group_member_lists
                        .entry(join.get_group().clone())
                        .or_insert(vec![]);

                    group_members.push(join.get_sender().clone());
                }
            }
        }
    }

    fn queue_user_chat(&self, user: User, chat: Chat) {
        let mut pending_chat_queues = self.pending_chat_queues.lock().unwrap();
        let pending_chats = pending_chat_queues.entry(user).or_insert(VecDeque::new());

        pending_chats.push_back(chat);
    }

    fn queue_group_chat(&self, group: Group, chat: Chat) {
        let mut group_member_lists = self.group_member_lists.lock().unwrap();
        let group_members = group_member_lists.entry(group).or_insert(vec![]);

        for member in group_members {
            if member == chat.get_sender() {
                continue;
            }

            self.queue_user_chat(member.clone(), chat.clone());
        }
    }

    fn handle_write_stream(
        &self,
        mut stream: TcpStream,
        pulse_receiver: mpsc::Receiver<()>,
        client_username: String,
    ) {
        let client = User::new(client_username);

        while self.is_pulsing(&pulse_receiver) {
            self.send_chat(&mut stream, &client);
        }
    }

    fn is_pulsing(&self, pulse_receiver: &mpsc::Receiver<()>) -> bool {
        if let Err(pulse) = pulse_receiver.try_recv() {
            if let TryRecvError::Disconnected = pulse {
                return false;
            }
        }

        true
    }

    fn send_chat(&self, stream: &mut TcpStream, client: &User) {
        let mut pending_chat_queues = self.pending_chat_queues.lock().unwrap();
        let pending_chats = pending_chat_queues.get_mut(client);

        if let Some(pending_chats) = pending_chats {
            if let Some(chat) = pending_chats.pop_front() {
                let chat = bincode::serialize(&chat).unwrap();
                stream.write(&chat[..]).unwrap();
            }
        }
    }
}
