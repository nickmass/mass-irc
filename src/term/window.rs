use term::controls::{TabBar, TabToken, TabStatus, MessagePane, MessageType};
use term::{TermBuffer};

use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelToken(u32);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowToken(u32);
pub struct Window {
    token: WindowToken,
    name: String,
    tab: TabToken,
}

pub struct Channel {
    name: String,
    window: WindowToken,
    users: Vec<String>,
    new_users: Vec<String>,
}

pub struct ChatWindows {
    message_pane: MessagePane,
    tab_bar: TabBar,
    next_window: u32,
    channels: Vec<Channel>,
    tabs: HashMap<TabToken, WindowToken>,
    windows: HashMap<WindowToken, Window>,
}

impl ChatWindows {
    pub fn new(message_pane: MessagePane, tab_bar: TabBar) -> ChatWindows {
        ChatWindows {
            message_pane: message_pane,
            tab_bar: tab_bar,
            next_window: 0,
            channels: Vec::new(),
            tabs: HashMap::new(),
            windows: HashMap::new(),
        }
    }

    pub fn add_chat_message(&mut self, target: String, from: &str, to: &str,  msg: &str, m_type: MessageType) {
        match self.find_tab(&target) {
            Some(wt) => {
                let ref win = self.windows[&wt];
                self.message_pane.add_chat_message(Some(win.tab),
                    from.to_string(), msg.to_string(), m_type);
                if Some(win.tab) != self.tab_bar.active_tab() {
                    if msg.contains(to) {
                        self.tab_bar.set_alert(win.tab)
                    } else {
                        self.tab_bar.set_unread(win.tab);
                    }
                }
            },
            None => {}
        }
    }

    pub fn next_tab(&mut self) {
        match self.tab_bar.active_tab() {
            Some(t) => {
                match self.find_channel(&t) {
                    Some(token) => {
                        let mut tokens: Vec<WindowToken> = self.windows.keys()
                            .map(|x| x.clone())
                            .collect();
                        tokens.sort();
                        match tokens.binary_search(&token) {
                            Ok(pos) => {
                                if pos + 1 < tokens.len() {
                                    let token = tokens[pos + 1];
                                    let tab = self.windows[&token].tab;
                                    self.tab_bar.set_active(tab);
                                    self.message_pane.set_dirty();
                                }
                            },
                            Err(_) => {}
                        }
                    },
                    None =>  {},
                }
            },
            None => {}
        }
    }

    pub fn prev_tab(&mut self) {
        match self.tab_bar.active_tab() {
            Some(t) => {
                match self.find_channel(&t) {
                    Some(token) => {
                        let mut tokens: Vec<WindowToken> = self.windows.keys()
                            .map(|x| x.clone())
                            .collect();
                        tokens.sort();
                        match tokens.binary_search(&token) {
                            Ok(pos) => {
                                if pos > 0 {
                                    let token = tokens[pos - 1];
                                    let tab = self.windows[&token].tab;
                                    self.tab_bar.set_active(tab);
                                    self.message_pane.set_dirty();
                                }
                            },
                            Err(_) => {}}
                    },
                    None =>  {},
                }
            },
            None => {}
        }
    }

    pub fn add_server_message(&mut self, msg: String) {
        self.message_pane.add_server_message(None, msg);
    }

    pub fn set_tab(&mut self, index: u32) {
        match index {
            0 => self.server_tab(),
            _ => {
                let mut tokens: Vec<WindowToken> = self.windows.keys()
                        .map(|x| x.clone())
                        .collect();
                tokens.sort();
                match tokens.get(index as usize - 1) {
                    Some(t) => {
                        let ref tab = self.windows[t].tab;
                        self.tab_bar.set_active(tab.clone());
                        self.message_pane.set_dirty();
                    },
                    None => {}
                }
            }
        }
    }

    pub fn active_channel(&self) -> Option<&str> {
        let tab = self.tab_bar.active_tab();
        match tab {
            Some(t) => {
                match self.find_channel(&t) {
                    Some(wt) => {
                        let ref w = self.windows[&wt];
                        Some(&*w.name)
                    },
                    None => None,
                }
            },
            None => None,
        }
    }

    pub fn server_tab(&mut self) {
        self.tab_bar.clear_active();
        self.message_pane.set_dirty();
    }

    pub fn add_channel(&mut self, channel: String) {
        let tab = self.tab_bar.add_tab(channel.clone(), "".to_string(), TabStatus::Active);
        self.next_window += 1;
        let window = WindowToken(self.next_window);
        self.tabs.insert(tab.clone(), window.clone());
        self.channels.push( Channel { name: channel.clone(),
                                      window: window.clone(),
                                      users: Vec::new(),
                                      new_users: Vec::new()});
        self.windows.insert(window, Window { token: window, name: channel, tab: tab});
        self.message_pane.set_dirty();
    }

    pub fn remove_channel(&mut self, channel: &str) {
        match self.find_tab(&channel) {
            Some(wt) => {
                {
                    let ref w = self.windows[&wt];
                    self.tab_bar.remove_tab(w.tab);
                }
                self.remove_window(&wt);
                self.message_pane.set_dirty();
            },
            None => {}
        }
    }

    pub fn add_topic(&mut self, target:String, topic: String) {
        let tab = self.find_tab(&target);
        match tab {
            Some(w) => self.tab_bar.set_topic(self.windows[&w].tab, topic),
            None => {}
        }
    }

    pub fn add_names(&mut self, target: String, mut names: Vec<String>) {
        if let Some(chan) = self.channels.iter_mut().find(|x|x.name == target) {
            chan.new_users.append(&mut names);
        }
    }

    pub fn set_names(&mut self, target: String) {
        if let Some(chan) = self.channels.iter_mut().find(|x|x.name == target) {
            for name in &mut chan.new_users {
                if name.starts_with('@') || name.starts_with('+') { name.remove(0); }
            }
            chan.users.append(&mut chan.new_users);
        } 
    }

    pub fn add_name(&mut self, target: String, mut name: String) {
        if let Some(chan) = self.channels.iter_mut().find(|x|x.name == target) {
            if name.starts_with('@') || name.starts_with('+') { name.remove(0); }
            chan.users.push(name);
        }
    }

    pub fn remove_name(&mut self, target: String, mut name: String) {
        if let Some(chan) = self.channels.iter_mut().find(|x|x.name == target) {
            if name.starts_with('@') || name.starts_with('+') { name.remove(0); }
            if let Some(pos) = chan.users.iter().position(|x|x == &name) {
                chan.users.remove(pos);
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.message_pane.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.message_pane.scroll_down();
    }

    fn remove_window(&mut self, wt: &WindowToken) {
        {
            let ref window = self.windows[wt];
            self.tabs.remove(&window.tab);
            if let Some(pos) = self.channels.iter().position(|x|x.window == *wt) {
                self.channels.remove(pos);
            }
        }
        self.windows.remove(wt);
    }

    fn find_channel(&self, tab: &TabToken) -> Option<WindowToken> {
        match self.tabs.get(tab) {
            Some(wt) => {
                self.windows.get(&wt).map(|x| x.token)
            },
            None => None
        }
    }

    fn find_tab(&self, channel: &str) -> Option<WindowToken> {
        match self.channels.iter().find(|x|x.name == channel) {
            Some(c) => {
                Some(c.window)
            },
            None => None
        }
    }

    pub fn render(&mut self, window: &mut TermBuffer) {
        self.message_pane.render(window, self.tab_bar.active_tab());
        self.tab_bar.render(window);
    }
}
