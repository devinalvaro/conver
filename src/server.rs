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
use crate::store::{MemoryStore, Store};

pub struct Server<'a> {
    host: &'a str,
    port: &'a str,

    inner: Arc<ServerInner>,
}

struct ServerInner {
    store: Mutex<Box<dyn Store + Send>>,
}

impl<'a> Server<'a> {
    pub fn new(host: &'a str, port: &'a str) -> Self {
        Server {
            host,
            port,

            inner: Arc::new(ServerInner {
                store: Mutex::new(Server::store()),
            }),
        }
    }

    fn store() -> Box<dyn Store + Send> {
        Box::new(MemoryStore::new())
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
                self.queue_user_chat(&user.clone(), chat);
            }
            People::Group(group) => {
                self.queue_group_chat(&group.clone(), chat);
            }
        };
    }

    fn join_group(&self, join: Join) {
        let mut store = self.store.lock().unwrap();
        store.add_group_member(join.get_sender().clone(), join.get_group());
    }

    fn leave_group(&self, leave: Leave) {
        let mut store = self.store.lock().unwrap();
        store.remove_group_member(leave.get_sender(), leave.get_group());
    }

    fn queue_user_chat(&self, user: &User, chat: Chat) {
        let mut store = self.store.lock().unwrap();
        store.queue_user_chat(user, chat);
    }

    fn queue_group_chat(&self, group: &Group, chat: Chat) {
        let mut store = self.store.lock().unwrap();
        store.queue_group_chat(group, chat);
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
            self.send_chat(&mut stream, &user);
        }
    }

    fn send_chat(&self, stream: &mut TcpStream, user: &User) {
        let mut store = self.store.lock().unwrap();
        if let Some(chat) = store.first_user_chat(user) {
            if self.write_chat(stream, &chat) {
                store.dequeue_user_chat(user);
            }
        }
    }

    fn write_chat(&self, stream: &mut TcpStream, chat: &Chat) -> bool {
        let chat = bincode::serialize(chat).unwrap();
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
