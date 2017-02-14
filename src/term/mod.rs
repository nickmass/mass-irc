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

use irc::{Client as IrcClient, ClientEvent, UserInputParser, UserCommand};
use std::thread;
use std::time::Duration;

use log;

pub enum UserInput {
    Close,
    Text(String),
    SetTab(u32),
    PrevTab,
    NextTab,
    ScrollUp,
    ScrollDown,
}

pub struct Terminal {
    client: IrcClient,
    stream: TermStream,
    window: TermBuffer,
    chat: ChatWindows,
    text_input: TextInput,
    nickname: String,
    realname: String,
    error_recv: Option<Receiver<String>>,
}

impl Terminal {
    pub fn new(client: IrcClient, nickname: String, realname: String) -> Terminal {
        let term = Terminal {
            client: client,
            stream: TermStream::new().unwrap(),
            window: TermBuffer::new(),
            chat: ChatWindows::new(MessagePane::new(), TabBar::new()),
            text_input: TextInput::new(),
            nickname: nickname,
            realname: realname,
            error_recv: None,
        };

        term
    }

    pub fn run(mut self) {
        self.client.send_message(UserCommand::Nick(
            self.nickname.to_string()));
        self.client.send_message(UserCommand::User(
            self.nickname.to_string(),
            "8".to_string(),
            self.realname.to_string()));
        while self.client.is_connected() {
            for message in self.client.poll_messages() {
                match message {
                    ClientEvent::Command(m) => {
                        self.chat.add_server_message(m.to_string());
                    },
                    ClientEvent::ChannelMessage(channel, sender, message) => {
                        self.chat.add_chat_message(channel,
                                              sender.as_ref().map(|x| &**x)
                                                .unwrap_or(&*self.nickname),
                                              &*self.nickname,
                                              &message);
                    },
                    ClientEvent::JoinChannel(channel, sender) => {
                        let sender = sender.unwrap_or("".to_string());
                        if sender == self.nickname {
                            self.chat.add_channel(channel);
                        } else {
                            self.chat.add_name(channel, sender);
                        }
                    },
                    ClientEvent::LeaveChannel(channel, sender) => {
                        let sender = sender.unwrap_or("".to_string());
                        if sender == self.nickname {
                            self.chat.remove_channel(&channel);
                        } else {
                            self.chat.remove_name(channel, sender);
                        }
                    },
                    ClientEvent::Topic(channel, topic) => {
                        self.chat.add_topic(channel, topic);
                    },
                    ClientEvent::Names(channel, names) => {
                        self.chat.add_names(channel, names);
                    },
                    ClientEvent::NamesEnd(channel) => {
                        self.chat.set_names(channel);
                    },
                    _ => {},
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
                Some(UserInput::ScrollUp) => {
                    self.chat.scroll_up();
                },
                Some(UserInput::ScrollDown) => {
                    self.chat.scroll_down();
                },
                Some(UserInput::Text(s)) => {
                    let channel = self.chat.active_channel();

                    match UserInputParser::parse(s, channel) {
                        Ok(msg) => { let _ = self.client.send_message(msg); },
                        Err(_) =>{ error!("Unknown command") },
                    }
                },
                _ => {},
            }

            if self.error_recv.is_some() {
                match self.error_recv.as_ref().unwrap().try_recv() {
                    Ok(msg) => {
                        self.chat.add_server_message(msg);
                    },
                    _ => {}
                }
            }

            self.window.init();
            self.text_input.render(&mut self.window);
            self.chat.render(&mut self.window);
            self.window.render(&mut self.stream);
            self.text_input.set_cursor(&mut self.stream, &self.window);

            thread::sleep(Duration::from_millis(50));
        }

        drop(self.stream);
        match self.client.close() {
            Err(e) => panic!("From Client: {}", e),
            _ => {}
        }
    }

    pub fn init_log(&mut self) -> Result<(), log::SetLoggerError> {
        let (tx, rx) = channel();
        self.error_recv = Some(rx);
        log::set_logger(|max_log_level| {
            max_log_level.set(log::LogLevelFilter::Error);
            Box::new(TerminalLogger::new(tx))
        })
    }
}

struct TerminalLogger {
    log_sink: Arc<Mutex<Sender<String>>>,
}

use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};


impl TerminalLogger {
    fn new(log_sink: Sender<String>) -> TerminalLogger {
        TerminalLogger { log_sink: Arc::new(Mutex::new(log_sink)) }
    }
}

impl log::Log for TerminalLogger {
    fn enabled(&self, _metadata: &log::LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            self.log_sink.lock().unwrap().send(
                format!("{}: {} @ {:?}", record.level(), record.args(), record.location()));
        }
    }
}
