use term::{Rect, Point, Color, TermBuffer, Surface};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TabToken(u32);

impl TabToken {
    fn none() -> TabToken {
        TabToken(u32::max_value())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TabStatus {
    Read,
    Unread,
    Alert,
    Active,
}

const ALERT_TICK: [&'static str; 8]  = ["|", "/", "-", "\\", "|" , "/", "-", "\\" ];

impl TabStatus {
    fn to_string(&self, tick: u32) -> String {
        match *self {
            TabStatus::Read | TabStatus::Active => " ".to_string(),
            TabStatus::Unread => "~".to_string(),
            TabStatus::Alert => ALERT_TICK[tick as usize % ALERT_TICK.len()].to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct Tab {
    token: TabToken,
    title: String,
    topic: String,
    status: TabStatus,
    tick: u32,
    next: u32,
}

impl Tab {
    fn new(token: TabToken, title: String, topic:String, status: TabStatus, tick: u32) -> Tab {
        Tab {
            token: token,
            title: title,
            topic: topic,
            status: status,
            tick: tick,
            next: 0,
        }
    }
    
    fn set_status(&mut self, status: TabStatus) {
        self.status = status;
    }

    fn get_status(&self) -> TabStatus {
        self.status
    }

    fn get_topic(&self) -> &str {
        &self.topic
    }

    fn set_topic(&mut self, topic: String) {
        self.topic = topic;
    }

    fn tick(&mut self, t: u32) -> bool {
        let m = t.wrapping_add(self.tick);
        let mut dirty = false;
        if m % 10 == 0 && self.status == TabStatus::Alert {
            self.next += 1;
            dirty = true;
        }

        dirty
    }

    fn to_string(&self, index: u32) -> String {
        format!(" {} {}. {}  ", self.status.to_string(self.next), index, self.title)
    }
}

pub struct TabBar {
    next_id: u32,
    tabs: Vec<Tab>,
    dirty: bool,
    tick: u32,
}

impl TabBar {
    pub fn new() -> TabBar {
        TabBar {
            next_id: 1,
            tabs: Vec::new(),
            dirty: true,
            tick: 0,
        }
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn add_tab(&mut self, title: String, topic: String, status: TabStatus) -> TabToken {
        if status == TabStatus::Active { self.clear_active(); }
        let token = TabToken(self.next_id);
        self.tabs.push(Tab::new(token, title, topic, status, self.tick));
        self.next_id += 1;

        token
    }

    pub fn set_topic(&mut self, tab: TabToken, topic: String) {
        if let Some(tab) = self.tabs.iter_mut()
                .find(|x| x.token == tab) {
            tab.set_topic(topic);
        }
    }

    pub fn remove_tab(&mut self, token: TabToken) {
        let tab_index = self.tabs.iter().position(|x| x.token == token);
        if let Some(tab_index) = tab_index {
            let status = self.tabs[tab_index].get_status();
            self.tabs.retain(|x| x.token != token);
            if status == TabStatus::Active {
                if tab_index < self.tabs.len() {
                    self.tabs[tab_index].set_status(TabStatus::Active);
                } else if tab_index != 0 {
                    self.tabs[tab_index - 1].set_status(TabStatus::Active);
                }
            }
        }
    }

    pub fn active_tab(&self) -> Option<TabToken> {
        if let Some(tab) = self.tabs.iter()
                .find(|x| x.get_status() == TabStatus::Active) {
            Some(tab.token)
        } else {
            None
        }
    }

    pub fn clear_active(&mut self) {
        for tab in self.tabs.iter_mut().filter(|x| x.get_status() == TabStatus::Active) {
            tab.set_status(TabStatus::Read);
        }
    }

    pub fn set_active(&mut self, tab: TabToken) {
        self.clear_active();
        let tab = self.tabs.iter_mut().find(|x| x.token == tab);
        if let Some(tab) = tab {
            tab.set_status(TabStatus::Active);
        }
    }

    pub fn set_read(&mut self, tab: TabToken) {
        let tab = self.tabs.iter_mut().find(|x| x.token == tab && x.status != TabStatus::Active);
        if let Some(tab) = tab {
            tab.set_status(TabStatus::Read);
        }
    }

    pub fn set_unread(&mut self, tab: TabToken) {
        let tab = self.tabs.iter_mut().find(|x| x.token == tab && x.status == TabStatus::Read);
        if let Some(tab) = tab {
            tab.set_status(TabStatus::Unread);
        }
    }

    pub fn set_alert(&mut self, tab: TabToken) {
        let tab = self.tabs.iter_mut().find(|x| x.token == tab && x.status != TabStatus::Active);
        if let Some(tab) = tab {
            tab.set_status(TabStatus::Alert);
        }
    }

    pub fn render(&mut self, window: &mut TermBuffer) {
        self.tick = self.tick.wrapping_add(1);
        let mut dirty = self.dirty;
        for tab in &mut self.tabs {
            dirty = dirty | tab.tick(self.tick);
        }
        if !window.is_dirty() && !dirty { return; }

        let width = window.width();
        let mut surf = Surface::new(Rect(Point(0,0), width, 2));
        let mut i = 0;
        let active_tab = self.active_tab().unwrap_or(TabToken(9999999));
        for (index, tab) in &mut self.tabs.iter_mut().enumerate() {
            let tab_str = tab.to_string(index as u32 + 1);
            let status = tab.get_status();
            let colors = match status {
                TabStatus::Active => (Color::White, Color::Red, Color::Black),
                TabStatus::Alert => (Color::LightBlack, Color::Red, Color::LightWhite),
                TabStatus::Read |
                TabStatus::Unread => (Color::LightBlack, Color::Black, Color::LightWhite)
            };

            if tab.token == active_tab {
                let topic_len = if ((width - 2) as usize) < tab.topic.len() {
                    (width - 2) as usize
                } else {
                    tab.topic.len()
                };
                surf.text(&tab.topic[0..topic_len], Point(1,1));
                surf.set_color(Point(0, 1), Some(Color::Black),
                                            Some(Color::White));
            }

            if i < width {
                surf.text(&tab_str, Point(i,0));
                surf.set_color(Point(i, 0), Some(colors.2), Some(colors.0));
                if i + 2 < width {
                    surf.set_color(Point(i + 1, 0), Some(colors.1), Some(colors.0));
                }
                if i + 3 < width {
                    surf.set_color(Point(i + 2, 0), Some(colors.2), Some(colors.0));
                }
            } else {
                break;
            }
            i += tab_str.len() as i32;
            surf.set_color(Point(i, 0), Some(Color::White), Some(Color::Black));
        }
        window.blit(&surf, Point(0,0));
    }
}
