extern crate tokio_core;
extern crate futures;

use self::tokio_core::net::TcpStream;
use self::tokio_core::io::{Codec, EasyBuf, Io};
use self::tokio_core::reactor::{Core, Interval};
use self::futures::{Stream, Future, Sink};
use self::futures::sync::mpsc::{
    unbounded as fut_unbounded,
    UnboundedSender as FutSender,
};
use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

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
    connected: Arc<AtomicBool>,
    thread: JoinHandle<io::Result<()>>,
}
impl Client {
    pub fn connect<S: ToSocketAddrs>(addr: S) -> Client {
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = fut_unbounded();
        let core_out_tx = out_tx.clone();
        let echo_in_tx = in_tx.clone();

        let connected = Arc::new(AtomicBool::new(true));
        let inner_connected = connected.clone();

        let thread = ::std::thread::spawn(move || {
            let connected = inner_connected;
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
                        .map(|_| ());

                    let watchdog = Interval::new(::std::time::Duration::from_millis(50), &handle)
                        .unwrap()
                        .take_while(|_| Ok(connected.load(Ordering::SeqCst)))
                        .collect()
                        .map(|_| ());

                    incoming
                        .select(outgoing).map(|_| ()).map_err(|(e, _next)| e)
                        .select(watchdog).map(|_| ()).map_err(|(e, _next)| e)
                }).map(|_|());

            let r = core.run(c);
            connected.store(false, Ordering::SeqCst);
            r
        });

        Client {
            sender: out_tx,
            receiver: in_rx,
            connected: connected,
            thread: thread,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    pub fn poll_messages(&self) -> PollMessagesIter {
        PollMessagesIter {
            source: &self.receiver,
        }
    }

    pub fn send_message(&self, cmd: UserCommand) {
        let _ = FutSender::send(&self.sender, cmd.to_command().unwrap());
    }

    pub fn close(self) -> io::Result<()> {
        self.connected.store(false, Ordering::SeqCst);
        self.thread.join().map_err(|e| panic!(e)).unwrap()
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
