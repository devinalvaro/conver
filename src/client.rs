use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::str;
use std::thread;

use bincode;

use crate::message::Message;
use crate::point::{Point, User};

type Buffer = [u8; 4096];

pub struct Client<'a> {
    user: User,

    server_address: &'a str,
    server_port: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(user_id: i64, server_address: &'a str, server_port: &'a str) -> Self {
        Client {
            user: User::new(user_id),

            server_address,
            server_port,
        }
    }

    pub fn start(&self) {
        let server_url = [self.server_address, self.server_port].join(":");
        let mut stream = TcpStream::connect(server_url).unwrap();

        let user = bincode::serialize(&self.user).unwrap();
        stream.write(&user[..]).unwrap();

        let user = self.user.clone();

        // TODO: figure out a better input format
        let write_stream = stream.try_clone().unwrap();
        let write_handler =
            thread::spawn(move || Client::handle_connection_write(write_stream, user));

        let read_stream = stream;
        let read_handler = thread::spawn(move || Client::handle_connection_read(read_stream));

        write_handler.join().unwrap();
        read_handler.join().unwrap();
    }

    pub fn handle_connection_write(mut stream: TcpStream, user: User) {
        loop {
            let mut receiver_id = String::new();
            io::stdin().read_line(&mut receiver_id).unwrap();
            let receiver_id = receiver_id.trim().parse::<i64>().unwrap();
            let receiver = User::new(receiver_id);

            let mut text = String::new();
            io::stdin().read_line(&mut text).unwrap();
            let message = Message::new(user.clone(), Point::User(receiver), text);

            let message = bincode::serialize(&message).unwrap();
            stream.write(&message[..]).unwrap();
        }
    }

    pub fn handle_connection_read(mut stream: TcpStream) {
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
}
