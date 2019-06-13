use point::client::Client;

fn main() {
    // TODO: parse flags
    let client = Client::new(1, "127.0.0.1", "7878");
    client.start();
}
