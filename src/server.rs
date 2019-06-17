use std::collections::{vec_deque::VecDeque, HashMap};
use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::message::Message;
use crate::people::{Group, People, User};

pub struct Server<'a> {
    host: &'a str,
    port: &'a str,

    inner: Arc<ServerInner>,
}

struct ServerInner {
    group_member_lists: Mutex<HashMap<Group, Vec<User>>>,
    pending_message_queues: Mutex<HashMap<User, VecDeque<Message>>>,
}

type Buffer = [u8; 4096];

impl<'a> Server<'a> {
    pub fn new(host: &'a str, port: &'a str) -> Self {
        Server {
            host,
            port,

            inner: Arc::new(ServerInner {
                group_member_lists: Mutex::new(HashMap::new()),
                pending_message_queues: Mutex::new(HashMap::new()),
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
            match message.get_receiver() {
                People::User(user) => {
                    self.queue_user_message(user.clone(), message);
                }
                People::Group(group) => {
                    self.queue_group_message(group.clone(), message);
                }
            };
        }
    }

    fn queue_user_message(&self, user: User, message: Message) {
        let mut pending_message_queues = self.pending_message_queues.lock().unwrap();
        let pending_messages = pending_message_queues
            .entry(user)
            .or_insert(VecDeque::new());

        pending_messages.push_back(message);
    }

    fn queue_group_message(&self, group: Group, message: Message) {
        let mut group_member_lists = self.group_member_lists.lock().unwrap();
        let group_members = group_member_lists.entry(group).or_insert(vec![]);

        for member in group_members {
            self.queue_user_message(member.clone(), message.clone());
        }
    }

    fn handle_write_stream(
        &self,
        mut stream: TcpStream,
        pulse_receiver: mpsc::Receiver<()>,
        client_username: String,
    ) {
        loop {
            if !self.is_pulsing(&pulse_receiver) {
                break;
            }

            let client = User::new(client_username.clone());
            self.send_message(&mut stream, &client);
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

    fn send_message(&self, stream: &mut TcpStream, client: &User) {
        let mut pending_message_queues = self.pending_message_queues.lock().unwrap();
        let pending_messages = pending_message_queues.get_mut(client);

        if let Some(pending_messages) = pending_messages {
            if let Some(message) = pending_messages.pop_front() {
                let message = bincode::serialize(&message).unwrap();
                stream.write(&message[..]).unwrap();
            }
        }
    }
}
