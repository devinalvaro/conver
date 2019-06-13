use serde::{Deserialize, Serialize};

mod group;
mod user;

pub use group::Group;
pub use user::User;

#[derive(Debug, Serialize, Deserialize)]
pub enum Point {
    User(User),
    Group(Group),
}
