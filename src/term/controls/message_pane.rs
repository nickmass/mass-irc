use irc::Command;
use term::{TermBuffer, Point, Rect};
use std::collections::VecDeque;

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
        
        let height = window.height();
        let width = window.width();

        let rendered_msgs = TextWindow::render(&*messages,
                           width,
                           height - 1,
                           FlowDirection::BottomToTop);
        window.draw(rendered_msgs, Rect(Point(0,0), width, height - 1));
        self.dirty = false;
    }
}

pub enum FlowDirection {
    TopToBottom,
    BottomToTop,
}

pub struct TextWindow {}

impl TextWindow {
    pub fn render(text: &str, width: u32, height: u32, dir: FlowDirection) -> Vec<u8> {
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

        match dir {
            FlowDirection::TopToBottom => {
                let mut wrapped_lines: Vec<String> = wrapped_buf.lines().map(String::from).rev().collect();
                while lines < height {
                    wrapped_lines.push(String::new());
                    lines += 1;
                }

                let mut flipped_buf = VecDeque::new();
                for line in wrapped_lines.drain(0..height as usize) {
                    let len = line.len() as u32;
                    flipped_buf.extend(line.into_bytes());
                    for _ in len..width { flipped_buf.push_back(b' ') }
                    lines -= 1;
                    if lines == 0 { break; }
                }

                flipped_buf.into()
            },
            FlowDirection::BottomToTop => {
                while lines < height {
                    wrapped_buf.push('\n');
                    lines += 1;
                }
                
                let mut wrapped_lines: Vec<String> = wrapped_buf.lines().map(String::from).collect();

                let mut buf = Vec::new();
                let total_lines = wrapped_lines.len();
                for line in &mut wrapped_lines.drain(total_lines - height as usize .. total_lines) {
                    let len = line.len() as u32;
                    buf.append(&mut line.into_bytes());
                    for _ in len..width { buf.push(b' ') }
                    lines -= 1;
                    if lines == 0 { break; }
                }

                buf
            }
        }
    }
}

