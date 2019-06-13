use std::collections::{vec_deque::VecDeque, HashMap};
use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::message::Message;
use crate::point::{Group, Point, User};

pub struct Server<'a> {
    address: &'a str,
    port: &'a str,

    inner: Arc<ServerInner>,
}

struct ServerInner {
    group_member_lists: Mutex<HashMap<Group, Vec<User>>>,
    pending_message_queues: Mutex<HashMap<User, VecDeque<Message>>>,
}

type Buffer = [u8; 4096];

impl<'a> Server<'a> {
    pub fn new(address: &'a str, port: &'a str) -> Self {
        Server {
            address,
            port,

            inner: Arc::new(ServerInner {
                group_member_lists: Mutex::new(HashMap::new()),
                pending_message_queues: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn start(self) -> Result<(), Box<dyn Error>> {
        let url = [self.address, self.port].join(":");
        let listener = TcpListener::bind(url)?;

        for stream in listener.incoming() {
            let mut stream = stream?;

            let sender = self.read_sender(&mut stream)?;

            let write_stream = stream.try_clone()?;
            let write_inner = self.inner.clone();
            thread::spawn(move || write_inner.handle_connection_write(write_stream, &sender));

            let read_stream = stream;
            let read_inner = self.inner.clone();
            thread::spawn(move || read_inner.handle_connection_read(read_stream));
        }

        Ok(())
    }

    fn read_sender(&self, stream: &mut TcpStream) -> Result<User, Box<dyn Error>> {
        let mut buffer: Buffer = [0; 4096];
        stream.read(&mut buffer)?;

        Ok(bincode::deserialize(&buffer[..])?)
    }
}

impl ServerInner {
    fn handle_connection_write(&self, mut stream: TcpStream, user: &User) {
        loop {
            let mut pending_message_queues = self.pending_message_queues.lock().unwrap();

            if let Some(pending_messages) = pending_message_queues.get_mut(user) {
                if let Some(message) = pending_messages.pop_front() {
                    let message = bincode::serialize(&message).unwrap();
                    stream.write(&message[..]).unwrap();
                }
            }
        }
    }

    fn handle_connection_read(&self, mut stream: TcpStream) {
        loop {
            let mut buffer: Buffer = [0; 4096];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }

                    let message: Message = bincode::deserialize(&buffer[..]).unwrap();
                    match message.get_receiver() {
                        Point::User(user) => {
                            self.send_message_to_user(user.clone(), message);
                        }
                        Point::Group(group) => {
                            self.send_message_to_group(group.clone(), message);
                        }
                    };
                }
                Err(err) => panic!("{}", err),
            };
        }
    }

    fn send_message_to_user(&self, user: User, message: Message) {
        let mut pending_message_queues = self.pending_message_queues.lock().unwrap();
        let pending_messages = pending_message_queues
            .entry(user)
            .or_insert(VecDeque::new());

        pending_messages.push_back(message);
    }

    fn send_message_to_group(&self, group: Group, message: Message) {
        let mut group_member_lists = self.group_member_lists.lock().unwrap();
        let group_members = group_member_lists.entry(group).or_insert(vec![]);

        for member in group_members {
            self.send_message_to_user(member.clone(), message.clone());
        }
    }
}
