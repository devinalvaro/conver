use std::io::prelude::*;
use std::net::TcpListener;
use std::str;
use std::thread;

pub struct Server<'a> {
    address: &'a str,
    port: &'a str,
}

impl<'a> Server<'a> {
    pub fn new(address: &'a str, port: &'a str) -> Self {
        Server { address, port }
    }

    pub fn start(&self) {
        let url = [self.address, self.port].join(":");
        let listener = TcpListener::bind(url).unwrap();

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            thread::spawn(move || loop {
                let mut buffer = [0; 4096];

                match stream.read(&mut buffer) {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }

                        match str::from_utf8(&buffer) {
                            Ok(message) => {
                                println!("{}", message);
                            }
                            Err(err) => panic!("{}", err),
                        }
                    }
                    Err(err) => panic!("{}", err),
                }
            });
        }
    }
}
