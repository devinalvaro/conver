use serde::{Deserialize, Serialize};

use crate::people::People;
use crate::people::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    sender: User,
    receiver: People,

    body: String,
}

impl Message {
    pub fn new(sender: User, receiver: People, body: String) -> Self {
        Message {
            sender,
            receiver,

            body,
        }
    }

    pub fn get_receiver(&self) -> &People {
        &self.receiver
    }
}
