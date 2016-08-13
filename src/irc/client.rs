use super::super::mio::timer::Builder;
use super::super::mio::channel;

use super::{Irc, UserCommand, CommandType, CommandBuilder};

use super::tokio::util::timer::Timer;
use super::tokio::util::channel::{Receiver as tokReceiver};
use super::tokio::tcp::TcpStream;
use super::tokio::reactor;
use super::tokio::reactor::*;
use super::tokio::io::Transport;
use super::tokio::proto::pipeline::Frame;

use std::io;
use std::net::SocketAddr;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

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

    pub fn connect(self, addr: SocketAddr) -> OuterTunnel {
        let reactor = match self.reactor {
            Some(r) => r,
            None => {
                let reactor = Reactor::default().unwrap();
                let handle = reactor.handle();
                reactor.spawn();
                handle
            }
        };

        connect(&reactor, addr,|stream, timer, tunnel| {
            Ok(ClientTask::new(stream, timer, tunnel))
        })
    }
}

pub struct ClientTask {
    server: Irc<TcpStream>,
    timer: Timer<()>,
    tunnel: InnerTunnel,
    tick: u64,
}

impl ClientTask  {
    fn new(stream: TcpStream, mut timer: Timer<()>, tunnel: InnerTunnel) -> ClientTask {
        ClientTask {
            server: Irc::new(stream),
            timer: timer,
            tunnel: tunnel,
            tick: 0,
        }
    }
}

impl Task for ClientTask {
    fn tick(&mut self) -> io::Result<Tick> {
        if self.tick == 1 {
            try!(self.server.write(Frame::Message(UserCommand::User(
                        "NickMass".into(),
                        "8".into(),
                        "*".into(),
                        "Nick Massey".into()).to_command().unwrap())));
            try!(self.server.write(Frame::Message(UserCommand::Nick(
                        "NickMass".into()).to_command().unwrap())));
            try!(self.server.flush());
        }
        if let Some(frame) = try!(self.server.read()) {
            match frame {
                Frame::Message(bundle) => {
                    for msg in bundle {
                        match msg.command {
                            CommandType::Ping => {
                                let pong =
                                    CommandBuilder::new()
                                    .command(CommandType::Pong)
                                    .add_param(msg.params.data[0].clone())
                                    .build().unwrap();
                                try!(self.server.write(Frame::Message(pong)));
                                let pong =
                                    CommandBuilder::new()
                                    .command(CommandType::Pong)
                                    .add_param(msg.params.data[0].clone())
                                    .build().unwrap();
                                self.tunnel.0.try_send(msg.to_string().into_bytes());
                                self.tunnel.0.try_send(pong.to_string().into_bytes());
                            },
                            _ => { self.tunnel.0.try_send(msg.to_string().into_bytes()); }

                        }
                    }
                },
                _ => {}
            }

        }

        match self.tunnel.1.recv() {
            Ok(Some(d)) => {
                let user_command = UserCommand::Nick(String::from_utf8(d).unwrap());
                let command = user_command.to_command().unwrap();
                try!(self.server.write(Frame::Message(command)));
            },
            _ => (),
        }

        self.tick += 1;
        Ok(Tick::WouldBlock)
    }
}

pub type OuterTunnel = (channel::SyncSender<Vec<u8>>, Receiver<Vec<u8>>);
pub type InnerTunnel = (SyncSender<Vec<u8>>, tokReceiver<Vec<u8>>);

pub fn connect<T>(reactor: &ReactorHandle, addr: SocketAddr, new_task: T)
    -> OuterTunnel where T: NewTermTask
{
    let (inner_tx, outer_rx) = sync_channel(256);
    let (outer_tx, inner_rx) = channel::sync_channel(256);
    let outer_tunnel = (outer_tx, outer_rx);
    
    reactor.oneshot(move || {
        // Create a new Tokio TcpStream from the Mio socket
        let socket = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(_) => unimplemented!(),
        };

        let timer: Timer<()> = match Timer::watch(Builder::default().build()) {
            Ok(t) => t,
            Err(_) => unimplemented!(),
        };

        let inner_rx = match tokReceiver::watch(inner_rx) {
            Ok(r) => r,
            Err(_) => unimplemented!(),
        };

        let inner_tunnel = (inner_tx, inner_rx);

        let task = match new_task.new_task(socket, timer, inner_tunnel) {
            Ok(d) => d,
            Err(_) => unimplemented!(),
        };

        try!(reactor::schedule(task));
        Ok(())
    });

    outer_tunnel
}

pub trait NewTermTask: Send + 'static {
    type Item: Task;

    fn new_task(&self, stream: TcpStream, timer: Timer<()>, tunnel: InnerTunnel) -> io::Result<Self::Item>;
}

impl<T, U> NewTermTask for T
    where T: Fn(TcpStream, Timer<()>, InnerTunnel) -> io::Result<U> + Send + 'static,
          U: Task,
{
    type Item = U;

    fn new_task(&self, stream: TcpStream, timer: Timer<()>, tunnel: InnerTunnel) -> io::Result<Self::Item> {
        self(stream, timer, tunnel)
    }
}
