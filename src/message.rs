use serde::{Deserialize, Serialize};

use crate::point::Point;
use crate::point::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    sender: User,
    receiver: Point,
    text: String,
}

impl Message {
    pub fn new(sender: User, receiver: Point, text: String) -> Self {
        Message {
            sender,
            receiver,
            text,
        }
    }
}
