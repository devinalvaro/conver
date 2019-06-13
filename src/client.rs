use std::error::Error;
use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::message::Message;
use crate::point::{Point, User};

type Buffer = [u8; 4096];

pub struct Client<'a> {
    server_address: &'a str,
    server_port: &'a str,

    inner: Arc<ClientInner>,
}

struct ClientInner {
    user: Mutex<User>,
}

impl<'a> Client<'a> {
    pub fn new(user_id: i64, server_address: &'a str, server_port: &'a str) -> Self {
        Client {
            server_address,
            server_port,

            inner: Arc::new(ClientInner {
                user: Mutex::new(User::new(user_id)),
            }),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let server_url = [self.server_address, self.server_port].join(":");
        let mut stream = TcpStream::connect(server_url)?;

        self.write_user(&mut stream)?;

        let write_stream = stream.try_clone()?;
        let write_inner = self.inner.clone();
        let write_handler =
            thread::spawn(move || write_inner.handle_connection_write(write_stream));

        let read_stream = stream;
        let read_inner = self.inner.clone();
        let read_handler = thread::spawn(move || read_inner.handle_connection_read(read_stream));

        write_handler.join().unwrap();
        read_handler.join().unwrap();

        Ok(())
    }

    fn write_user(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let user = self.inner.user.lock().unwrap();
        let user = bincode::serialize(&*user)?;
        stream.write(&user[..])?;

        Ok(())
    }
}

impl ClientInner {
    fn handle_connection_write(&self, mut stream: TcpStream) {
        loop {
            let user = self.user.lock().unwrap();
            let receiver = self.read_receiver();
            let body = self.read_body();

            let message = Message::new(user.clone(), receiver, body);
            let message = bincode::serialize(&message).unwrap();
            stream.write(&message[..]).unwrap();
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
                    println!("{:?}", message);
                }
                Err(err) => panic!("{}", err),
            };
        }
    }

    fn read_receiver(&self) -> Point {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let receiver_id = input.trim().parse::<i64>().unwrap();

        Point::User(User::new(receiver_id))
    }

    fn read_body(&self) -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let body = input;

        body
    }
}
