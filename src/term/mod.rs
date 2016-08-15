mod stream;
pub use self::stream::TermStream;
mod buffer;
pub use self::buffer::{Point, Rect, TermBuffer};
mod controls;
pub use self::controls::{MessagePane, TextInput};

use irc::{UserInputParser, Command, UserCommand, ClientTunnel, ClientSender, ClientReceiver};
use std::thread;
use std::time::Duration;

pub enum UserInput {
    Close,
    Text(String),
}

pub struct Terminal<S,R> where S: ClientSender, R: ClientReceiver {
    tunnel: ClientTunnel<S, R>,
    stream: TermStream,
    window: TermBuffer,
    message_pane: MessagePane,
    text_input: TextInput,
}

impl<S,R> Terminal<S,R> where S: ClientSender<Msg=UserCommand>, R: ClientReceiver<Msg=Command> {
    pub fn new(tunnel: ClientTunnel<S, R>) -> Terminal<S,R> {
        Terminal {
            tunnel: tunnel,
            stream: TermStream::new().unwrap(),
            window: TermBuffer::new(),
            message_pane: MessagePane::new(),
            text_input: TextInput::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            loop {
                match self.tunnel.try_read() {
                    Ok(Some(m)) => {
                        self.message_pane.add_message(m);
                    },
                    Ok(None) => break,
                    Err(_) => unimplemented!(),
                }
            }

            match self.text_input.read(&mut self.stream) {
                Some(UserInput::Close) => break,
                Some(UserInput::Text(s)) => {
                    match UserInputParser::parse(s) {
                        Ok(msg) => { let _ = self.tunnel.write(msg); },
                        Err(_) =>{ unimplemented!(); },
                    }
                },
                _ => (),
            }

            if self.window.is_dirty() {
                self.message_pane.set_dirty();
                self.text_input.set_dirty();
            }

            self.message_pane.render(&mut self.window);
            self.text_input.render(&mut self.window);
            self.window.render(&mut self.stream);
 
            self.text_input.set_cursor(&mut self.stream, &self.window);

            thread::sleep(Duration::from_millis(16));
        }
    }
}

