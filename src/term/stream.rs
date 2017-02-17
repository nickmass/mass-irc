use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Read, Write, Result, Stdout, stdout};

pub struct TermStream {
    out_stream: RawTerminal<Stdout>,
}

impl TermStream {
    pub fn new() -> Result<TermStream> {
        Ok(TermStream {
            out_stream: try!(stdout().into_raw_mode()),
        })
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

impl Write for TermStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        TermStream::write(self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.out_stream.flush()
    }
}
