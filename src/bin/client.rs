use std::env;

use point::client::Client;

fn main() {
    let mut args = env::args();
    args.next();

    let username = args.next().unwrap();

    let client = Client::new(username, "127.0.0.1", "7878");
    client.start().unwrap();
}
