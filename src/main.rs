#[macro_use]
extern crate nom;

mod parse;
use parse::*;

fn main() {
    let i = b"@asdad=asdad;123123=qqqq :nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n".to_vec();
    println!("Input: {:?}", i);
    let parser = CommandParser::new(&i);
    let r = parser.parse();

    println!("Output: {:?}", r);
    println!("Output: {:?}", r.to_cmd())
}
