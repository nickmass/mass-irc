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

    pub fn poll_messages(&mut self) -> PollMessagesIter {
        PollMessagesIter {
            source: &self.receiver,
        }
    }

    pub fn send_message(&mut self, cmd: UserCommand) {
        let _ = self.sender.send(cmd);
    }
}

pub struct PollMessagesIter<'a> {
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



pub mod tokio {
    extern crate tokio_core;
    extern crate futures;

    use self::tokio_core::net::TcpStream;
    use self::tokio_core::io::{Codec, EasyBuf, Io};
    use self::tokio_core::reactor::Core;
    use self::futures::{Stream, Future, Sink};
    use self::futures::sync::mpsc::{
        unbounded as fut_unbounded,
        UnboundedSender as FutSender,
    };
    use std::io;
    use std::net::ToSocketAddrs;
    use std::sync::mpsc::{channel, Receiver};

    use irc::{ClientEvent, Command, CommandBuilder, CommandType, CommandParser, UserCommand};

    struct IrcCodec {
        parser: CommandParser,
    }
    impl IrcCodec {
        fn new() -> IrcCodec {
            IrcCodec {
                parser: CommandParser::new(),
            }
        }
    }
    impl Codec for IrcCodec {
        type In = Command;
        type Out = Command;

        fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
            if let Some(index) = buf.as_slice().iter().position(|x| *x == b'\n') {
                let msg = buf.drain_to(index + 1).as_slice().into();
                let msg = self.parser.parse(&msg);
                Ok(Some(msg))
            } else {
                Ok(None)
            }
        }

        fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
            buf.append(&mut msg.to_string().into_bytes());
            Ok(())
        }
    }

    pub struct Client {
        sender: FutSender<Command>,
        receiver: Receiver<Command>,
    }
    impl Client {
        pub fn connect<S: ToSocketAddrs>(addr: S) -> Client {
            let addr = addr.to_socket_addrs().unwrap().next().unwrap();
            let (in_tx, in_rx) = channel();
            let (out_tx, out_rx) = fut_unbounded();
            let core_out_tx = out_tx.clone();
            let echo_in_tx = in_tx.clone();
            ::std::thread::spawn(move || {
                let core_tx = core_out_tx;
                let echo_tx = echo_in_tx;
                let mut core = Core::new().unwrap();
                let handle = core.handle();
                let c = TcpStream::connect(&addr, &handle)
                    .and_then(|stream| {
                        let codec = IrcCodec::new();
                        let (w, r) = stream.framed(codec).split();
                        let incoming = r.for_each(|cmd| {
                            match cmd.command {
                                CommandType::Ping => {
                                    let pong = CommandBuilder::new()
                                        .command(CommandType::Pong)
                                        .add_param(cmd.params.data[0].clone())
                                        .build().unwrap();
                                    let _ = FutSender::send(&core_tx, pong);
                                },
                                _ => {}
                            }
                            let _ = in_tx.send(cmd);
                            Ok(())
                        });

                        let out = out_rx.map(move |cmd| {
                            match cmd.command {
                                CommandType::PrivMsg => {
                                    let _ = echo_tx.send(cmd.clone());
                                },
                                _ => {},
                            }
                            cmd
                        }).map_err(|_| io::Error::new(io::ErrorKind::Other, "Recv Error"));

                        let outgoing =
                            w.send_all(out)
                            .map(|_| ())
                            .map_err(|e|{
                                error!("Outgoing Command: {:?}", e);
                                ()
                            });
                        handle.spawn(outgoing);

                        incoming
                    });

                error!("Connecting...");
                let r = core.run(c);
                match r {
                    Err(e) => error!("Connection Closed: {:?}", e),
                    _ => error!("Connection Closed"),
                }
            });

            Client {
                sender: out_tx,
                receiver: in_rx,
            }
        }

        pub fn poll_messages(&self) -> PollMessagesIter {
            PollMessagesIter {
                source: &self.receiver,
            }
        }

        pub fn send_message(&self, cmd: UserCommand) {
            let _ = FutSender::send(&self.sender, cmd.to_command().unwrap());
        }
    }

    pub struct PollMessagesIter<'a> {
        source: &'a Receiver<Command>,
    }

    impl<'a> Iterator for PollMessagesIter<'a> {
        type Item = ClientEvent;
        fn next(&mut self) -> Option<Self::Item> {
            match self.source.try_recv() {
                Ok(e) => {
                    match ClientEvent::from_command(&e) {
                        Some(ce) => Some(ce),
                        None => Some(ClientEvent::Command(e)),
                    }
                },
                _ => None,
            }
        }
    }
}
