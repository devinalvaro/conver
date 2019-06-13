use point::client::Client;

fn main() {
    let client = Client::new(1, "127.0.0.1", "7878");
    client.start();
}
