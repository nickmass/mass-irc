use term::{Modifier, Key, KeyReader, Color, Surface, TermStream, TermBuffer, UserInput, Point, Rect};
use termion::cursor;

use std::io::Write;
use std::collections::VecDeque;

pub struct TextInput {
    history: VecDeque<Vec<u8>>,
    history_index: usize,
    cursor: i32,
    dirty: bool,
    reader: KeyReader,
}

impl TextInput {
    pub fn new() -> TextInput {
        let mut input = TextInput {
            history_index: 0,
            history: VecDeque::new(),
            cursor: 0,
            dirty: true,
            reader: KeyReader::new(),
        };

        input.history.push_front(Vec::new());

        input
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn read(&mut self, stream: &mut TermStream) -> Option<UserInput> {
        self.reader.fill(stream);
        let keys: Vec<Modifier> = self.reader.by_ref().collect();
        for m in keys {
            match m {
                Modifier::None(k) => {
                    match k {
                        Key::Close =>{ return Some(UserInput::Close); },
                        Key::Up => {
                            if self.history_index + 1 < self.history.len() {
                                self.set_dirty();
                                self.history_index += 1;
                                self.cursor = self.history[self.history_index].len() as i32;
                            }
                        },
                        Key::Down => {
                            if self.history_index > 0 {
                                self.set_dirty();
                                self.history_index -= 1;
                                self.cursor = self.history[self.history_index].len() as i32;
                            }

                        },
                        Key::Right => {
                            let len = self.history[self.history_index].len() as i32;
                            if self.cursor + 1 <= len {
                                self.set_dirty();
                                self.cursor += 1;
                            }
                        },
                        Key::Left => {
                            if self.cursor > 0 {
                                self.set_dirty();
                                self.cursor -= 1;
                            }
                        },
                        Key::Ins => {},
                        Key::Del => {
                            let len = self.history[self.history_index].len() as i32;
                            if self.cursor < len {
                                self.set_dirty();
                                self.cursor += 1;
                                self.delete_character();
                            }
                        },
                        Key::Home => {
                            self.set_dirty();
                            self.cursor = 0;
                        },
                        Key::End => {
                            self.set_dirty();
                            let len = self.history[self.history_index].len() as i32;
                            self.cursor = len;
                        },
                        Key::PageUp => { return Some(UserInput::ScrollUp); },
                        Key::PageDown => { return Some(UserInput::ScrollDown); },
                        Key::Backspace => {
                            self.delete_character();
                        },
                        Key::Return => {
                            let result = self.current_line();
                            self.cursor = 0;
                            if self.history_index == 0 {
                                self.history.push_front(Vec::new());
                            } else {
                                self.history_index = 0;
                                self.history[0] = Vec::new();
                            }
                            self.set_dirty();
                            return Some(UserInput::Text(result));
                        },
                        Key::Printable(c) => {
                            self.type_character(c as u8);
                        },
                        _ => {}
                    }
                },
                Modifier::Alt(k) => {
                    match k {
                        Key::Printable(c) if c.is_digit(10) => {
                            return Some(UserInput::SetTab(c as u32 - 48));
                        },
                        Key::Left => {
                            return Some(UserInput::PrevTab);
                        },
                        Key::Right => {
                            return Some(UserInput::NextTab);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        None
    }

    pub fn current_line(&self) -> String {
        String::from_utf8(self.history[self.history_index].clone()).unwrap()
    }

    fn type_character(&mut self, c: u8) {
        let len = self.history[self.history_index].len() as i32;
        if self.cursor <= len {
            self.set_dirty();
            self.history[self.history_index].insert(self.cursor as usize, c);
            self.cursor += 1;
        }
    }

    fn delete_character(&mut self) {
        if self.cursor > 0 {
            self.set_dirty();
            self.cursor -= 1;
            let _ = self.history[self.history_index].remove(self.cursor as usize);
        }
    }

    pub fn render(&mut self, window: &mut TermBuffer) {
        if !window.is_invalid() && !self.is_dirty() { return; }

        let height = window.height();
        let width = window.width();

        let mut surface = Surface::new(Rect(Point(0,0), width, 1));
        let line =  self.current_line();

        let offset = self.get_render_offset(width) as usize;
        let mut end = offset + width as usize;
        if end > line.len() { end = line.len(); }

        surface.text(&line[offset..end], Point(0,0));
        surface.set_color(Point(0,0), Some(Color::White), Some(Color::Black)); 
        window.blit(&surface, Point(0, height - 1));

        self.dirty = false;
    }

    fn get_render_offset(&self, width: i32) -> i32 {
        if self.cursor > width - 3 {
            self.cursor - (width - 3)
        } else {
            0
        }
    }

    pub fn get_display_cursor(&self, window: &TermBuffer) -> i32 {
        let width = window.width();
        self.cursor - self.get_render_offset(width)
    }

    pub fn set_cursor(&mut self, stream: &mut TermStream, window: &TermBuffer) {
        let _ = stream.write_all(&*format!("{}",
                    cursor::Goto(self.get_display_cursor(window) as u16 + 1,
                    window.height() as u16)).into_bytes());
        let _ = stream.flush();
    }
}

