use termion::{terminal_size};
use term::TermStream;
use std::io::Write;
use term::term_string::TermString;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black,
    Blue,
    Cyan,
    Green,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
    Magenta,
    Red,
    White,
    Yellow,
}

impl Color {
    pub fn fg_code(&self) -> u32 {
        match *self {
            Color::Black => 30,
            Color::Blue => 34,
            Color::Cyan => 36,
            Color::Green => 32,
            Color::LightBlack => 90,
            Color::LightBlue => 94,
            Color::LightCyan => 96,
            Color::LightGreen => 92,
            Color::LightMagenta => 95,
            Color::LightRed => 91,
            Color::LightWhite => 97,
            Color::LightYellow => 93,
            Color::Magenta => 35,
            Color::Red => 31,
            Color::White => 37,
            Color::Yellow => 93,
        }
    }
    
    pub fn bg_code(&self) -> u32 {
        self.fg_code() + 10
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Glyph(pub char, pub Option<Color>, pub Option<Color>);

impl Glyph {
    pub fn to_string(&self) -> String {
        let c = if (self.0 as u32) < 0x20 {
            '\x20'
        } else {
            self.0
        };
        


        if self.1.is_some() && self.2.is_some() {
            let fg = self.1.unwrap().fg_code();
            let bg = self.2.unwrap().bg_code();
            format!("\x1b[{};{}m{}",fg,bg,c)
        } else if self.1.is_some() {
            let fg = self.1.unwrap().fg_code();
            format!("\x1b[{}m{}",fg,c)
        } else if self.2.is_some() {
            let bg = self.2.unwrap().bg_code();
            format!("\x1b[{}m{}",bg,c)
        } else {
            let mut s = String::with_capacity(1);
            s.push(c);
            s
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point(pub i32, pub i32);

impl Point {
    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect(pub Point, pub i32, pub i32);

impl Rect {
    pub fn x(&self) -> i32 {
        self.0.x()
    }

    pub fn y(&self) -> i32 {
        self.0.y()
    }

    pub fn left(&self) -> i32 {
        self.0.x()
    }

    pub fn right(&self) -> i32 {
        self.0.x() + self.1
    }

    pub fn top(&self) -> i32 {
        self.0.y()
    }

    pub fn bottom(&self) -> i32 {
        self.0.y() + self.2
    }

    pub fn width(&self) -> i32 {
        self.1
    }

    pub fn height(&self) -> i32 {
        self.2
    }

    pub fn horizontal(&self) -> ::std::ops::Range<i32> {
        ::std::ops::Range { start: self.left(), end: self.right() }
    }

    pub fn vertical(&self) -> ::std::ops::Range<i32> {
        ::std::ops::Range { start: self.top(), end: self.bottom() }
    }
}

pub struct Surface {
    area: Rect,
    buf: Vec<Glyph>,
}

impl Surface {
    pub fn new(area: Rect) -> Surface {
        let mut surface = Surface {
            area: area,
            buf: Vec::new(),
        };

        let area = surface.area;
        for _ in 0..area.width() * area.height() {
           surface.buf.push(Glyph(' ', None, None));
        }

        surface
    }

    pub fn rect(&self) -> Rect {
        self.area
    }

    pub fn buf(&self) -> &[Glyph] {
        &*self.buf
    }

    pub fn set_color(&mut self, p: Point, fg: Option<Color>, bg: Option<Color>) {
        let x = p.x() as usize;
        let y = p.y() as usize;
        if p.y() < self.area.height() && p.x() < self.area.width() &&
           p.y() >= 0 && p.x() >= 0 {
            self.buf[(y * self.area.width() as usize ) + x] =
                Glyph(self.get_char(p), fg, bg);
        }
    }

    pub fn clear(&mut self) {
        let area = self.area;
        self.clear_rect(area);
    }

    pub fn clear_rect(&mut self, rect: Rect) {
        for x in rect.horizontal() {
            for y in rect.vertical() {
                self.set_char(' ', Point(x, y));
            }
        }
    }

    pub fn formatted_text(&mut self, text: TermString, dest: Point) {
        for i in 0..text.len() {
            let x = i as i32;
            if x + dest.x() >= self.rect().width() { break; }
            self.set_glyph(text.get(i).unwrap(), Point(x + dest.x(), dest.y()));
        }
    }

    pub fn text(&mut self, text: &str, dest: Point) {
        for i in 0..text.len() {
            let x = i as i32;
            if x + dest.x() >= self.rect().width() { break; }
            self.set_char(text.chars().nth(i).unwrap(), Point(x + dest.x(), dest.y()));
        }
    }

    pub fn blit(&mut self, source: &Surface, dest: Point) {
        let x = dest.x();
        let y = dest.y();

        if source.buf.len() == 0 { return };
        for x1 in source.rect().horizontal() {
            for y1 in source.rect().vertical() {
                let p = Point(x1, y1);
                self.set_glyph(source.get_glpyh(p), Point(x1 + x, y1 + y));
            }
        }
    }

    fn set_glyph(&mut self, val: Glyph, p: Point) {
        let x = p.x() as usize;
        let y = p.y() as usize;
        if p.y() < self.area.height() && p.x() < self.area.width() &&
           p.y() >= 0 && p.x() >= 0 {
            self.buf[(y * self.area.width() as usize ) + x] = val;
        }
    }

    fn get_glpyh(&self, p: Point) -> Glyph {
        let x = p.x() as usize;
        let y = p.y() as usize;
        let width = self.area.width() as usize;
        let ind = (y * width) + x;
        if p.y() < self.area.height() && p.x() < self.area.width() &&
           ind < self.buf.len() && p.y() >= 0 && p.x() >= 0 && ind >= 0 {
            self.buf[(y * width) + x]
        } else {
            Glyph(' ', None, None)
        }
    }
    fn set_char(&mut self, val: char, p: Point) {
        let x = p.x() as usize;
        let y = p.y() as usize;
        if p.y() < self.area.height() && p.x() < self.area.width() && 
           p.y() >= 0 && p.x() >= 0 {
            self.buf[(y * self.area.width() as usize ) + x] = Glyph(val, None, None);
        }
    }

    fn get_char(&self, p: Point) -> char {
        let x = p.x() as usize;
        let y = p.y() as usize;
        let width = self.area.width() as usize;
        let ind = (y * width) + x;
        if p.y() < self.area.height() && p.x() < self.area.width() &&
           ind < self.buf.len() && p.y() >= 0 && p.x() >= 0 && ind >= 0 {
            self.buf[(y * width) + x].0
        } else {
            ' '
        }
    }
}

pub struct TermBuffer {
    surface: Surface,
    dirty: bool,
}

impl TermBuffer {
    pub fn new() -> TermBuffer {
        let mut buf = TermBuffer {
            surface: Surface::new(Rect(Point(0, 0), 0, 0)),
            dirty: true,
        };

        buf.init();
        buf
    }

    pub fn set_dirty(&mut self) { self.dirty = true; }
    pub fn is_dirty(&self) -> bool { self.dirty }

    pub fn height(&self) -> i32 { self.surface.rect().height() }
    pub fn width(&self) -> i32 { self.surface.rect().width() }

    pub fn rect(&self) -> Rect { self.surface.rect() }

    fn init(&mut self) {
        let size  = terminal_size().unwrap();
        let width = size.0 as i32;
        let height = size.1 as i32;
        if width != self.width() || height != self.height() {
            self.set_dirty();
            self.surface = Surface::new(Rect(Point(0, 0), width, height));
        }
    }

    pub fn clear(&mut self) {
        self.set_dirty();
        self.surface.clear();
    }

    pub fn blit(&mut self, source: &Surface, dest: Point) {
        self.set_dirty();
        self.surface.blit(source, dest);
    }

    pub fn render(&mut self, stream: &mut TermStream) {
        if !self.is_dirty() { return; }

        let mut buf = String::new();

        for glyph in self.surface.buf() {
            buf.push_str(&*glyph.to_string());            
        }

        let _ = stream.write_all(&*format!("\x1b[H\x1b[37;40m{}",buf).into_bytes());
        
        let _ = stream.flush();
        self.dirty = false;
        self.init();
    }
}
