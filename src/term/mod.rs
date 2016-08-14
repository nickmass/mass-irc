mod stream;
pub use self::stream::TermStream;
mod term_buffer;

use self::term_buffer::{Point, TermBuffer};

use termion::{color, cursor, terminal_size, clear};
use irc::{Command, UserCommand, ClientTunnel, ClientSender, ClientReceiver};
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::collections::VecDeque;

pub struct Terminal<S,R> where S: ClientSender<UserCommand>, R: ClientReceiver<Command> {
    tunnel: ClientTunnel<S, R, UserCommand, Command>,
    stream: TermStream,
    window: TermBuffer,
    message_pane: MessagePane,
    text_input: TextInput,
}

impl<S,R> Terminal<S,R> where S: ClientSender<UserCommand>, R: ClientReceiver<Command> {
    pub fn new(tunnel: ClientTunnel<S, R, UserCommand, Command>) -> Terminal<S,R> {
        let size  = terminal_size().unwrap();
        Terminal {
            tunnel: tunnel,
            stream: TermStream::new().unwrap(),
            window: TermBuffer::new(size.0 as u32, size.1 as u32),
            message_pane: MessagePane::new(),
            text_input: TextInput::new(),
        }
    }

    pub fn run(&mut self) {
        let mut size = self.get_size();
        loop {
            loop {
                match self.tunnel.try_read() {
                    Ok(Some(m)) => {
                        self.message_pane.add_message(m);
                        //Must redraw text box too as msesage pane overlaps
                        self.text_input.set_dirty();
                    },
                    Ok(None) => break,
                    Err(_) => unimplemented!(),
                }
            }

            // Force immediate redraw if term changes sizes
            let new_size = self.get_size();
            if new_size != size {
                self.window.resize(new_size.0 as u32, new_size.1 as u32);
                size = new_size;
            } 

            match self.text_input.read(&mut self.stream) {
                Some(UserInput::Close) => break,
                Some(UserInput::Text(s)) => {
                    let msg = UserCommand::Nick(s);
                    self.tunnel.write(msg);
                },
                _ => (),
            }

            if self.window.is_dirty() ||
                self.message_pane.is_dirty() ||
                self.text_input.is_dirty() {
                self.message_pane.set_dirty();
                self.text_input.set_dirty();
            }

            self.message_pane.render(&mut self.window);
            self.text_input.render(&mut self.window);
            self.window.render(&mut self.stream);
 
            self.set_cursor();

            thread::sleep(Duration::from_millis(16));
        }
    }

    pub fn set_cursor(&mut self) {
        self.stream.write_all(&*format!("{}",
                    cursor::Goto(self.text_input.get_cursor() as u16 + 1,
                    self.window.get_height() as u16)).into_bytes());
        self.stream.flush();
    }

    pub fn get_size(&self) -> (u16, u16) {
        terminal_size().unwrap()
    }
}

enum UserInput {
    Close,
    Text(String),
}

struct TextInput {
    history: Vec<Vec<u8>>,
    line: Vec<u8>,
    read_buf: VecDeque<u8>,
    dirty: bool,
}

impl TextInput {
    fn new() -> TextInput {
        TextInput {
            history: Vec::new(),
            line: Vec::new(),
            read_buf: VecDeque::new(),
            dirty: true,
        }
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    fn read(&mut self, stream: &mut TermStream) -> Option<UserInput> {
        let mut buf = [0;128];
        if let Ok(bytes) = stream.read(&mut buf) {
            self.read_buf.extend(&buf[0..bytes])
        }
                
        while self.read_buf.len() > 0 {
            match self.read_buf.pop_front() {
                Some(3) => return Some(UserInput::Close),
                Some(127) => {
                    self.set_dirty();
                    self.line.pop();
                },
                Some(13) => {
                    let result = String::from_utf8(self.line.clone()).unwrap();
                    let mut new_item = Vec::new();
                    new_item.append(&mut self.line);
                    self.history.push(new_item);
                    self.set_dirty();
                    return Some(UserInput::Text(result));
                },
                Some(10) => (),
                None => (),
                Some(c) => {
                    self.set_dirty();
                    self.line.push(c)
                },
            }
        }
        
        None
    }

    pub fn render(&mut self, window: &mut TermBuffer) {
        if !self.is_dirty() { return; }

        let mut buf = Vec::new();
        buf.push(self.line.clone());

        let height = window.get_height();
        let width = window.get_width();

        window.draw(buf, Point(0, height), width);
       
        self.dirty = false;
    }

    pub fn get_cursor(&self) -> usize {
        self.line.len()
    }
}



struct MessagePane {
    messages: Vec<Command>,
    dirty: bool,
}

impl MessagePane {
    fn new() -> MessagePane {
        MessagePane {

            messages: Vec::new(),
            dirty: true,
        }
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    fn add_message(&mut self, msg: Command) {
        self.set_dirty();
        self.messages.push(msg);
    }

    fn render(&mut self, window: &mut TermBuffer) {
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
        window.draw(rendered_msgs, Point(0,0), width);
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
