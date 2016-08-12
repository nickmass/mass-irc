use super::Irc;

use super::{Command};
use super::tokio::Service;
use super::tokio::reactor::*;
use super::tokio::io::{Transport};
use super::tokio::proto::pipeline;
use std::io;
use std::net::SocketAddr;


pub struct Client<S: Service, I: Transport> {
    server: S,
    input: I,
    reactor: Reactor,
    
}

pub type ServerService = pipeline::ClientHandle<Command, Command, io::Error>;
impl<I: Transport> Client<ServerService, I> {
    pub fn new(input: I, addr: SocketAddr) ->
            Client<ServerService, I> {
        let reactor = Reactor::default().unwrap();
        let handle = &reactor.handle();
        let server_connection = pipeline::connect(handle, addr, |stream| Ok(Irc::new(stream))); 

        Client {
            server: server_connection,
            input: input,
            reactor: reactor,
        } 
    }

    pub fn connect(self) {
        self.reactor.run().unwrap();
    }
}
