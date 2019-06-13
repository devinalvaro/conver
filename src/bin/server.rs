use point::server::Server;

fn main() {
    // TODO: parse flags
    let server = Server::new("127.0.0.1", "7878");
    server.start().unwrap();
}
