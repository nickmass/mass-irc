extern crate tokio;
extern crate futures;
use self::tokio::{Service, client};
use self::tokio::tcp::TcpStream;
use self::tokio::reactor::*;
use self::tokio::io::{TryWrite, TryRead, Readiness, Transport};
use self::tokio::proto::pipeline;
use self::tokio::proto::pipeline::{Server, Frame};
use self::futures::{Future};
use std::io;

use super::parse::{Command, CommandParser};

pub struct Client {}
impl Client {
    pub fn start() {
        let reactor = Reactor::default().unwrap();

        let stream = TcpStream::connect(&"127.0.0.1:6667".parse().unwrap()).unwrap();
        let server = Server::new(
            /*
            tokio::simple_service(|r| {
                Ok(CommandParser::new(b"NICK nickmass".to_vec()).parse())
            })
            */
            IrcService::new()
            , IrcTransport::new(stream)).unwrap();
        let handle = &reactor.handle();

        handle.schedule(server);

        reactor.run().unwrap();
    }
}

enum ServiceError {
    Transport(TransportError),
    Service,
}

impl From<pipeline::Error<TransportError>> for ServiceError {
    fn from(e: pipeline::Error<TransportError>) -> Self {
        ServiceError::Service
    }
}
impl From<TransportError> for ServiceError {
    fn from(e: TransportError) -> Self {
        ServiceError::Transport(e)
    }
}

enum TransportError {
    Something,
    Bad
}

struct IrcService {}
impl IrcService {
    fn new() -> IrcService {
        IrcService {}
    }
}

impl Service for IrcService {
    type Req = Command;
    type Resp = Command;
    type Error = ServiceError;
    type Fut = Box<Future<Item = Self::Resp, Error = Self::Error>>;

    fn call(&self, req: Command) -> Self::Fut {
        println!("Req: {:?}", req.to_cmd());
        futures::finished(CommandParser::new(b"NICK nickmass".to_vec()).parse()).boxed()
    }
}

struct IrcTransport<T> {
    stream: T,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

impl<T: TryRead + TryWrite + Readiness> IrcTransport<T> {
    fn new(stream: T) -> IrcTransport<T> {
        let mut write_buf = Vec::new();
        write_buf.append(&mut b"USER nickmass 8 * :Nick Massey\r\n".to_vec());
        write_buf.append(&mut b"NICK nickmass\r\n".to_vec());
        IrcTransport { stream: stream, read_buf: Vec::new(), write_buf: write_buf }
    }
}

impl<T> Readiness for IrcTransport<T> {
    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }
}

impl<T: TryRead + TryWrite + Readiness> Transport for IrcTransport<T> {
    type In = Command;
    type Out = Command;

    fn read(&mut self) -> io::Result<Option<Self::Out>> {
        let mut buf = Vec::new();
        if let Some(bytes) = try!(self.stream.try_read(&mut buf)) {
            if bytes > 0 {
                println!("read");
                println!("{}", ::std::str::from_utf8(&*buf).unwrap());
                buf.truncate(bytes);
                let mut count = 0;
                for i in &*buf {
                    count += 1;
                    if *i == 0x10 {
                        break;
                    }
                }

                let remainder = buf.split_off(count);
                self.read_buf.append(&mut buf);
                if self.read_buf.len() > 0 {
                    println!("{}", ::std::str::from_utf8(&*self.read_buf).unwrap());
                    return Ok(Some(CommandParser::new(self.read_buf.clone()).parse()));
                }
                self.read_buf = remainder;
            }
        }
        Ok(None)
    }

    fn write(&mut self, req: Self::In) -> io::Result<Option<()>>{
        self.write_buf.append(&mut req.to_cmd().into_bytes());
        if self.write_buf.len() > 0 {
            if let Some(bytes) = try!(self.stream.try_write(&*self.write_buf)) {
                println!("{}", ::std::str::from_utf8(&*self.write_buf).unwrap());
                self.write_buf = self.write_buf.split_off(bytes);
                return Ok(Some(()));
            }
        }
        Ok(None)
    }

    fn flush(&mut self) -> io::Result<Option<()>>{
        Ok(None)
    }
}
