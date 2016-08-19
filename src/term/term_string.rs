use term::buffer::{Color, Glyph};

#[derive(Debug, Clone)]
pub struct TermString {
    internal: Vec<Glyph>,
}

// "[{color:'Green';background:'Red';}NickMass{color:'Black';}]:
// Hello {color:'Yellow';}Person{color:'Black';} how are you?
impl TermString {
    pub fn new() -> TermString {
        TermString {
            internal: Vec::new(),
        }
    }

    pub fn from_str(string: &str) -> TermString {

        fn color_map(color: &str) -> Color {
            match color {
                "Black" => Color::Black,
                "Blue" => Color::Blue,
                "Cyan" => Color::Cyan,
                "Green" => Color::Green,
                "LightBlack" => Color::LightBlack,
                "LightBlue" => Color::LightBlue,
                "LightCyan" => Color::LightCyan,
                "LightGreen" => Color::LightGreen,
                "LightMagenta" => Color::LightMagenta,
                "LightRed" => Color::LightRed,
                "LightWhite" => Color::LightWhite,
                "LightYellow" => Color::LightYellow,
                "Magenta" => Color::Magenta,
                "Red" => Color::Red,
                "White" => Color::White,
                "Yellow" => Color::Yellow,
                _ => unimplemented!()
            }
        }

        enum ColorType {
            Background(Color),
            Foreground(Color)
        }

        named!(color <&str, Color >, map!(
                delimited!(tag_s!(":"), take_until_s!(";"), tag_s!(";")),
                color_map));

        named!(background<&str, ColorType >, map!(
                preceded!(tag_s!("background"), color), 
                |c| ColorType::Background(c) ));

        named!(foreground<&str, ColorType >, map!(
                preceded!(tag_s!("color"), color), 
                |c| ColorType::Foreground(c) ));

        named!(color_block<&str, (Option<Color>, Option<Color>) >, map!(
                delimited!(
                  tag_s!("\0"),
                    many1!(complete!(alt!(foreground | background))),
                   tag_s!("\0")),
                |colors| {
                    let mut fg = None;
                    let mut bg = None;
                    for ct in colors {
                        match ct {
                            ColorType::Background(c) => { bg = Some(c)},
                            ColorType::Foreground(c) => { fg = Some(c)},
                        }
                    }

                    (fg, bg)
                }));

        named!(color_span<&str, Vec<Glyph> >, chain!(
                color_tag: color_block? ~
                text: is_not_s!("\0"),
                || {
                    let mut fg_color = None;
                    let mut bg_color = None;
                    if color_tag.is_some() {
                        fg_color = color_tag.unwrap().0;
                        bg_color = color_tag.unwrap().1;
                    }
                    let span: Vec<Glyph> = text.chars().map(|x| {
                        Glyph(x, fg_color, bg_color)
                    }).collect();

                    span
                }
                ));

        use nom::{eof};
        named!(term_string<&str, TermString>, chain!(
                res: many0!(color_span),
                || {
                    let mut term_string = Vec::new();
                    for span in res {
                        term_string.append(&mut span.clone());
                    }
                    
                    TermString { internal: term_string }
                }));


        let r = term_string(string);

        r.unwrap().1
    } 
    
    pub fn len(&self) -> usize {
        self.internal.len()
    }

    pub fn get(&self, index: usize) -> Option<Glyph> {
        self.internal.get(index).map(|x| *x)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for c in &self.internal {
            result.push_str(&c.to_string());
        }

        result
    }

    pub fn push(&mut self, glyph: Glyph) {
        self.internal.push(glyph);
    }

    pub fn extend_from_term_string(&mut self, string: &TermString) {
        self.internal.extend_from_slice(string.vec());
    }

    pub fn push_term_string(&mut self, string: &mut TermString) {
        self.internal.append(string.vec_mut());
    }

    pub fn vec_mut(&mut self) -> &mut Vec<Glyph> {
        &mut self.internal
    }

    pub fn vec(&self) -> &Vec<Glyph> {
        &self.internal
    }
}

impl From<String> for TermString {
    fn from(string: String) -> TermString {
        TermString::from_str(&string)
    }
}
