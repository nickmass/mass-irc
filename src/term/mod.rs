use super::termion::input::{TermRead};
use super::termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Read, Write};
use std::io;


use super::tokio::io::{Readiness, Transport};
use super::tokio::proto::pipeline;

use super::irc::{Command, UserCommand};

pub struct Terminal<I: Read + TermRead, O: Write + IntoRawMode> {
    in_stream: I,
    out_stream: RawTerminal<O>,
    read_buf: Vec<u8>,
}



impl<I: Read + TermRead, O: Write + IntoRawMode> Terminal<I, O> {
    pub fn new(in_stream: I, out_stream: O) -> Terminal<I, O> {
        Terminal {
            in_stream: in_stream,
            out_stream: out_stream.into_raw_mode().unwrap(),
            read_buf: Vec::new(),
        }
    }
}


impl<I: Read, O: Write> Readiness for Terminal<I, O> {
    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }
}

pub type InFrame = pipeline::Frame<Command, io::Error>;
pub type OutFrame = pipeline::Frame<UserCommand, io::Error>;

impl<I: Read + TermRead, O: Write> Transport for Terminal<I, O> {
    type In = InFrame;
    type Out = OutFrame;

    fn read(&mut self) -> io::Result<Option<Self::Out>> {
        if let Some(index) = self.read_buf.iter().position(|x| *x == b'\n') {
            let mut remainder = self.read_buf.split_off(index + 1);
            let mut out_buf = Vec::new();
            out_buf.append(&mut self.read_buf);
            self.read_buf.append(&mut remainder);
            return Ok(Some(pipeline::Frame::Message(
                        UserCommand::Nick(String::from_utf8(out_buf).unwrap())
                        )));
        }

        let mut buf = [0;512];
        if let Ok(bytes) = self.in_stream.read(&mut buf) {
            write!(self.out_stream, "{}", ::std::str::from_utf8(&buf).unwrap());
            self.read_buf.extend_from_slice(&mut buf[0..bytes]);
        }
        Ok(None)
    }

    fn write(&mut self, req: Self::In) -> io::Result<Option<()>>{
        match req {
            pipeline::Frame::Message(cmd) => {
                let _ = write!(self.out_stream, "{}", cmd.to_cmd());
            },
            _ => {}
        }
        Ok(None)
    }

    fn flush(&mut self) -> io::Result<Option<()>>{
        Ok(None)
    }
}
