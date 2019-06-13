use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::str;

use bincode;

use crate::message::Message;

use crate::point::{Point, User};

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

        // TODO: figure out a better input format
        loop {
            let mut receiver_id = String::new();
            io::stdin().read_line(&mut receiver_id).unwrap();
            let receiver_id = receiver_id.trim().parse::<i64>().unwrap();
            let receiver = User::new(receiver_id);

            let mut text = String::new();
            io::stdin().read_line(&mut text).unwrap();
            let message = Message::new(self.user.clone(), Point::User(receiver), text);

            let message = bincode::serialize(&message).unwrap();
            stream.write(&message[..]).unwrap();
        }
    }
}
