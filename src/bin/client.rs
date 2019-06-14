use std::env;

use point::client::Client;

fn main() {
    let mut args = env::args();
    args.next();

    let username = args.next().unwrap().trim().into();

    let client = Client::new("127.0.0.1", "7878", username);
    client.start().unwrap();
}
