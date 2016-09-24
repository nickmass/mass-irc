use termion::raw::{IntoRawMode, RawTerminal};
use termion::{async_stdin, AsyncReader};
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
        self.in_stream.read(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.out_stream.write(buf)
    }
}

impl Drop for TermStream {
    fn drop(&mut self) {
        let _ = self.write(b"\n");
        let _ = self.flush();
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
