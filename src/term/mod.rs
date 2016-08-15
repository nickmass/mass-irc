mod stream;
pub use self::stream::TermStream;
mod buffer;
pub use self::buffer::{Point, Rect, TermBuffer};
mod controls;
pub use self::controls::{MessagePane, TextInput, UserInput};

use termion::{color, cursor, terminal_size, clear};
use irc::{UserInputParser, Command, UserCommand, ClientTunnel, ClientSender, ClientReceiver};
use std::thread;
use std::time::Duration;
use std::io::Write;

pub struct Terminal<S,R> where S: ClientSender, R: ClientReceiver {
    tunnel: ClientTunnel<S, R>,
    stream: TermStream,
    window: TermBuffer,
    message_pane: MessagePane,
    text_input: TextInput,
}

impl<S,R> Terminal<S,R> where S: ClientSender<Msg=UserCommand>, R: ClientReceiver<Msg=Command> {
    pub fn new(tunnel: ClientTunnel<S, R>) -> Terminal<S,R> {
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
                    match UserInputParser::parse(s) {
                        Ok(msg) => { self.tunnel.write(msg); },
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
 
            self.set_cursor();

            thread::sleep(Duration::from_millis(16));
        }
    }

    pub fn set_cursor(&mut self) {
        self.stream.write_all(&*format!("{}",
                    cursor::Goto(self.text_input.get_display_cursor(&self.window) as u16 + 1,
                    self.window.get_height() as u16)).into_bytes());
        self.stream.flush();
    }

    pub fn get_size(&self) -> (u16, u16) {
        terminal_size().unwrap()
    }
}

