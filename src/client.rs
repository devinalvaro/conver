use std::error::Error;
use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::str;

use bincode;

use crate::buffer::{self, Buffer, BUFFER_SIZE};
use crate::message::{Chat, Message};
use crate::people::{People, User};

pub struct Client {
    user: User,
    stream: TcpStream,
}

impl Client {
    pub fn new(host: &str, port: &str, username: &str) -> Result<Self, Box<dyn Error>> {
        let address = [host, port].join(":");
        let mut stream = TcpStream::connect(address)?;

        let user = User::new(username.into());
        Client::write_user(&mut stream, &user)?;

        Ok(Client { user, stream })
    }

    fn write_user(stream: &mut TcpStream, user: &User) -> Result<(), Box<dyn Error>> {
        let user = bincode::serialize(&user)?;
        let buf = buffer::from_vec(user);
        if stream.write(&buf)? != buf.len() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "connection was aborted while initiating connection",
            )));
        }

        Ok(())
    }

    pub fn try_clone(&self) -> Result<Self, Box<dyn Error>> {
        Ok(Client {
            user: self.user.clone(),
            stream: self.stream.try_clone()?,
        })
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }

    pub fn read_chat(&mut self) -> Result<Chat, Box<dyn Error>> {
        let mut buf: Buffer = [0; BUFFER_SIZE];
        loop {
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                // disconnect
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "connection was aborted while reading chat",
                )));
            }
            if n != buf.len() {
                // retry
                continue;
            }
            break;
        }
        let chat: Chat = bincode::deserialize(&buf[..])?;
        match chat.get_receiver() {
            People::User(user) => assert_eq!(&self.user, user),
            People::Group(_) => {}
        };
        Ok(chat)
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        match message {
            Message::Chat(ref chat) => assert_eq!(&self.user, chat.get_sender()),
            Message::Join(ref join) => assert_eq!(&self.user, join.get_sender()),
            Message::Leave(ref leave) => assert_eq!(&self.user, leave.get_sender()),
        };
        let message = bincode::serialize(&message)?;
        let buf = buffer::from_vec(message);
        loop {
            let n = self.stream.write(&buf)?;
            if n == 0 {
                // disconnect
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "connection was aborted while sending message",
                )));
            }
            if n == buf.len() {
                // retry
                break;
            }
        }

        Ok(())
    }
}
