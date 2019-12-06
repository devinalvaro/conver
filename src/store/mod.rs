use crate::message::Chat;
use crate::people::{Group, User};

pub mod memory;
pub mod redis;

pub use self::memory::MemoryStore;
pub use self::redis::RedisStore;

pub trait Store {
    fn front_chat(&self, user: &User) -> Option<Chat>;
    fn queue_chat(&mut self, user: &User, chat: Chat);
    fn dequeue_chat(&mut self, user: &User);
    fn queue_group_chat(&mut self, group: &Group, chat: Chat);

    fn add_group_member(&mut self, user: User, group: &Group);
    fn remove_group_member(&mut self, user: &User, group: &Group);
}
