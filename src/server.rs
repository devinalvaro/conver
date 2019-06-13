use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

use bincode;

use crate::message::Message;

pub struct Server<'a> {
    address: &'a str,
    port: &'a str,
}

type Buffer = [u8; 4096];

impl<'a> Server<'a> {
    pub fn new(address: &'a str, port: &'a str) -> Self {
        Server { address, port }
    }

    pub fn start(&self) {
        let url = [self.address, self.port].join(":");
        let listener = TcpListener::bind(url).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            thread::spawn(move || Server::handle_connection(stream));
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        loop {
            let mut buffer: Buffer = [0; 4096];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }

                    Server::handle_buffer(buffer);
                }
                Err(err) => panic!("{}", err),
            };
        }
    }

    fn handle_buffer(buffer: Buffer) {
        let message: Message = bincode::deserialize(&buffer[..]).unwrap();
        println!("{:?}", message);
    }
}
