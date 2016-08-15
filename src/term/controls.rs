use super::super::irc::Command;
use super::{TermStream, TermBuffer, Point, Rect};
use std::collections::VecDeque;

pub enum UserInput {
    Close,
    Text(String),
}

pub struct TextInput {
    history: VecDeque<Vec<u8>>,
    history_index: usize,
    cursor: u32,
    read_buf: VecDeque<u8>,
    dirty: bool,
}

impl TextInput {
    pub fn new() -> TextInput {
        let mut input = TextInput {
            history_index: 0,
            history: VecDeque::new(),
            cursor: 0,
            read_buf: VecDeque::new(),
            dirty: true,
        };

        input.history.push_front(Vec::new());

        input
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn read(&mut self, stream: &mut TermStream) -> Option<UserInput> {
        let mut buf = [0;128];
        if let Ok(bytes) = stream.read(&mut buf) {
            self.read_buf.extend(&buf[0..bytes])
        }
        let mut escaped = false;
        let mut escape_ready = false;
        let mut escape_number = Some(0);
        while self.read_buf.len() > 0 {
            let c = self.read_buf.pop_front();
            if escape_ready {
                escape_ready = false;
                match c {
                    Some(b'A') => { //Up
                        self.set_dirty();
                        if self.history_index + 1 < self.history.len() {
                            self.set_dirty();
                            self.history_index += 1;
                        }
                    },
                    Some(b'B') => { //Down
                        self.set_dirty();
                        if self.history_index > 0 {
                            self.set_dirty();
                            self.history_index -= 1;
                        }

                    },
                    Some(b'C') => {
                        let len = self.history[self.history_index].len() as u32;
                        if self.cursor + 1 <= len {
                            self.set_dirty();
                            self.cursor += 1;
                        }
                    }, //Right
                    Some(b'D') => { //Left
                        if self.cursor > 0 {
                            self.set_dirty();
                            self.cursor -= 1;
                        }
                    },
                    Some(c) if c == b'~' => {
                        match escape_number {
                            Some(2) => {}, //Ins
                            Some(3) => { // Delete
                                let len = self.history[self.history_index].len() as u32;
                                if self.cursor < len {
                                    self.set_dirty();
                                    self.cursor += 1;
                                    self.delete_character();
                                }
                            },
                            Some(7) => {
                                self.set_dirty();
                                self.cursor = 0;
                            }, //Home
                            Some(8) => {
                                self.set_dirty();
                                let len = self.history[self.history_index].len() as u32;
                                self.cursor = len;
                            }, //End
                            Some(5) => {}, //PageUp
                            Some(6) => {}, //PageDown
                            _ => {
                                escape_number = None;
                                escape_ready = false;
                            }
                        }
                    },
                    Some(b'2') => { // Ins
                        escape_ready = true;
                        escape_number = Some(2);
                    },
                    Some(b'3') => { // Delete
                        escape_ready = true;
                        escape_number = Some(3);
                    },
                    Some(b'7') => { // Home
                        escape_ready = true;
                        escape_number = Some(7);
                    },
                    Some(b'8') => { // End
                        escape_ready = true;
                        escape_number = Some(8);
                    },
                    Some(b'5') => { // Page Up
                        escape_ready = true;
                        escape_number = Some(5);
                    },
                    Some(b'6') => { // Page Down
                        escape_ready = true;
                        escape_number = Some(6);
                    },
                    _ => {
                        escape_ready = false;
                        escape_number = None;
                    }
                }
                continue;
            }
            match c {
                Some(3) => return Some(UserInput::Close),
                Some(127) => {
                    self.delete_character();
                },
                Some(13) => {
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
                Some(27) => (escaped = true),
                Some(b'[') =>{
                    if escaped {
                        escape_ready = true;
                    } else {
                        self.type_character(b'[');
                    }
                    escaped = false;
                },
                Some(10) => (),
                None => (),
                Some(c) => {
                    if is_printable(c) {
                        self.type_character(c);
                    }
                },
            }

        }
        
        None
    }

    pub fn current_line(&self) -> String {
        String::from_utf8(self.history[self.history_index].clone()).unwrap()
    }

    fn type_character(&mut self, c: u8) {
        let len = self.history[self.history_index].len() as u32;
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
        if !self.is_dirty() { return; }

        let height = window.get_height();
        let width = window.get_width();

        let line =  self.current_line();

        let offset = self.get_render_offset(width);

        let mut buf = line[offset as usize ..].as_bytes().to_vec();
        buf.truncate(width as usize);

        let mut line_buf = Vec::new();
        line_buf.push(buf);

        window.draw(line_buf, Rect(Point(0, height), width, height));
       
        self.dirty = false;
    }

    fn get_render_offset(&self, width: u32) -> u32 {
        if self.cursor > width - 3 {
            self.cursor - (width - 3)
        } else {
            0
        }
    }

    pub fn get_display_cursor(&self, window: &TermBuffer) -> u32 {
        let width = window.get_width();
        self.cursor - self.get_render_offset(width)
    }
}



pub struct MessagePane {
    messages: Vec<Command>,
    dirty: bool,
}

impl MessagePane {
    pub fn new() -> MessagePane {
        MessagePane {

            messages: Vec::new(),
            dirty: true,
        }
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn add_message(&mut self, msg: Command) {
        self.set_dirty();
        self.messages.push(msg);
    }

    pub fn render(&mut self, window: &mut TermBuffer) {
        if ! self.is_dirty() { return }

        let mut messages = String::new(); 
        for msg in &self.messages {
            messages.push_str(&*msg.clone().to_string());
        }
        
        let height = window.get_height();
        let width = window.get_width();

        let rendered_msgs = TextWindow::render(&*messages,
                           width,
                           height,
                           FlowDirection::TopToBottom);
        window.draw(rendered_msgs, Rect(Point(0,0), width, height));
        self.dirty = false;
    }
}

pub enum FlowDirection {
    TopToBottom,
    BottomToTop,
}

pub struct TextWindow {}

impl TextWindow {
    pub fn render(text: &str, width: u32, height: u32, dir: FlowDirection) -> Vec<Vec<u8>> {
        let mut wrapped_buf = String::new();
        let mut lines = 0;
        for line in text.lines() {
            let mut current_width = 0;
            for word in line.split_whitespace() {
                let new_width = current_width + word.len();
                if new_width >= width as usize {
                    current_width = word.len();
                    lines += 1;
                    wrapped_buf.push('\n');
                } else {
                    if current_width != 0 { wrapped_buf.push(' '); }
                    current_width = new_width + 1;
                }
                wrapped_buf.push_str(word);
            }
            if current_width > 0 {
                lines += 1;
                wrapped_buf.push('\n')
            }
        }


        let mut wrapped_lines: Vec<String> = wrapped_buf.lines().map(String::from).collect();

        match dir {
            FlowDirection::BottomToTop => {
                let mut wrapped_lines: Vec<String> = wrapped_buf.lines().map(String::from).rev().collect();
                while lines < height {
                    wrapped_lines.push(String::new());
                    lines += 1;
                }

                let mut flipped_buf = VecDeque::new();
                for line in wrapped_lines.drain(0..height as usize) {
                    flipped_buf.push_front(line.into_bytes());
                    lines -= 1;
                    if lines == 0 { break; }
                }

                flipped_buf.into()
            },
            FlowDirection::TopToBottom => {
                while lines < height {
                    wrapped_buf.push('\n');
                    lines += 1;
                }
                
                let mut wrapped_lines: Vec<String> = wrapped_buf.lines().map(String::from).collect();

                let mut buf = Vec::new();
                let total_lines = wrapped_lines.len();
                for line in wrapped_lines.drain(total_lines - height as usize .. total_lines) {
                    buf.push(line.into_bytes());
                    lines -= 1;
                    if lines == 0 { break; }
                }

                buf
            }
        }
    }
}

fn is_printable(c: u8) -> bool {
    c >= 32 && c <= 127
}
