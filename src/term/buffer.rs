use termion::{color, cursor, terminal_size};
use term::TermStream;
use std::io::Write;

#[derive(Clone, Copy, Debug)]
pub struct Point(pub u32, pub u32);

impl Point {
    pub fn x(&self) -> u32 {
        self.0
    }

    pub fn y(&self) -> u32 {
        self.1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect(pub Point, pub u32, pub u32);

impl Rect {
    pub fn x(&self) -> u32 {
        self.0.x()
    }

    pub fn y(&self) -> u32 {
        self.0.y()
    }

    pub fn left(&self) -> u32 {
        self.0.x()
    }

    pub fn right(&self) -> u32 {
        self.0.x() + self.1
    }

    pub fn top(&self) -> u32 {
        self.0.y()
    }

    pub fn bottom(&self) -> u32 {
        self.0.y() + self.2
    }

    pub fn width(&self) -> u32 {
        self.1
    }

    pub fn height(&self) -> u32 {
        self.2
    }

    pub fn horizontal(&self) -> ::std::ops::Range<u32> {
        ::std::ops::Range { start: self.left(), end: self.right() }
    }

    pub fn vertical(&self) -> ::std::ops::Range<u32> {
        ::std::ops::Range { start: self.top(), end: self.bottom() }
    }
}

pub struct Surface {
    area: Rect,
    buf: Vec<u8>,
}

impl Surface {
    fn new(area: Rect) -> Surface {
        let mut surface = Surface {
            area: area,
            buf: Vec::new(),
        };

        let area = surface.area;
        for _ in 0..area.width() * area.height() {
           surface.buf.push(b' ');
        }

        surface
    }

    pub fn rect(&self) -> Rect {
        self.area
    }

    pub fn buf(&self) -> &[u8] {
        &*self.buf
    }

    pub fn clear(&mut self) {
        let area = self.area;
        self.clear_rect(area);
    }

    pub fn clear_rect(&mut self, rect: Rect) {
        for x in rect.horizontal() {
            for y in rect.vertical() {
                self.set_char(b' ', Point(x, y));
            }
        }
    }

    pub fn blit(&mut self, source: &Surface, dest: Point) {
        let x = dest.x();
        let y = dest.y();

        self.clear_rect(Rect(dest, source.rect().width(), source.rect().height()));

        if source.buf.len() == 0 { return };
        for x1 in source.rect().horizontal() {
            for y1 in source.rect().vertical() {
                let p = Point(x1, y1);
                self.set_char(source.get_char(p), Point(x1 + x, y1 + y));
            }
        }
    }

    fn set_char(&mut self, val: u8, p: Point) {
        let x = p.x() as usize;
        let y = p.y() as usize;
        if p.y() < self.area.height() && p.x() < self.area.width() {
            self.buf[(y * self.area.width() as usize ) + x] = val;
        }
    }

    fn get_char(&self, p: Point) -> u8 {
        let x = p.x() as usize;
        let y = p.y() as usize;
        let width = self.area.width() as usize;
        let ind = (y * width) + x;
        if p.y() < self.area.height() && p.x() < self.area.width() && ind < self.buf.len() {
            self.buf[(y * width) + x]
        } else {
            b' '
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

    pub fn height(&self) -> u32 { self.surface.rect().height() }
    pub fn width(&self) -> u32 { self.surface.rect().width() }

    pub fn rect(&self) -> Rect { self.surface.rect() }

    fn init(&mut self) {
        let size  = terminal_size().unwrap();
        let width = size.0 as u32;
        let height = size.1 as u32;
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

    pub fn draw(&mut self, input: Vec<u8>, rect: Rect) {
        let surf = Surface { buf: input, area: 
            Rect(Point(0, 0), rect.width(), rect.height())};
        self.blit(&surf, Point(rect.x(), rect.y()));
    }

    pub fn render(&mut self, stream: &mut TermStream) {
        self.init();
        if !self.is_dirty() { return; }

        let _ = stream.write_all(&*format!("{}{}{}",
                                 cursor::Goto(1,1),
                                 color::Fg(color::White),
                                 ::std::str::from_utf8(self.surface.buf()).unwrap(),
                                ).into_bytes());
        
        let _ = stream.flush();
        self.dirty = false;
    }
}
