use irc::{Irc, CLIENT, CLIENT_MSG, UserCommand, CommandType, ClientEvent, CommandBuilder};

use mio::tcp::{TcpStream};
use mio::channel::{Sender as MioSender, Receiver as MioReceiver, channel as mio_channel};
use mio::{Poll, PollOpt, Ready, Events};

use std::net::ToSocketAddrs;
use std::thread;
use std::sync::mpsc::{channel, Receiver};

pub struct Client {
    sender: MioSender<UserCommand>,
    receiver: Receiver<ClientEvent>,
}

impl Client {
    pub fn connect<S: ToSocketAddrs>(addr: S) -> Client
    {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx): (MioSender<UserCommand>, MioReceiver<UserCommand>) = mio_channel();
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();

        thread::spawn(move || {
            let poll = Poll::new().unwrap();
            let socket = TcpStream::connect(&addr).unwrap();

            let mut irc = Irc::new(socket, &poll);
            poll.register(&out_rx, CLIENT_MSG, Ready::readable(), PollOpt::edge()).unwrap();

            let mut events = Events::with_capacity(1024);

            loop {
                poll.poll(&mut events, None).unwrap();

                for event in events.iter() {
                    match event.token() {
                        CLIENT => {
                            if event.kind().is_readable() {
                                loop {
                                    match irc.read() {
                                        Ok(Some(msg)) => {
                                            match msg.command {
                                                CommandType::Ping => {
                                                    let pong = CommandBuilder::new()
                                                        .command(CommandType::Pong)
                                                        .add_param(msg.params.data[0].clone())
                                                        .build().unwrap();
                                                    irc.buf_write(pong);
                                                },
                                                _ => {},
                                            }
                                            match ClientEvent::from_command(&msg) {
                                                Some(ev) => in_tx.send(ev).unwrap(),
                                                _ => {} ,
                                            }
                                            let _ = in_tx.send(ClientEvent::Command(msg));
                                        },
                                        Ok(None) => break,
                                        Err(_) => break,
                                    }
                                }
                            }

                            if event.kind().is_writable() {
                                let _ = irc.write();
                            }
                        },
                        CLIENT_MSG => {
                            match out_rx.try_recv() {
                                Ok(msg) => {
                                    let command = msg.to_command().unwrap();
                                    match command.command {
                                        CommandType::PrivMsg  => {
                                            let _ = in_tx.send(
                                                ClientEvent::from_command(&command).unwrap());
                                        },
                                        _ => {},
                                    }
                                    irc.buf_write(command);
                                },
                                _ => return,
                            }
                            poll.reregister(&out_rx,
                                            CLIENT_MSG,
                                            Ready::readable(),
                                            PollOpt::edge()).unwrap();
                        },
                        _ => unreachable!(),
                    }
                }
            }
        });

        Client {
            sender: out_tx,
            receiver: in_rx,
        }
    }

    pub fn poll_messages<'a>(&'a mut self) -> PollMessagesIter {
        PollMessagesIter {
            source: &self.receiver,
        }
    }

    pub fn send_message(&mut self, cmd: UserCommand) {
        let _ = self.sender.send(cmd);
    }
}

struct PollMessagesIter<'a> {
    source: &'a Receiver<ClientEvent>,
}

impl<'a> Iterator for PollMessagesIter<'a> {
    type Item = ClientEvent;
    fn next(&mut self) -> Option<Self::Item> {
        match self.source.try_recv() {
            Ok(e) => Some(e),
            _ => None,
        }
    }
}
