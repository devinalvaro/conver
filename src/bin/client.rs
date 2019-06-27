use std::error::Error;
use std::io::{self, prelude::*};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

use clap::{App, Arg};

use conver::client::Client;
use conver::people::People;

mod parser;

use parser::Parser;

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

    let client = Client::new(host, port, &username).unwrap();
    handle_stream(client).unwrap();
}

fn handle_stream(client: Client) -> Result<(), Box<dyn Error>> {
    let (pulse_sender, pulse_receiver): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel();

    let read_client = client.try_clone()?;
    let read_handler = thread::spawn(move || handle_read_stream(read_client, pulse_sender));

    let write_client = client;
    let write_handler = thread::spawn(move || handle_write_stream(write_client, pulse_receiver));

    read_handler.join().unwrap();
    write_handler.join().unwrap();

    Ok(())
}

fn handle_read_stream(mut client: Client, _pulse_sender: mpsc::Sender<()>) {
    loop {
        let chat = client.read_chat().unwrap();
        match chat.get_receiver() {
            People::User(_) => println!("# {}: {}", chat.get_sender(), chat.get_body()),
            People::Group(group) => {
                println!("#[{}] {}: {}", group, chat.get_sender(), chat.get_body())
            }
        }
    }
}

fn handle_write_stream(mut client: Client, pulse_receiver: mpsc::Receiver<()>) {
    let parser = Parser::new(client.get_user().clone());

    while is_pulsing(&pulse_receiver) {
        let mut header = String::new();
        io::stdin().read_line(&mut header).unwrap();

        let body = match header.split_whitespace().next() {
            Some(method) => {
                if method == "CHAT" {
                    print!("> ");
                    io::stdout().flush().unwrap();

                    let mut body = String::new();
                    io::stdin().read_line(&mut body).unwrap();

                    Some(body)
                } else {
                    None
                }
            }
            None => None,
        };

        match parser.parse_message(header, body) {
            Ok(message) => client.send_message(message).unwrap(),
            Err(err) => println!("{}", err),
        };
        println!();
    }
}

fn is_pulsing(pulse_receiver: &mpsc::Receiver<()>) -> bool {
    if let Err(pulse) = pulse_receiver.try_recv() {
        if let TryRecvError::Disconnected = pulse {
            return false;
        }
    }
    true
}
