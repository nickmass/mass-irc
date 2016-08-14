mod stream;
pub use self::stream::TermStream;

use termion::{color, cursor, terminal_size, clear};
use irc::{Command, UserCommand, ClientTunnel, ClientSender, ClientReceiver};
use std::thread;
use std::time::Duration;
use std::io::Write;
use std::collections::VecDeque;

pub struct Terminal<S,R> where S: ClientSender<UserCommand>, R: ClientReceiver<Command> {
    tunnel: ClientTunnel<S, R, UserCommand, Command>,
    stream: TermStream,
    message_pane: MessagePane,
    text_input: TextInput,
}

impl<S,R> Terminal<S,R> where S: ClientSender<UserCommand>, R: ClientReceiver<Command> {
    pub fn new(tunnel: ClientTunnel<S, R, UserCommand, Command>) -> Terminal<S,R> {
        Terminal {
            tunnel: tunnel,
            stream: TermStream::new().unwrap(),
            message_pane: MessagePane::new(),
            text_input: TextInput::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            loop {
                match self.tunnel.try_read() {
                    Ok(Some(m)) => self.message_pane.add_message(m),
                    Ok(None) => break,
                    Err(_) => unimplemented!(),
                }
            }

            self.message_pane.render(&mut self.stream);
            match self.text_input.read(&mut self.stream) {
                Some(UserInput::Close) => break,
                Some(UserInput::Text(s)) => {
                    let msg = UserCommand::Nick(s);
                    self.tunnel.write(msg);
                },
                _ => (),
            }
            thread::sleep(Duration::from_millis(16));
        }
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
}

impl TextInput {
    fn new() -> TextInput {
        TextInput {
            history: Vec::new(),
            line: Vec::new(),
            read_buf: VecDeque::new(),
        }
    }

    fn read(&mut self, stream: &mut TermStream) -> Option<UserInput> {
        let mut buf = [0;128];
        if let Ok(bytes) = stream.read(&mut buf) {
            self.read_buf.extend(&buf[0..bytes])
        }
                
        while self.read_buf.len() > 0 {
            match self.read_buf.pop_front() {
                Some(3) => return Some(UserInput::Close),
                Some(127) => {
                    self.line.pop();
                },
                Some(13) => {
                    let result = String::from_utf8(self.line.clone()).unwrap();
                    let mut new_item = Vec::new();
                    new_item.append(&mut self.line);
                    self.history.push(new_item);
                    return Some(UserInput::Text(result));
                },
                Some(10) => (),
                None => (),
                Some(c) => self.line.push(c),
            }
        }

        let (width, height) = terminal_size().unwrap();
        let spaces = [' ';1000];
        let line_end = (self.line.len() + 1) as u16;
        let space_end = (width - line_end) as usize;
        stream.write_all(&*format!("{}{}{}{}{}",
                                 cursor::Goto(1,height),
                                 color::Fg(color::LightWhite),
                                 ::std::str::from_utf8(&*self.line).unwrap(),
                                 String::from(&spaces[0..space_end]),
                                 cursor::Goto(line_end, height)
                                ).into_bytes());
        stream.flush();

        None
    }
}



struct MessagePane {
    messages: Vec<Command>,
}

impl MessagePane {
    fn new() -> MessagePane {
        MessagePane {
            messages: Vec::new(),
        }
    }

    fn add_message(&mut self, msg: Command) {
        self.messages.push(msg);
    }

    fn render(&self, stream: &mut TermStream) {

        let mut messages: Vec<Vec<u8>> = Vec::new(); 
        for msg in self.messages.iter().rev() {
            messages.push(msg.to_string().into_bytes());
        }

        let (width, height) = terminal_size().unwrap();
        let width = width as usize;
        let height = height as usize;

        let msg_space = width * (height - 1);
        let mut total_length = 0;
        let mut msgs_to_display = 0;
        for msg in &*messages {
            let mut msg_length = 0;
            for c in msg {
                msg_length += match c {
                    &10 => 0,
                    &13 => width - (msg_length % width),
                    _ => 1,
                };

            }
            total_length += msg_length;
            if total_length < msg_space {
                msgs_to_display += 1;
            } else {
                break;
            }
        }

        let spare_lines = height - 1 - msgs_to_display;
        let spaces = [b' ';1000];
            
        let mut out_buf = Vec::new();
        if spare_lines > 0 {
            for i in  0..spare_lines {
                out_buf.extend_from_slice(&spaces[0..width]);
                out_buf.push(b'\r');    
                out_buf.push(b'\n');    
            }
        }
        let mut messages: Vec<&Vec<u8>> = messages.as_slice().iter().collect();
        messages.reverse();
        let mut i = 0;
        for msg in messages {
            let text_end = msg.len()-2;
            let right_padding = width - (text_end % width);
            out_buf.extend_from_slice(&msg[0..text_end]);
            out_buf.extend_from_slice(&spaces[0..right_padding]);
            out_buf.append(&mut b"\r\n".to_vec());
            i += 1;
            if i > msgs_to_display {
                break;
            }
        }


        stream.write_all(&*format!("{}{}{}{}",
                                 cursor::Goto(1,1),
                                 color::Fg(color::White),
                                 String::from_utf8(out_buf).unwrap(),
                                 cursor::Goto(1,height as u16)
                                ).into_bytes());
        stream.flush();
    }
}
