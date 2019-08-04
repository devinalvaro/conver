use clap::{App, Arg};

use conver::server::Server;
use conver::store::StoreKind;

fn main() {
    let matches = App::new("Point Client")
        .version("0.1.0")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("Server host"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Server port"),
        )
        .arg(
            Arg::with_name("store")
                .short("s")
                .long("store")
                .value_name("STORE")
                .help("Store kind"),
        )
        .get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("7878");

    let store_kind = matches.value_of("store").unwrap_or("memory");
    let store_kind = match store_kind {
        "redis" => {
            let url = matches
                .value_of("redis_url")
                .unwrap_or("redis://127.0.0.1/");
            StoreKind::RedisStore(url)
        }
        _ => StoreKind::MemoryStore,
    };

    let server = Server::new(host, port, store_kind);
    server.start().unwrap();
}
