#[macro_use]
extern crate nom;

mod parse;
use parse::*;

fn main() {
    let parser = CommandParser::new(Vec::new());
    parser.parse();
}
