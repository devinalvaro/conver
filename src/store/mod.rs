use crate::message::Chat;
use crate::people::{Group, User};

mod memory;
mod redis;

pub use self::memory::MemoryStore;
pub use self::redis::RedisStore;

pub trait Store {
    fn first_user_chat(&self, user: &User) -> Option<Chat>;
    fn queue_user_chat(&mut self, user: &User, chat: Chat);
    fn queue_group_chat(&mut self, group: &Group, chat: Chat);
    fn dequeue_user_chat(&mut self, user: &User);

    fn add_group_member(&mut self, user: User, group: &Group);
    fn remove_group_member(&mut self, user: &User, group: &Group);
}

pub enum StoreKind<'a> {
    MemoryStore,
    RedisStore(&'a str),
}
