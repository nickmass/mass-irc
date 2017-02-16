use term::TermStream;

use std::collections::VecDeque;

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

pub struct KeyReader {
    read_buf: VecDeque<u8>,
    escape_buf: Vec<u8>,
    alt_modifier: bool,
    utf8_mode: Utf8,
}

impl KeyReader {
    pub fn new() -> KeyReader {
        KeyReader {
            read_buf: VecDeque::new(),
            escape_buf: Vec::new(),
            alt_modifier: false,
            utf8_mode: Utf8::new(),
        }
    }

    pub fn fill(&mut self, stream: &mut TermStream) {
        let mut buf = [0;128];
        if let Ok(bytes) = stream.read(&mut buf) {
            self.read_buf.extend(&buf[0..bytes])
        }
    }

    fn read_key(&mut self) -> Option<Key> { 
        loop {
            let c = self.read_buf.pop_front();

            if c.is_none() { return None; }
            let c = c.unwrap();


            self.utf8_mode = self.utf8_mode.extend(c);
            let next = match self.utf8_mode {
                Utf8::Parsing(..) => continue,
                Utf8::Invalid => {
                    error!("Invalid");
                    self.utf8_mode = Utf8::new();
                    continue
                },
                Utf8::Done(c) => {
                    self.utf8_mode = Utf8::new();
                    c
                },
            };

            //This is not utf8 aware, but it might just sort of work??
            if next == '\x1b' || self.escape_buf.len() != 0 {
                self.escape_buf.push(c);
                let key = match self.escape_buf.as_slice() {
                    b"\x1b"   | b"\x1b["  | b"\x1b[2" |
                    b"\x1b[3" | b"\x1b[5" | b"\x1b[6" |
                    b"\x1b[7" | b"\x1b[8" => None,
                    b"\x1b[A" => Some(Key::Up),
                    b"\x1b[B" => Some(Key::Down),
                    b"\x1b[C" => Some(Key::Right),
                    b"\x1b[D" => Some(Key::Left),
                    b"\x1b[2~" => Some(Key::Ins),
                    b"\x1b[3~" => Some(Key::Del),
                    b"\x1b[5~" => Some(Key::PageUp),
                    b"\x1b[6~" => Some(Key::PageDown),
                    b"\x1b[7~" => Some(Key::Home),
                    b"\x1b[8~" => Some(Key::End),
                    _ => {
                        let _ = self.escape_buf.remove(0);
                        while self.escape_buf.len() != 0 {
                            let c = self.escape_buf.pop().unwrap();
                            self.read_buf.push_front(c);
                        }
                        if !self.alt_modifier {
                            self.alt_modifier = true;
                            None
                        } else {
                            Some(Key::Esc)
                        }
                    }
                };

                if key == None { continue; }

                self.escape_buf.clear();
                return key;
            }

            let key = match next {
                '\x03' => Some(Key::Close),
                '\x7F' => Some(Key::Backspace),
                '\x0D' => Some(Key::Return),
                '\x0A' => None,
                key => {
                    if Self::is_printable(key) {
                        Some(Key::Printable(key))
                    } else {
                        None
                    }
                },
            };

            if key == None { continue; }
            return key;
        }
    }

    fn is_printable(c: char) -> bool {
        let c = c as u32;
        !(c < 0x20 || (c >= 0x7f &&  c < 0xa0))
    }
}

impl Iterator for KeyReader {
    type Item=Modifier;
    fn next(&mut self) -> Option<Self::Item> {
        match self.read_key() {
            Some(k) => {
                if self.alt_modifier {
                    self.alt_modifier = false;
                    Some(Modifier::Alt(k))
                } else {
                    Some(Modifier::None(k))
                }
            },
            None => None
        }
    }

}
