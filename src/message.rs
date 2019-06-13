use serde::{Deserialize, Serialize};

use crate::point::Point;
use crate::point::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    sender: User,
    receiver: Point,
    text: String,
}
