use serde::{Deserialize, Serialize};

use crate::point::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    id: i64,
    users: Vec<User>,
}
