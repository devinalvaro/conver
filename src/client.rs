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
    user_id: Mutex<i64>,
}

impl<'a> Client<'a> {
    pub fn new(user_id: i64, server_address: &'a str, server_port: &'a str) -> Self {
        Client {
            server_address,
            server_port,

            inner: Arc::new(ClientInner {
                user_id: Mutex::new(user_id),
            }),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let server_url = [self.server_address, self.server_port].join(":");
        let mut stream = TcpStream::connect(server_url)?;

        self.write_user_id(&mut stream)?;
        self.handle_stream(stream)?;

        Ok(())
    }

    fn write_user_id(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let user_id = self.inner.user_id.lock().unwrap();
        let user_id = bincode::serialize(&*user_id)?;
        stream.write(&user_id[..])?;

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
            if let Err(pulse) = pulse_receiver.try_recv() {
                if let TryRecvError::Disconnected = pulse {
                    break;
                }
            }

            let user = self.get_user();
            let receiver = self.read_receiver();
            let body = self.read_body();

            let message = Message::new(user, receiver, body);
            let message = bincode::serialize(&message).unwrap();
            stream.write(&message[..]).unwrap();
        }
    }

    fn get_user(&self) -> User {
        let user_id = self.user_id.lock().unwrap();

        User::new(user_id.clone())
    }

    fn read_receiver(&self) -> People {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let receiver_id = input.trim().parse::<i64>().unwrap();

        People::User(User::new(receiver_id))
    }

    fn read_body(&self) -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let body = input;

        body
    }
}
