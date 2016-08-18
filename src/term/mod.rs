mod stream;
pub use self::stream::TermStream;
pub mod buffer;
pub use self::buffer::{Color, Point, Rect, Surface, TermBuffer};
pub mod controls;
pub use self::controls::{MessagePane, TextInput, TabBar, TabStatus, TabToken};
mod keys;
pub use self::keys::{Modifier, Key, KeyReader};
pub mod term_string;
pub use self::term_string::{TermString};
mod window;
use self::window::ChatWindows;

use irc::{ClientEvent, UserInputParser, UserCommand, ClientTunnel, ClientSender, ClientReceiver};
use std::thread;
use std::time::Duration;

pub enum UserInput {
    Close,
    Text(String),
    SetTab(u32),
    PrevTab,
    NextTab,
}

pub struct Terminal<S,R> where S: ClientSender, R: ClientReceiver {
    tunnel: ClientTunnel<S, R>,
    stream: TermStream,
    window: TermBuffer,
    chat: ChatWindows,
    text_input: TextInput,
    nickname: String,
    realname: String,
}

impl<S,R> Terminal<S,R> where S: ClientSender<Msg=UserCommand>, R: ClientReceiver<Msg=ClientEvent> {
    pub fn new(tunnel: ClientTunnel<S, R>, nickname: String, realname: String) ->
            Terminal<S,R> {
        let term = Terminal {
            tunnel: tunnel,
            stream: TermStream::new().unwrap(),
            window: TermBuffer::new(),
            chat: ChatWindows::new(MessagePane::new(), TabBar::new()),
            text_input: TextInput::new(),
            nickname: nickname,
            realname: realname,
        };
        
        term
    }

    pub fn run(&mut self) {
        loop {
            loop {
                match self.tunnel.try_read() {
                    Ok(Some(ClientEvent::Connected)) => {
                        let _ = self.tunnel.write(UserCommand::Nick(
                                self.nickname.to_string()));
                        let _ = self.tunnel.write(UserCommand::User(
                                self.nickname.to_string(), 
                                "8".to_string(), 
                                self.realname.to_string()));
                    },
                    Ok(Some(ClientEvent::Command(m))) => {
                        self.chat.add_server_message(m.to_string());
                    },
                    Ok(Some(ClientEvent::ChannelMessage(channel, sender, message))) => {
                        self.chat.add_chat_message(channel,
                                              sender.as_ref().map(|x| &**x)
                                                .unwrap_or(&*self.nickname),
                                              &message);
                    },
                    Ok(Some(ClientEvent::JoinChannel(channel, sender))) => {
                        if sender.unwrap_or("".to_string()) == self.nickname {
                            self.chat.add_channel(channel);
                        }
                    },
                    Ok(Some(ClientEvent::LeaveChannel(channel, sender))) => {
                        if sender.unwrap_or("".to_string()) == self.nickname {
                            self.chat.remove_channel(&channel);
                        }
                    },
                    Ok(Some(ClientEvent::Topic(channel, topic))) => {
                        self.chat.add_topic(channel, topic);
                    },
                    Ok(None) => break,
                    Ok(_) => {},
                    Err(_) => unimplemented!(),
                }
            }

            match self.text_input.read(&mut self.stream) {
                Some(UserInput::Close) => break,
                Some(UserInput::SetTab(c)) => {
                    self.chat.set_tab(c);
                },
                Some(UserInput::PrevTab) => {
                    self.chat.prev_tab();
                },
                Some(UserInput::NextTab) => {
                    self.chat.next_tab();
                },
                Some(UserInput::Text(s)) => {
                    let channel = self.chat.active_channel();

                    match UserInputParser::parse(s, channel) {
                        Ok(msg) => { let _ = self.tunnel.write(msg); },
                        Err(_) =>{ unimplemented!(); },
                    }
                },
                _ => (),
            }

            if self.window.is_dirty() {
                self.text_input.set_dirty();
            }

            self.text_input.render(&mut self.window);
            self.chat.render(&mut self.window);
            self.window.render(&mut self.stream);
            self.text_input.set_cursor(&mut self.stream, &self.window);

            thread::sleep(Duration::from_millis(16));
        }
    }
}
