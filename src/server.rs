use std::collections::{vec_deque::VecDeque, HashMap};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

use bincode;

use crate::message::Message;
use crate::point::{Group, User};

pub struct Server<'a> {
    address: &'a str,
    port: &'a str,

    inner: Arc<ServerInner>,
}

struct ServerInner {
    group_members: Mutex<HashMap<Group, Vec<User>>>,
    pending_messages: Mutex<HashMap<User, VecDeque<Message>>>,
}

type Buffer = [u8; 4096];

impl<'a> Server<'a> {
    pub fn new(address: &'a str, port: &'a str) -> Self {
        Server {
            address,
            port,

            inner: Arc::new(ServerInner {
                group_members: Mutex::new(HashMap::new()),
                pending_messages: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn start(self) {
        let url = [self.address, self.port].join(":");
        let listener = TcpListener::bind(url).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let inner = self.inner.clone();
            thread::spawn(move || inner.handle_connection(stream));
        }
    }
}

impl ServerInner {
    fn handle_connection(&self, mut stream: TcpStream) {
        loop {
            let mut buffer: Buffer = [0; 4096];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }

                    self.handle_buffer(buffer);
                }
                Err(err) => panic!("{}", err),
            };
        }
    }

    fn handle_buffer(&self, buffer: Buffer) {
        let message: Message = bincode::deserialize(&buffer[..]).unwrap();
        println!("{:?}", message);
    }
}
