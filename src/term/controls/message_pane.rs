use irc::Command;
use term::{TabToken, TermBuffer, Color, Surface, Point, Rect};
use term::term_string::TermString;
use term::buffer::Glyph;

pub struct MessagePane {
    messages: Vec<(Option<TabToken>, Message)>,
    dirty: bool,
    width: u32,
}

impl MessagePane {
    pub fn new() -> MessagePane {
        MessagePane {
            messages: Vec::new(),
            dirty: true,
            width: 0, //I don't like this
        }
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn add_server_message(&mut self, tab: Option<TabToken>, msg: String) {
        self.set_dirty();
        let index = self.messages.iter().filter(|x| x.0 == tab).count() as u32;

        let message = Message::from_server(self.width, index, msg); 

        self.messages.push((tab, message));
    }

    pub fn add_chat_message(&mut self, tab: Option<TabToken>,
                             name: String, message: String) {
        self.set_dirty();
        let index = self.messages.iter().filter(|x| x.0 == tab).count() as u32;
       
        let message = Message::from_chat(self.width, index, name, message);

        self.messages.push((tab, message));
    }

    pub fn render(&mut self, window: &mut TermBuffer, tab: Option<TabToken>) {
        if self.width != window.width() {
            self.width = window.width();
            self.set_dirty();
            self.messages = self.messages.iter().map(|x| (x.0, x.1.resize(self.width))).collect();
        }

        if !window.is_dirty() && !self.is_dirty() { return }

        let tab_messages = self.messages.iter().filter(|x| x.0 == tab);
        
        let height = window.height();
        let width = window.width();

        let mut rendered_msgs = Surface::new(Rect(Point(0, 0), width, height - 3));
        rendered_msgs.set_color(Point(0,0), Some(Color::White), Some(Color::Black));

        let mut h = (height - 3) as i32;
        for m in tab_messages.rev() {
            h -= m.1.height as i32;
            if h < 0 { break; }
            rendered_msgs.blit(&m.1.surface, Point(0, h as u32));
        }
        
        window.blit(&rendered_msgs, Point(0,2));
        self.dirty = false;
    }
}

struct Message {
    width: u32,
    height: u32,
    name: Option<String>,
    message: String,
    index: u32,
    surface: Surface,
}

impl Message {
    pub fn from_server(width: u32, index: u32, message: String) -> Message {
        let msg_len = message.len() as u32;

        let height = if msg_len % width == 0 {
            msg_len / width
        } else {
            (msg_len / width) + 1
        };
        let mut surface = Surface::new(Rect(Point(0, 0), width, height));
        
        let line_color = if index % 2 != 0 {
            "\0color:White;background:LightBlack;\0"
        } else {
            "\0color:White;background:Black;\0"
        };

        let chars: Vec<char> = message.chars().filter(|x| *x != '\r' && *x != '\n').collect();
        let mut char_count = chars.len() as u32;
        for i in 0..height {
            let mut line_buf = String::from(line_color);
            let line_width = if width < char_count {
                width
            } else {
                char_count
            };
            char_count -= line_width;
            let start = (i * width) as usize;
            let end = start + line_width as usize;
            for c in &chars[start..end] {
                line_buf.push(*c);
            }
            for i in 0..width-line_width {
                line_buf.push(' ');
            }
            surface.formatted_text(line_buf.into(), Point(0, i));
        }
        
        Message {
            width: width,
            height: height,
            name: None,
            message: message,
            index: index,
            surface: surface,
        }
    }

    pub fn from_chat(width: u32, index: u32, name: String, message: String)
            -> Message {
        let name_width = 14;
        let msg_width = width - name_width;
        let msg_len = message.len() as u32;

        let height = if msg_len % msg_width == 0 {
            msg_len / msg_width
        } else {
            (msg_len / msg_width) + 1
        };

        let mut surface = Surface::new(Rect(Point(0, 0),width, height));
        let name_string = Self::format_name(&*name, name_width);

        surface.formatted_text(name_string.into(), Point(0,0));
        for i in 1..height { //Color gutter
            surface.formatted_text(
                "\0color:White;background:Black;\0 ".to_string().into(),
                Point(0, i))
        }

        let line_color = if index % 2 != 0 {
            "\0color:White;background:LightBlack;\0"
        } else {
            "\0color:White;background:Black;\0"
        };
        
        let chars: Vec<char> = message.chars().collect();
        let mut char_count = chars.len() as u32;
        for i in 0..height {
            let mut line_buf = String::from(line_color);
            let line_width = if msg_width < char_count {
                msg_width
            } else {
                char_count
            };
            char_count -= line_width;
            let start = (i * msg_width) as usize;
            let end = start + line_width as usize;
            for c in &chars[start..end] {
                line_buf.push(*c);
            }
            for i in 0..msg_width-line_width {
                line_buf.push(' ');
            }
            surface.formatted_text(line_buf.into(), Point(name_width, i));
        }
        
        Message {
            width: width,
            height: height,
            name: Some(name),
            message: message,
            index: index,
            surface: surface,
        }
    }
    
    fn resize(&self, width: u32) -> Message {
        match self.name.clone() {
            Some(name) => {
                Message::from_chat(width, self.index, name, self.message.clone())
            },
            None => {
                Message::from_server(width, self.index, self.message.clone())
            }
        }
    }

    fn format_name(nick: &str, width: u32) -> String {
        let color_options: [&'static str; 12] = 
            [ "Blue",
            "Cyan" ,
            "Green" ,
            "LightBlue",
            "LightCyan",
            "LightGreen" ,
            "LightMagenta",
            "LightRed" ,
            "LightYellow",
            "Magenta",
            "Red" ,
            "Yellow"];
        let index = nick.bytes().fold(0, |acc, x| acc ^ x) % 12;
        
        format!("\0color:White;background:Black;\0 [\0color:{};\0{: >width$.width$}\0color:White;\0] "
                ,color_options[index as usize]
                ,nick, width = width as usize - 4)
    }
}

pub enum FlowDirection {
    TopToBottom,
    BottomToTop,
}

pub struct TextWindow {}

impl TextWindow {
    pub fn render(text: TermString, width: u32, height: u32, dir: FlowDirection) -> Surface {
        let mut surface = Surface::new(Rect(Point(0,0), width, height));
        let mut wrapped_buf = Vec::new();
        let mut current_line = TermString::new();
        let mut lines = 0;
        for glyph in text.vec() {
            if current_line.len() >= width as usize {
                wrapped_buf.push(current_line);
                current_line = TermString::new();
                lines += 1;
            }

            let &Glyph(character,_,_) = glyph;
            match character {
                '\n' => {
                    wrapped_buf.push(current_line);
                    current_line = TermString::new();
                    lines += 1;
                },
                '\r' => {},
                _ => {
                    current_line.push(*glyph);
                },
            }
        }



        /*
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
        */

        let skip = if lines > height { lines - height } else { 0 };
        let mut wrapped_lines = wrapped_buf.iter().skip(skip as usize);

        match dir {
            FlowDirection::TopToBottom => {
                let mut ind = 0;
                while ind < height && ind < lines {
                    surface.formatted_text(wrapped_lines.next().unwrap().clone(),
                        Point(0, ind));
                    ind += 1;
                }
            },
            FlowDirection::BottomToTop => {
                let mut ind = if height > lines { height - lines } else { 0 };
                while ind < height {
                    let next = wrapped_lines.next().unwrap().clone();
                    //surface.text(&next.to_string(), Point(0, ind));
                    surface.formatted_text(next, Point(0, ind));
                    ind += 1;
                }
            }
        }
        surface.set_color(Point(0,0), Some(Color::White), Some(Color::Black));
        surface
    }
}

