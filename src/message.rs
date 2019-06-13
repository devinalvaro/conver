use crate::point::Point;
use crate::point::User;

pub struct Message {
    sender: User,
    receiver: Point,
    text: String,
}
