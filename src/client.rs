use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::str;

use bincode;

use crate::message::{Chat, Message};
use crate::people::User;
use crate::server::Buffer;

pub struct Client {
    user: User,
    stream: TcpStream,
}

impl Client {
    pub fn new(host: &str, port: &str, username: &str) -> Result<Self, Box<dyn Error>> {
        let address = [host, port].join(":");
        let mut stream = TcpStream::connect(address)?;

        let user = User::new(username.into());
        {
            let user = bincode::serialize(&user)?;
            stream.write(&user[..])?;
        }

        Ok(Client { user, stream })
    }

    pub fn try_clone(&self) -> Result<Self, Box<dyn Error>> {
        Ok(Client {
            user: self.user.clone(),
            stream: self.stream.try_clone()?,
        })
    }

    pub fn read_chat(&mut self) -> Option<Chat> {
        let mut buffer: Buffer = [0; 4096];

        if self.stream.read(&mut buffer).unwrap() == 0 {
            None
        } else {
            Some(bincode::deserialize(&buffer[..]).unwrap())
        }
    }

    pub fn send_message(&mut self, message: Message) {
        match message {
            Message::Chat(ref chat) => assert_eq!(&self.user, chat.get_sender()),
            Message::Join(ref join) => assert_eq!(&self.user, join.get_sender()),
            Message::Leave(ref leave) => assert_eq!(&self.user, leave.get_sender()),
        };

        let message = bincode::serialize(&message).unwrap();
        self.stream.write(&message[..]).unwrap();
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }
}
