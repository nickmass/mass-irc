use irc::Command;
use term::{TabToken, TermBuffer, Color, Surface, Point, Rect};
use term::term_string::TermString;
use term::buffer::Glyph;

pub struct MessagePane {
    messages: Vec<(Option<TabToken>, TermString)>,
    dirty: bool,
}

impl MessagePane {
    pub fn new() -> MessagePane {
        MessagePane {

            messages: Vec::new(),
            dirty: true,
        }
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn add_message(&mut self, tab: Option<TabToken>, msg: String) {
        self.set_dirty();

        let term : TermString = msg.clone().into();
        self.messages.push((tab, msg.into()));
    }

    pub fn add_formatted_message(&mut self, tab: Option<TabToken>, msg: TermString) {
        self.set_dirty();

        self.messages.push((tab, msg));
    }

    pub fn render(&mut self, window: &mut TermBuffer, tab: Option<TabToken>) {
        if ! self.is_dirty() { return }

        let mut messages = TermString::new(); 
        let tab_messages = self.messages.iter().filter(|x| x.0 == tab);
        for msg in tab_messages {
            messages.extend_from_term_string(&(msg.1));
        }
        
        let height = window.height();
        let width = window.width();

        let rendered_msgs = TextWindow::render(messages,
                           width,
                           height - 3,
                           FlowDirection::BottomToTop);
        window.blit(&rendered_msgs, Point(0,2));

        self.dirty = false;
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

