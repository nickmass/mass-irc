use super::{Irc, UserCommand, CommandType, CommandBuilder};

use super::super::Terminal;

use super::super::termion::{async_stdin, AsyncReader};
use super::tokio::tcp::TcpStream;
use super::tokio::{client};
use super::tokio::reactor::*;
use super::tokio::io::{Transport};
use super::tokio::proto::pipeline::Frame;
use std::io;
use std::io::{stdin, stdout, Stdout};
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

        client::connect(&reactor, addr, |stream| Ok(ClientTask::new(stream, async_stdin(), stdout())));
    }
}

pub struct ClientTask {
    input: Terminal<AsyncReader, Stdout>,
    server: Irc<TcpStream>,
    tick: u64,
}

impl ClientTask  {
    fn new(stream: TcpStream, stdin: AsyncReader, stdout: Stdout) -> ClientTask {
        ClientTask {
            input: Terminal::new(stdin, stdout),
            server: Irc::new(stream),
            tick: 0,
        }
    }
}

impl Task for ClientTask {
    fn tick(&mut self) -> io::Result<Tick> {
        if self.tick == 1 {
            self.server.write(Frame::Message(UserCommand::User(
                        "NickMass".into(),
                        "8".into(),
                        "*".into(),
                        "Nick Massey".into()).to_command().unwrap())).unwrap();
            self.server.write(Frame::Message(UserCommand::Nick(
                        "NickMass".into()).to_command().unwrap())).unwrap();
            self.server.flush();
        }
        if let Ok(Some(frame)) = self.server.read() {
            match frame {
                Frame::Message(bundle) => {
                    for msg in bundle {
                        match msg.command {
                            CommandType::Ping => {
                                let pong =
                                    Frame::Message(CommandBuilder::new()
                                        .command(CommandType::Pong)
                                        .add_param(msg.params.data[0].clone())
                                        .build().unwrap());
                                self.server.write(pong);
                                let pong =
                                    Frame::Message(CommandBuilder::new()
                                        .command(CommandType::Pong)
                                        .add_param(msg.params.data[0].clone())
                                        .build().unwrap());
                                self.input.write(pong);
                            },
                            _ => {}
                        }
                        self.input.write(Frame::Message(msg)).unwrap();
                    }
                },
                _ => {}
            }

        }
        if let Ok(Some(Frame::Message(frame))) = self.input.read() {
            self.server.write(Frame::Message(frame.to_command().unwrap())).unwrap();
        }

        self.tick += 1;
        Ok(Tick::WouldBlock)
    }
}
