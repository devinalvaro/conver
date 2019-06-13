use std::collections::vec_deque::VecDeque;

use crate::message::Message;

pub struct User {
    id: i64,
    pending_messages: VecDeque<Message>,
}
