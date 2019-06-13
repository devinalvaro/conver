use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum People {
    User(User),
    Group(Group),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Group {
    id: i64,
}

impl Group {
    pub fn new(id: i64) -> Self {
        Group { id }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct User {
    id: i64,
}

impl User {
    pub fn new(id: i64) -> Self {
        User { id }
    }
}
