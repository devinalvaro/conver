use std::error::Error;
use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::message::Message;
use crate::people::{People, User};

type Buffer = [u8; 4096];

pub struct Client<'a> {
    server_address: &'a str,
    server_port: &'a str,

    inner: Arc<ClientInner>,
}

struct ClientInner {
    username: Mutex<String>,
}

impl<'a> Client<'a> {
    pub fn new(username: String, server_address: &'a str, server_port: &'a str) -> Self {
        Client {
            server_address,
            server_port,

            inner: Arc::new(ClientInner {
                username: Mutex::new(username),
            }),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let server_url = [self.server_address, self.server_port].join(":");
        let mut stream = TcpStream::connect(server_url)?;

        self.write_username(&mut stream)?;
        self.handle_stream(stream)?;

        Ok(())
    }

    fn write_username(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let username = self.inner.username.lock().unwrap();
        let username = bincode::serialize(&*username)?;
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

            let message: Message = bincode::deserialize(&buffer[..]).unwrap();
            println!("{:?}", message);
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
        let receiver = self.read_receiver();
        let body = self.read_body();

        Message::new(user, receiver, body)
    }

    fn get_user(&self) -> User {
        let username = self.username.lock().unwrap();

        User::new(username.clone())
    }

    fn read_receiver(&self) -> People {
        let mut receiver_username = String::new();
        io::stdin().read_line(&mut receiver_username).unwrap();

        People::User(User::new(receiver_username))
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
