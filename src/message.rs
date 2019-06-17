use serde::{Deserialize, Serialize};

use crate::people::{Group, People, User};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Chat(Chat),
    Join(Join),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chat {
    sender: User,
    receiver: People,
    body: String,
}

impl Chat {
    pub fn new(sender: User, receiver: People, body: String) -> Self {
        Chat {
            sender,
            receiver,
            body,
        }
    }

    pub fn get_sender(&self) -> &User {
        &self.sender
    }

    pub fn get_receiver(&self) -> &People {
        &self.receiver
    }

    pub fn get_body(&self) -> &str {
        &self.body
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Join {
    sender: User,
    group: Group,
}

impl Join {
    pub fn new(sender: User, group: Group) -> Self {
        Join { sender, group }
    }

    pub fn get_sender(&self) -> &User {
        &self.sender
    }

    pub fn get_group(&self) -> &Group {
        &self.group
    }
}
