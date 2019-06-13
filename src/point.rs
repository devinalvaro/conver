mod group;
mod user;

pub use group::Group;
pub use user::User;

pub enum Point {
    User(User),
    Group(Group),
}
