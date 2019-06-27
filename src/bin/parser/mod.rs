use std::str::SplitWhitespace;

use conver::message::{Chat, Join, Leave, Message};
use conver::people::{Group, People, User};

mod error;

pub use error::ParseError;

pub struct Parser {
    sender: User,
}

impl Parser {
    pub fn new(sender: User) -> Self {
        Parser { sender }
    }

    pub fn parse_message(
        &self,
        header: String,
        body: Option<String>,
    ) -> Result<Message, ParseError> {
        let mut header = header.split_whitespace();
        let method = header.next().ok_or(ParseError::method_type_not_found())?;
        match method {
            "CHAT" => Ok(Message::Chat(self.parse_chat(header, body.unwrap())?)),
            "JOIN" => Ok(Message::Join(self.parse_join(header)?)),
            "LEAVE" => Ok(Message::Leave(self.parse_leave(header)?)),
            _ => Err(ParseError::unknown_method_type()),
        }
    }

    fn parse_chat(&self, mut header: SplitWhitespace, body: String) -> Result<Chat, ParseError> {
        let receiver_type = header.next().ok_or(ParseError::receiver_type_not_found())?;
        let receiver = match receiver_type.trim() {
            "USER" => {
                let username = header.next().ok_or(ParseError::username_not_found())?;
                let username = username.trim().into();

                People::User(User::new(username))
            }
            "GROUP" => {
                let groupname = header.next().ok_or(ParseError::groupname_not_found())?;
                let groupname = groupname.trim().into();

                People::Group(Group::new(groupname))
            }
            _ => return Err(ParseError::unknown_receiver_type()),
        };
        Ok(Chat::new(self.sender.clone(), receiver, body))
    }

    fn parse_join(&self, mut header: SplitWhitespace) -> Result<Join, ParseError> {
        let groupname = header.next().ok_or(ParseError::groupname_not_found())?;
        let group = Group::new(groupname.into());
        Ok(Join::new(self.sender.clone(), group))
    }

    fn parse_leave(&self, mut header: SplitWhitespace) -> Result<Leave, ParseError> {
        let groupname = header.next().ok_or(ParseError::groupname_not_found())?;
        let group = Group::new(groupname.into());
        Ok(Leave::new(self.sender.clone(), group))
    }
}
