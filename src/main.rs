#[macro_use]
extern crate nom;

mod parse;
use parse::*;

fn main() {
    let i = b"@asdad=asdad;123123=qqqq :nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n".to_vec();
    let parser = CommandParser::new(&i);
    parser.parse();
}
