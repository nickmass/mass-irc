use super::super::mio::{would_block};
use super::super::termion::input::{TermRead};
use super::super::termion::raw::{IntoRawMode, RawTerminal};
use super::super::termion::{async_stdin, AsyncReader};
use super::super::tokio::io::Readiness;
use std::io::{Read, Write, Result, Stdout, stdout};

pub struct TermStream {
    in_stream: AsyncReader,
    out_stream: RawTerminal<Stdout>,
}

impl TermStream {
    pub fn new() -> Result<TermStream> {
        Ok(TermStream {
            in_stream: async_stdin(),
            out_stream: try!(stdout().into_raw_mode()),
        })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.is_readable() {
            return Err(would_block())
        }

        self.in_stream.read(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if !self.is_writable() {
            return Err(would_block())
        }

        self.out_stream.write(buf)
    }
}

impl Readiness for TermStream {
    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }
}

impl Read for TermStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        TermStream::read(self, buf)
    }
}

impl Write for TermStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        TermStream::write(self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.out_stream.flush()
    }
}
