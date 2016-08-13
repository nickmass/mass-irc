use super::super::mio::timer::Builder;

use super::{Irc, UserCommand, CommandType, CommandBuilder};

use super::super::Terminal;
use super::super::term::stream::TermStream;

use super::tokio::util::timer::Timer;
use super::tokio::tcp::TcpStream;
use super::tokio::reactor;
use super::tokio::reactor::*;
use super::tokio::io::Transport;
use super::tokio::proto::pipeline::Frame;
use std::io;
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


        connect(&reactor, addr,|stream, term, timer| {
            Ok(ClientTask::new(stream, term, timer))
        });
    }
}

pub struct ClientTask {
    term: Terminal,
    server: Irc<TcpStream>,
    timer: Timer<()>,
    tick: u64,
}

impl ClientTask  {
    fn new(stream: TcpStream, term: TermStream, mut timer: Timer<()>) -> ClientTask {
        let _ = timer.set_timeout(::std::time::Duration::from_millis(50), ());
        ClientTask {
            term: Terminal::new(term),
            server: Irc::new(stream),
            timer: timer,
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
                                    Frame::Message(CommandBuilder::new()
                                        .command(CommandType::Pong)
                                        .add_param(msg.params.data[0].clone())
                                        .build().unwrap());
                                try!(self.server.write(pong));
                                let pong =
                                    Frame::Message(CommandBuilder::new()
                                        .command(CommandType::Pong)
                                        .add_param(msg.params.data[0].clone())
                                        .build().unwrap());
                                try!(self.term.write(pong));
                            },
                            _ => {}
                        }
                        try!(self.term.write(Frame::Message(msg)));
                    }
                },
                _ => {}
            }

        }
        if let Some(Frame::Message(frame)) = try!(self.term.read()) {
            try!(self.server.write(Frame::Message(frame.to_command().unwrap())));
        }

        match self.timer.poll() {
            Some(_) => {
                let _ = self.timer.set_timeout(::std::time::Duration::from_millis(50), ());
            },
            None => ()
        }

        self.tick += 1;
        Ok(Tick::WouldBlock)
    }
}

pub fn connect<T>(reactor: &ReactorHandle, addr: SocketAddr, new_task: T)
        where T: NewTermTask
{
    reactor.oneshot(move || {
        // Create a new Tokio TcpStream from the Mio socket
        let socket = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(_) => unimplemented!(),
        };

        let term = match TermStream::new() {
            Ok(s) => s,
            Err(_) => unimplemented!(),
        };
        
        let timer: Timer<()> = match Timer::watch(Builder::default().build()) {
            Ok(t) => t,
            Err(_) => unimplemented!(),
        };


        let task = match new_task.new_task(socket, term, timer) {
            Ok(d) => d,
            Err(_) => unimplemented!(),
        };

        try!(reactor::schedule(task));
        Ok(())
    });
}

pub trait NewTermTask: Send + 'static {
    type Item: Task;

    fn new_task(&self, stream: TcpStream, term: TermStream, timer: Timer<()>) -> io::Result<Self::Item>;
}

impl<T, U> NewTermTask for T
    where T: Fn(TcpStream, TermStream, Timer<()>) -> io::Result<U> + Send + 'static,
          U: Task,
{
    type Item = U;

    fn new_task(&self, stream: TcpStream, term: TermStream, timer: Timer<()>) -> io::Result<Self::Item> {
        self(stream, term, timer)
    }
}
