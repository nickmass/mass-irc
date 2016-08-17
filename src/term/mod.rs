mod stream;
pub use self::stream::TermStream;
mod buffer;
pub use self::buffer::{Color, Point, Rect, Surface, TermBuffer};
pub mod controls;
pub use self::controls::{MessagePane, TextInput, TabBar, TabStatus, TabToken};
mod keys;
pub use self::keys::{Key, KeyReader};

use irc::{CommandType, CommandBuilder, ClientEvent, UserInputParser, Command, UserCommand, ClientTunnel, ClientSender, ClientReceiver};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

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
    tab_bar: TabBar,
    channels: HashMap<String, TabToken>,
}

impl<S,R> Terminal<S,R> where S: ClientSender<Msg=UserCommand>, R: ClientReceiver<Msg=ClientEvent> {
    pub fn new(tunnel: ClientTunnel<S, R>) -> Terminal<S,R> {
        let term = Terminal {
            tunnel: tunnel,
            stream: TermStream::new().unwrap(),
            window: TermBuffer::new(),
            message_pane: MessagePane::new(),
            text_input: TextInput::new(),
            tab_bar: TabBar::new(),
            channels: HashMap::new(),
        };
        
        term
    }

    pub fn run(&mut self) {
        let me = "NickMass";
        loop {
            loop {
                match self.tunnel.try_read() {
                    Ok(Some(ClientEvent::Command(m))) => {
                        self.message_pane.add_message(None, m.to_string());
                    },
                    Ok(Some(ClientEvent::ChannelMessage(channel, sender, message))) => {
                        match self.channels.get(&channel) {
                            Some(tab) => {
                                let msg = format!("[{: >13.13}]: {}\r\n", sender.unwrap_or(me.to_string()), message);
                                self.message_pane.add_message(Some(*tab), msg);
                            },
                            None => {},
                        }
                    },
                    Ok(Some(ClientEvent::JoinChannel(channel, sender))) => {
                        if sender.unwrap_or("".to_string()) == me {
                            let tab = self.tab_bar.add_tab(channel.to_string(),
                                                           "".to_string(),
                                                           TabStatus::Active);
                            self.channels.insert(channel.to_string(), tab);
                        }
                    },
                    Ok(Some(ClientEvent::LeaveChannel(channel, sender))) => {
                        if sender.unwrap_or("".to_string()) == me {
                            match self.channels.remove(&channel) {
                                Some(tab) => {
                                    self.tab_bar.remove_tab(tab);
                                },
                                None => {}
                            }
                        }
                    },
                    Ok(Some(ClientEvent::Topic(channel, topic))) => {
                        match self.channels.get(&channel) {
                            Some(tab) => { 
                                self.tab_bar.set_topic(*tab, topic.to_string()); 
                            },
                            None => {}
                        }
                    },
                    Ok(None) => break,
                    Ok(_) => {},
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
                self.tab_bar.set_dirty();
            }

            self.message_pane.render(&mut self.window, self.tab_bar.active_tab());
            self.text_input.render(&mut self.window);
            self.tab_bar.render(&mut self.window);
            self.window.render(&mut self.stream);
            self.text_input.set_cursor(&mut self.stream, &self.window);

            thread::sleep(Duration::from_millis(16));
        }
    }
}

