use termion::{async_stdin, AsyncReader};
use std::collections::VecDeque;
use std::io::Read;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Printable(char),
    Up,
    Down,
    Left,
    Right,
    Ins,
    Del,
    Home,
    End,
    PageUp,
    PageDown,
    Close,
    Backspace,
    Return,
    Esc,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Modifier {
    None(Key),
    Alt(Key),
    Ctrl(Key),
    Shift(Key),
    Meta(Key),
}

pub enum Utf8 {
    Parsing(Option<u8>, Option<u8>, Option<u8>),
    Done(char),
    Invalid,
}

impl Utf8 {
    fn new() -> Utf8 { Utf8::Parsing(None, None, None) }
    fn extend(&self, byte: u8) -> Utf8 {
        match *self {
            Utf8::Parsing(None, None, None) => {
                if byte & 0x80 == 0 {
                    ::std::char::from_u32(byte as u32).map(|x|Utf8::Done(x)).unwrap_or(Utf8::Invalid)
                } else if byte & 0xE0 == 0xC0 ||
                    byte & 0xF0 == 0xE0 ||
                    byte & 0xF8 == 0xF0
                {
                    Utf8::Parsing(Some(byte), None, None)
                } else {
                    Utf8::Invalid
                }
            },
            Utf8::Parsing(Some(b), None, None) => {
                if byte & 0xC0 != 0x80 {
                    Utf8::Invalid
                } else if b & 0xE0 == 0xC0 {
                    let val = ((b as u32 & 0x1F) << 6) | (byte as u32 & 0x3F);
                    ::std::char::from_u32(val).map(|x|Utf8::Done(x)).unwrap_or(Utf8::Invalid)
                } else {
                    Utf8::Parsing(Some(b), Some(byte), None)
                }
            },
            Utf8::Parsing(Some(b), Some(b1), None) => {
                if byte & 0xC0 != 0x80 {
                    Utf8::Invalid
                } else if b & 0xF0 == 0xE0 {
                    let val = ((b as u32 & 0x0F) << 12) |
                        ((b1 as u32 & 0x3F) << 6) |
                        (byte as u32 & 0x3F);
                    ::std::char::from_u32(val).map(|x|Utf8::Done(x)).unwrap_or(Utf8::Invalid)
                } else {
                    Utf8::Parsing(Some(b), Some(byte), None)
                }
            },
            Utf8::Parsing(Some(b), Some(b1), Some(b2)) => {
                if byte & 0xC0 != 0x80 {
                    Utf8::Invalid
                } else if b & 0xF8 == 0xF0 {
                    let val = ((b as u32 & 0x07) << 18) |
                        ((b1 as u32 & 0x3F) << 12) |
                        ((b2 as u32 & 0x3F) << 6) |
                        (byte as u32 & 0x3F);
                    ::std::char::from_u32(val).map(|x|Utf8::Done(x)).unwrap_or(Utf8::Invalid)
                } else {
                    Utf8::Invalid
                }
            },
            _ => Utf8::Invalid,
        }
    }
}

fn is_printable(c: char) -> bool {
    let c = c as u32;
    !(c < 0x20 || (c >= 0x7f &&  c < 0xa0))
}

pub struct TermIterator {
    stream: AsyncReader,
    buffer: VecDeque<u8>,
}

impl TermIterator {
    fn new() -> Self {
        TermIterator {
            stream: async_stdin(),
            buffer: VecDeque::new(),
        }
    }
}

impl Iterator for TermIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffer.pop_front() {
            next @ Some(..) => next,
            None => {
                let mut buf = [0; 1024];
                if let Ok(bytes) = self.stream.read(&mut buf) {
                    self.buffer.extend(&buf[0..bytes])
                }
                self.buffer.pop_front()
            }
        }
    }
}

pub struct Utf8Iterator<T> {
    state: Utf8,
    stream: T,
}

impl<T> Utf8Iterator<T> where T: Iterator<Item=u8> {
    fn new(stream: T) -> Self {
        Utf8Iterator {
            state: Utf8::new(),
            stream: stream,
        }
    }
}

impl<T> Iterator for Utf8Iterator<T> where T: Iterator<Item=u8> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.stream.next();
            if c.is_none() {
                return None;
            }
            let c = c.unwrap();

            self.state = self.state.extend(c);
            match self.state {
                Utf8::Parsing(..) => continue,
                Utf8::Invalid => {
                    error!("Invalid");
                    self.state = Utf8::new();
                    continue
                },
                Utf8::Done(c) => {
                    self.state = Utf8::new();
                    return Some(c);
                },
            }
        }
    }
}

pub struct KeyIterator<T> {
    stream: T,
    escape_buffer: String,
    escaping: bool,
    alt: bool,
}

pub type KeyReader = KeyIterator<Utf8Iterator<TermIterator>>;
impl<T> KeyIterator<T> {
    pub fn stdin() -> KeyIterator<Utf8Iterator<TermIterator>> {
        KeyIterator {
            stream: Utf8Iterator::new(TermIterator::new()),
            escape_buffer: String::new(),
            escaping: false,
            alt: false,
        }
    }
}

impl<T> Iterator for KeyIterator<T> where T: Iterator<Item=char> {
    type Item = Modifier;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = if !self.escaping && self.escape_buffer.len() != 0 {
                self.escape_buffer.remove(0)
            } else {
                let next = self.stream.next();
                if next.is_none() {
                    if self.escape_buffer == "\x1b" {
                        self.escaping = false;
                        self.escape_buffer.clear();
                        if self.alt {
                            self.alt = false;
                            return Some(Modifier::Alt(Key::Esc));
                        } else {
                            return Some(Modifier::None(Key::Esc));
                        }
                    } else {
                        return None;
                    }
                } else {
                    next.unwrap()
                }
            };

            if next == '\x1b' { self.escaping = true; }

            if self.escaping {
                self.escaping = false;
                self.escape_buffer.push(next);
                let escaped_key = match &*self.escape_buffer {
                    "\x1b"   | "\x1b["  | "\x1b[2" |
                    "\x1b[3" | "\x1b[5" | "\x1b[6" |
                    "\x1b[7" | "\x1b[8" => {
                        self.escaping = true;
                        None
                    },
                    "\x1b[A" => Some(Key::Up),
                    "\x1b[B" => Some(Key::Down),
                    "\x1b[C" => Some(Key::Right),
                    "\x1b[D" => Some(Key::Left),
                    "\x1b[2~" => Some(Key::Ins),
                    "\x1b[3~" => Some(Key::Del),
                    "\x1b[5~" => Some(Key::PageUp),
                    "\x1b[6~" => Some(Key::PageDown),
                    "\x1b[7~" => Some(Key::Home),
                    "\x1b[8~" => Some(Key::End),
                    _ => None,
                };

                if escaped_key.is_some() {
                    self.escape_buffer.clear();
                    return if self.alt {
                        self.alt = false;
                        escaped_key.map(|k| Modifier::Alt(k))
                    } else {
                        escaped_key.map(|k| Modifier::None(k))
                    }
                }

                if !self.escaping {
                    self.escape_buffer.remove(0);
                    self.alt = true;
                }

                continue;
            }

            let key = match next {
                '\x03' => Some(Key::Close),
                '\x7F' => Some(Key::Backspace),
                '\x0D' => Some(Key::Return),
                '\x0A' => None,
                key => {
                    if is_printable(key) {
                        Some(Key::Printable(key))
                    } else {
                        None
                    }
                },
            };

            if key.is_none() { continue; }

            return if self.alt {
                self.alt = false;
                Some(Modifier::Alt(key.unwrap()))
            } else {
                Some(Modifier::None(key.unwrap()))
            }
        }
    }
}

