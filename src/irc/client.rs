use super::Irc;

use super::super::Terminal;

use super::tokio::tcp::TcpStream;
use super::tokio::{client};
use super::tokio::reactor::*;
use super::tokio::io::{Transport};
use super::tokio::proto::pipeline::Frame;
use std::io;
use std::io::{stdin, stdout, Stdin, Stdout};
use std::net::SocketAddr;

pub struct Client {
    reactor: Option<ReactorHandle>,
}

impl Client {
    pub fn new() -> Client {
        let reactor = None;

        Client {
            reactor: reactor,
        } 
    }

    pub fn connect(self, addr: SocketAddr) {
        let reactor = match self.reactor {
            Some(r) => r,
            None => {
                let reactor = Reactor::default().unwrap();
                let handle = reactor.handle();
                reactor.spawn();
                handle
            }
        };

        client::connect(&reactor, addr, |stream| Ok(ClientTask::new(stream, stdin(), stdout())));
        //self.reactor.run().unwrap();
    }
}

pub struct ClientTask {
    input: Terminal<Stdin, Stdout>,
    server: Irc<TcpStream>,
}

impl ClientTask  {
    fn new(stream: TcpStream, stdin: Stdin, stdout: Stdout) -> ClientTask {
        ClientTask {
            input: Terminal::new(stdin, stdout),
            server: Irc::new(stream),
        }
    }
}

impl Task for ClientTask {
    fn tick(&mut self) -> io::Result<Tick> {
        //debug!("Start Read");
        if let Ok(Some(frame)) = self.server.read() {
           // debug!("Read");
            self.input.write(frame).unwrap();
        }

        //debug!("Start Write");
        if let Ok(Some(Frame::Message(frame))) = self.input.read() {
            //debug!("Write");
            self.server.write(Frame::Message(frame.to_command().unwrap())).unwrap();
        }

        Ok(Tick::WouldBlock)
    }
}
