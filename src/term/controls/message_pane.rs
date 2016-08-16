use irc::Command;
use term::{TermBuffer, Color, Surface, Point, Rect};

pub struct MessagePane {
    messages: Vec<Command>,
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

    pub fn add_message(&mut self, msg: Command) {
        self.set_dirty();
        self.messages.push(msg);
    }

    pub fn render(&mut self, window: &mut TermBuffer) {
        if ! self.is_dirty() { return }

        let mut messages = String::new(); 
        for msg in &self.messages {
            messages.push_str(&*msg.clone().to_string());
        }
        
        let height = window.height();
        let width = window.width();

        let rendered_msgs = TextWindow::render(&*messages,
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
    pub fn render(text: &str, width: u32, height: u32, dir: FlowDirection) -> Surface {
        let mut surface = Surface::new(Rect(Point(0,0), width, height));
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

        let skip = if lines > height { lines - height } else { 0 };
        let mut wrapped_lines = wrapped_buf.lines().skip(skip as usize);

        match dir {
            FlowDirection::TopToBottom => {
                let mut ind = 0;
                while ind < height && ind < lines {
                    surface.text(wrapped_lines.next().unwrap(),
                        Point(0, ind));
                    ind += 1;
                }
            },
            FlowDirection::BottomToTop => {
                let mut ind = if height > lines { height - lines } else { 0 };
                while ind < height {
                    surface.text(wrapped_lines.next().unwrap(),
                        Point(0, ind));
                    ind += 1;
                }
            }
        }
        surface.set_color(Point(0,0), Some(Color::White), Some(Color::Black));
        surface
    }
}

