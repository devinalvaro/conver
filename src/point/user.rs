use std::collections::vec_deque::VecDeque;

use serde::{Deserialize, Serialize};

use crate::message::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: i64,
    pending_messages: VecDeque<Message>,
}
