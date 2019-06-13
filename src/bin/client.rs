use std::env;

use point::client::Client;

fn main() {
    // TODO: parse flags
    let mut args = env::args();
    args.next();

    let user_id = if let Some(user_id) = args.next() {
        user_id.trim().parse::<i64>().unwrap()
    } else {
        1
    };

    let client = Client::new(user_id, "127.0.0.1", "7878");
    client.start();
}
