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

pub struct KeyReader {
    read_buf: VecDeque<u8>,
    escape_buf: VecDeque<u8>,
    alt_modifier: bool,
}

impl KeyReader {
    pub fn new() -> KeyReader {
        KeyReader {
            read_buf: VecDeque::new(),
            escape_buf: VecDeque::new(),
            alt_modifier: false,
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

            if c == b'\x1b' || self.escape_buf.len() != 0 { 
                self.escape_buf.push_back(c);
                let key = match self.escape_buf.as_slices().0 {
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
                        let esc = self.escape_buf.pop_front().unwrap();
                        while self.escape_buf.len() != 0 {
                            let c = self.escape_buf.pop_back().unwrap();
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

            let key = match c {
                3 => Some(Key::Close),
                127 => Some(Key::Backspace),
                13 => Some(Key::Return),
                10 => None,
                key => {
                    if Self::is_printable(key) {
                        Some(Key::Printable(key as char))
                    } else {
                        None
                    }
                },
            };

            if key == None { continue; }
            return key;
        }
    }
        
    fn is_printable(c: u8) -> bool {
        c >= 32 && c <= 127
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
