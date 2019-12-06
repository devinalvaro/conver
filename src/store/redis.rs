use std::cell::RefCell;
use std::error::Error;

use bincode;
use redis::{Commands, Connection, ErrorKind, RedisError, RedisResult, RedisWrite, Value};

use crate::buffer;
use crate::message::Chat;
use crate::people::{Group, User};
use crate::store::Store;

pub struct RedisStore {
    conn: RefCell<Connection>,
}

impl RedisStore {
    pub fn new(url: &str) -> Result<RedisStore, Box<dyn Error>> {
        let client = redis::Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(RedisStore {
            conn: RefCell::new(conn),
        })
    }
}

impl Store for RedisStore {
    fn front_chat(&self, user: &User) -> Option<Chat> {
        let chats: RedisResult<Vec<Chat>> = self.conn.borrow_mut().lrange(user, 0, 0);
        if let Ok(chats) = chats {
            if chats.len() >= 1 {
                return Some(chats[0].clone());
            }
        }
        None
    }

    fn queue_chat(&mut self, user: &User, chat: Chat) {
        let _: RedisResult<()> = self.conn.borrow_mut().rpush(user, chat);
    }

    fn dequeue_chat(&mut self, user: &User) {
        let _: RedisResult<()> = self.conn.borrow_mut().lpop(user);
    }

    fn queue_group_chat(&mut self, group: &Group, chat: Chat) {
        let group_members: RedisResult<Vec<User>> =
            self.conn.borrow_mut().smembers(group.get_groupname());
        if let Ok(group_members) = group_members {
            for member in group_members.iter() {
                if member == chat.get_sender() {
                    continue;
                }
                let _: RedisResult<()> = self.conn.borrow_mut().rpush(member, chat.clone());
            }
        }
    }

    fn add_group_member(&mut self, user: User, group: &Group) {
        let _: RedisResult<()> = self.conn.borrow_mut().sadd(group, user);
    }

    fn remove_group_member(&mut self, user: &User, group: &Group) {
        let _: RedisResult<()> = self.conn.borrow_mut().srem(group, user);
    }
}

impl redis::ToRedisArgs for Chat {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        let chat = bincode::serialize(self).unwrap();
        let buf = buffer::from_vec(chat);
        out.write_arg(&buf);
    }
}

impl redis::FromRedisValue for Chat {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(v) = v {
            let chat = bincode::deserialize(v).unwrap();
            Ok(chat)
        } else {
            Err(RedisError::from((
                ErrorKind::TypeError,
                "chat not deserializable",
            )))
        }
    }
}

impl redis::ToRedisArgs for User {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        let user = bincode::serialize(self).unwrap();
        let buf = buffer::from_vec(user);
        out.write_arg(&buf);
    }
}

impl redis::FromRedisValue for User {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(v) = v {
            let user = bincode::deserialize(v).unwrap();
            Ok(user)
        } else {
            Err(RedisError::from((
                ErrorKind::TypeError,
                "user not deserializable",
            )))
        }
    }
}

impl redis::ToRedisArgs for &User {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        let user = bincode::serialize(self).unwrap();
        let buf = buffer::from_vec(user);
        out.write_arg(&buf);
    }
}

impl redis::ToRedisArgs for &Group {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        let group = bincode::serialize(self).unwrap();
        let buf = buffer::from_vec(group);
        out.write_arg(&buf);
    }
}
