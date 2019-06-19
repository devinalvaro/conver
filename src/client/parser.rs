use std::io::{self, prelude::*};
use std::str::SplitWhitespace;

use crate::message::{Chat, Join, Leave, Message};
use crate::people::{Group, People, User};

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn read_message(&self) -> Message {
        let mut message = String::new();
        io::stdin().read_line(&mut message).unwrap();
        let mut message = message.split_whitespace();

        let message_type = message.next().expect("invalid message format");
        match message_type {
            "CHAT" => Message::Chat(self.read_chat(message)),
            "JOIN" => Message::Join(self.read_join(message)),
            "LEAVE" => Message::Leave(self.read_leave(message)),
            _ => panic!("unknown message type"),
        }
    }

    fn read_chat(&self, mut message: SplitWhitespace) -> Chat {
        let receiver_type = message.next().expect("invalid message format");
        let receiver = match receiver_type.trim() {
            "USER" => {
                let username = message.next().expect("invalid message format");
                let username = username.trim().into();

                People::User(User::new(username))
            }
            "GROUP" => {
                let groupname = message.next().expect("invalid message format");
                let groupname = groupname.trim().into();

                People::Group(Group::new(groupname))
            }
            _ => panic!("unknown receiver type"),
        };

        print!("> ");
        io::stdout().flush().unwrap();

        let mut body = String::new();
        io::stdin().read_line(&mut body).unwrap();

        Chat::new(self.get_user(), receiver, body)
    }

    fn read_join(&self, mut message: SplitWhitespace) -> Join {
        let groupname = message.next().expect("invalid message format");
        let group = Group::new(groupname.into());

        Join::new(self.get_user(), group)
    }

    fn read_leave(&self, mut message: SplitWhitespace) -> Leave {
        let groupname = message.next().expect("invalid message format");
        let group = Group::new(groupname.into());

        Leave::new(self.get_user(), group)
    }
}
