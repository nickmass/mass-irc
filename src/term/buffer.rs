use termion::{color, cursor, terminal_size};
use term::TermStream;
use std::io::Write;

#[derive(Clone, Copy, Debug)]
pub struct Point(pub u32, pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Rect(pub Point, pub u32, pub u32);

impl Rect {
    pub fn left(&self) -> u32 {
        (self.0).0
    }

    pub fn right(&self) -> u32 {
        (self.0).0 + self.1
    }

    pub fn top(&self) -> u32 {
        (self.0).1
    }

    pub fn bottom(&self) -> u32 {
        (self.0).1 + self.2
    }
}

pub struct TermBuffer {
    out_buf: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    dirty: bool,
}

impl TermBuffer {
    pub fn new() -> TermBuffer {
        let mut buf = TermBuffer {
            out_buf: Vec::new(),
            width: 0,
            height: 0,
            dirty: true,
        };

        buf.init();

        buf
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn get_height(&self) -> u32 { self.height }
    pub fn get_width(&self) -> u32 { self.width }

    fn init(&mut self) {
        let size  = terminal_size().unwrap();
        let width = size.0 as u32;
        let height = size.1 as u32;
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.set_dirty();
        self.clear_buf();
    }

    fn clear_buf(&mut self) {
        let mut line = Vec::with_capacity(self.width as usize + 2);
        for _ in 0..self.width{
            line.push(b' ');
        }
        let last_line = line.clone();
        line.push(b'\r');
        line.push(b'\n');

        let mut lines = Vec::with_capacity(self.height as usize);
        for _ in 0..self.height {
            lines.push(line.clone());
        }

        lines.push(last_line);
        self.out_buf = lines;
    }

    fn clear_region(&mut self, rect: Rect) {
        for x in rect.left() .. rect.right() {
            for y in rect.top() .. rect.bottom() {
                self.draw_char(b' ', x, y);
            }
        }
        
    }

    fn draw_char(&mut self, val: u8, x: u32, y: u32) {
        if y <= self.height && x < self.width {
            self.out_buf[y as usize][x as usize] = val;
        }
    }

    pub fn draw(&mut self, input: Vec<Vec<u8>>, rect: Rect) {
        self.set_dirty();

        let x = rect.left();
        let mut y = rect.top();
        
        self.clear_region(rect);
        
        for line in input {
            let mut x = x;
            for c in line {
                if x < rect.right() {
                    self.draw_char(c, x, y);
                    x += 1;
                } else {
                    break;
                }
            }
            if y >= rect.bottom() { break; }
            y += 1;
        }
    }

    pub fn render(&mut self, stream: &mut TermStream) {
        self.init();
        if !self.is_dirty() { return; }

        let mut write_buf = Vec::new();
        for line in &self.out_buf {
            write_buf.append(&mut line.clone());
        }

        let _ = stream.write_all(&*format!("{}{}{}",
                                 cursor::Goto(1,1),
                                 color::Fg(color::White),
                                 String::from_utf8(write_buf).unwrap(),
                                ).into_bytes());
        
        let _ = stream.flush();
        self.dirty = false;
    }
}
