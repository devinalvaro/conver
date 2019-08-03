use std::collections::{vec_deque::VecDeque, HashMap, HashSet};
use std::sync::Arc;

use crate::message::Chat;
use crate::people::{Group, User};
use crate::store::Store;

pub struct MemoryStore {
    group_member_lists: HashMap<Group, HashSet<User>>,
    pending_chat_queues: HashMap<User, VecDeque<Arc<Chat>>>,
}

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore {
            group_member_lists: HashMap::new(),
            pending_chat_queues: HashMap::new(),
        }
    }
}

impl Store for MemoryStore {
    fn first_user_chat(&self, user: &User) -> Option<&Chat> {
        if let Some(pending_chats) = self.pending_chat_queues.get(user) {
            if let Some(chat) = pending_chats.front() {
                return Some(chat);
            }
        }
        None
    }

    fn queue_user_chat(&mut self, user: &User, chat: Chat) {
        let pending_chats = self
            .pending_chat_queues
            .entry(user.clone())
            .or_insert_with(|| VecDeque::new());
        pending_chats.push_back(Arc::new(chat));
    }

    fn queue_group_chat(&mut self, group: &Group, chat: Chat) {
        if let Some(group_members) = self.group_member_lists.get(group) {
            for member in group_members.iter() {
                if member == chat.get_sender() {
                    continue;
                }
                let pending_chats = self
                    .pending_chat_queues
                    .entry(member.clone())
                    .or_insert_with(|| VecDeque::new());
                pending_chats.push_back(Arc::new(chat.clone()));
            }
        }
    }

    fn dequeue_user_chat(&mut self, user: &User) {
        let pending_chats = self
            .pending_chat_queues
            .entry(user.clone())
            .or_insert_with(|| VecDeque::new());
        pending_chats.pop_front();
    }

    fn add_group_member(&mut self, user: User, group: &Group) {
        let group_members = self
            .group_member_lists
            .entry(group.clone())
            .or_insert_with(|| HashSet::new());
        group_members.insert(user);
    }

    fn remove_group_member(&mut self, user: &User, group: &Group) {
        let group_members = self
            .group_member_lists
            .entry(group.clone())
            .or_insert_with(|| HashSet::new());
        group_members.remove(user);
    }
}
