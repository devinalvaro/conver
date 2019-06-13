use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: i64,
}

impl User {
    pub fn new(id: i64) -> Self {
        User { id }
    }
}
