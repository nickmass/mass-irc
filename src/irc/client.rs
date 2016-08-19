use mio::timer::Builder;
use mio::channel::{sync_channel as mio_sync_channel, SyncSender as MioSyncSender};

use irc::{Irc, UserCommand, CommandType, ClientEvent, CommandBuilder, ClientTunnel};

use tokio::util::channel::Receiver as TokioReceiver;
use tokio::util::timer::Timer;
use tokio::tcp::TcpStream;
use tokio::reactor;
use tokio::reactor::*;
use tokio::io::Transport;
use tokio::proto::pipeline::Frame;

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::net::SocketAddr;
use std::io;

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
    fn new(stream: TcpStream, timer: Timer<()>, tunnel: InnerTunnel) -> ClientTask {
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
            let _ = self.tunnel.try_write(ClientEvent::Connected);
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
                            },
                            _ => {  }

                        }
                        match ClientEvent::from_command(&msg) {
                            Some(ev) => { let _ = self.tunnel.try_write(ev); },
                            _ => {  },
                        }
                        let _ = self.tunnel.try_write(ClientEvent::Command(msg));
                    }
                },
                _ => {}
            }

        }

        match self.tunnel.try_read() {
            Ok(Some(d)) => {
                let command = d.to_command().unwrap();
                let _ = self.tunnel.try_write(ClientEvent::Command(command.clone()));
                match command.command {
                    CommandType::PrivMsg => {
                        match ClientEvent::from_command(&command) {
                            Some(ev) => { let _ = self.tunnel.try_write(ev); },
                            _ => {},
                        }
                    }
                    _ => {},
                }
                try!(self.server.write(Frame::Message(command)));
            },
            _ => (),
        }

        self.tick += 1;
        Ok(Tick::WouldBlock)
    }
}

pub type OuterTunnel = ClientTunnel<MioSyncSender<UserCommand>, Receiver<ClientEvent>>;
pub type InnerTunnel = ClientTunnel<SyncSender<ClientEvent>, TokioReceiver<UserCommand>>;

pub fn connect<T>(reactor: &ReactorHandle, addr: SocketAddr, new_task: T)
    -> OuterTunnel where T: NewTermTask
{
    let (inner_tx, outer_rx) = sync_channel(256);
    let (outer_tx, inner_rx) = mio_sync_channel(256);
    let outer_tunnel = ClientTunnel::new(outer_tx, outer_rx);
    
    reactor.oneshot(move || {
        let socket = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(_) => unimplemented!(),
        };

        let timer: Timer<()> = match Timer::watch(Builder::default().build()) {
            Ok(t) => t,
            Err(_) => unimplemented!(),
        };

        let inner_rx = match TokioReceiver::watch(inner_rx) {
            Ok(r) => r,
            Err(_) => unimplemented!(),
        };

        let inner_tunnel = ClientTunnel::new(inner_tx, inner_rx);

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
