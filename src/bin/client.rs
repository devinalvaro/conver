use clap::{App, Arg};

use point::client::Client;

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
            Arg::with_name("username")
                .short("u")
                .long("user")
                .value_name("USERNAME")
                .help("Your username")
                .required(true),
        )
        .get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = matches.value_of("port").unwrap_or("7878");
    let username = matches.value_of("username").unwrap().to_string();

    let client = Client::new(host, port, username);
    client.start().unwrap();
}
