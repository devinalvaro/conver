use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum People {
    User(User),
    Group(Group),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct User {
    username: String,
}

impl User {
    pub fn new(username: String) -> Self {
        User { username }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Group {
    groupname: String,
}

impl Group {
    pub fn new(groupname: String) -> Self {
        Group { groupname }
    }
}
