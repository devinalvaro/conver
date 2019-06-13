use serde::{Deserialize, Serialize};

use crate::people::People;
use crate::people::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    sender: User,
    receiver: People,
    text: String,
}

impl Message {
    pub fn new(sender: User, receiver: People, text: String) -> Self {
        Message {
            sender,
            receiver,
            text,
        }
    }

    pub fn get_receiver(&self) -> &People {
        &self.receiver
    }
}
