use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use conver::client::Client;
use conver::message::{Chat, Join};
use conver::people::{Group, People, User};

lazy_static! {
    // To ensure tests not run in parallel, as all clients connect to the same server process
    pub static ref TEST_LOCK: Mutex<()> = Mutex::new(());
}

const HOST: &str = "127.0.0.1";
const PORT: &str = "7878";

pub fn create_client(user: &User) -> Client {
    Client::new(HOST, PORT, user.get_username()).unwrap()
}

pub fn generate_user() -> User {
    let username = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
    User::new(username)
}

pub fn generate_group() -> Group {
    let groupname = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
    Group::new(groupname)
}

pub fn generate_chat(sender: &User, receiver: &User) -> Chat {
    let body = thread_rng().sample_iter(&Alphanumeric).take(512).collect();
    Chat::new(sender.clone(), People::User(receiver.clone()), body)
}

pub fn generate_group_chat(sender: &User, receiver: &Group) -> Chat {
    let body = thread_rng().sample_iter(&Alphanumeric).take(512).collect();
    Chat::new(sender.clone(), People::Group(receiver.clone()), body)
}

pub fn create_join(sender: &User, group: &Group) -> Join {
    Join::new(sender.clone(), group.clone())
}
