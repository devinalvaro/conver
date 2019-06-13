use serde::{Deserialize, Serialize};

// TODO: figure out a better name
#[derive(Debug, Serialize, Deserialize)]
pub enum Point {
    User(User),
    Group(Group),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: i64,
}

impl User {
    pub fn new(id: i64) -> Self {
        User { id }
    }
}
