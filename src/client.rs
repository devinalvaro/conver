use std::error::Error;
use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::Arc;
use std::thread;

use bincode;

use crate::message::{Chat, Join, Message};
use crate::people::{Group, People, User};

type Buffer = [u8; 4096];

pub struct Client<'a> {
    server_host: &'a str,
    server_port: &'a str,

    inner: Arc<ClientInner>,
}

struct ClientInner {
    username: String,
}

impl<'a> Client<'a> {
    pub fn new(server_host: &'a str, server_port: &'a str, username: String) -> Self {
        Client {
            server_host,
            server_port,

            inner: Arc::new(ClientInner { username }),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let server_address = [self.server_host, self.server_port].join(":");
        let mut stream = TcpStream::connect(server_address)?;

        self.write_username(&mut stream)?;
        self.handle_stream(stream)?;

        Ok(())
    }

    fn write_username(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let username = bincode::serialize(&self.inner.username)?;
        stream.write(&username[..])?;

        Ok(())
    }

    fn handle_stream(&self, stream: TcpStream) -> Result<(), Box<dyn Error>> {
        let (pulse_sender, pulse_receiver): (mpsc::Sender<()>, mpsc::Receiver<()>) =
            mpsc::channel();

        let read_stream = stream.try_clone()?;
        let read_inner = self.inner.clone();
        let read_handler =
            thread::spawn(move || read_inner.handle_read_stream(read_stream, pulse_sender));

        let write_stream = stream;
        let write_inner = self.inner.clone();
        let write_handler =
            thread::spawn(move || write_inner.handle_write_stream(write_stream, pulse_receiver));

        write_handler.join().unwrap();
        read_handler.join().unwrap();

        Ok(())
    }
}

impl ClientInner {
    fn handle_read_stream(&self, mut stream: TcpStream, _pulse_sender: mpsc::Sender<()>) {
        loop {
            let mut buffer: Buffer = [0; 4096];
            if stream.read(&mut buffer).unwrap() == 0 {
                break;
            }

            let chat: Chat = bincode::deserialize(&buffer[..]).unwrap();
            println!("{}: {}", chat.get_sender(), chat.get_body());
        }
    }

    fn handle_write_stream(&self, mut stream: TcpStream, pulse_receiver: mpsc::Receiver<()>) {
        loop {
            if !self.is_pulsing(&pulse_receiver) {
                break;
            }

            let message = self.read_message();
            self.send_message(&mut stream, message);
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

    fn read_message(&self) -> Message {
        let user = self.get_user();

        let mut message_type = String::new();
        io::stdin().read_line(&mut message_type).unwrap();
        match message_type.trim() {
            "CHAT" => {
                let receiver = self.read_receiver();
                let body = self.read_body();

                Message::Chat(Chat::new(user, receiver, body))
            }
            "JOIN" => {
                let group = self.read_group();

                Message::Join(Join::new(user, group))
            }
            _ => panic!("unknown message type"),
        }
    }

    fn get_user(&self) -> User {
        User::new(self.username.clone())
    }

    fn read_receiver(&self) -> People {
        let mut receiver_type = String::new();
        io::stdin().read_line(&mut receiver_type).unwrap();
        match receiver_type.trim() {
            "USER" => {
                let receiver = self.read_user();

                People::User(receiver)
            }
            "GROUP" => {
                let receiver = self.read_group();

                People::Group(receiver)
            }
            _ => panic!("unknown receiver type"),
        }
    }

    fn read_user(&self) -> User {
        let mut username = String::new();
        io::stdin().read_line(&mut username).unwrap();

        User::new(username.trim().into())
    }

    fn read_group(&self) -> Group {
        let mut groupname = String::new();
        io::stdin().read_line(&mut groupname).unwrap();

        Group::new(groupname.trim().into())
    }

    fn read_body(&self) -> String {
        let mut body = String::new();
        io::stdin().read_line(&mut body).unwrap();

        body
    }

    fn send_message(&self, stream: &mut TcpStream, message: Message) {
        let message = bincode::serialize(&message).unwrap();
        stream.write(&message[..]).unwrap();
    }
}
