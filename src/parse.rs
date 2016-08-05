#[derive(Debug)]
pub struct Tags<'a> {
    data: Vec<(&'a str, String)>,
}

pub struct Command {
}

pub struct CommandParser {
    raw: Vec<u8>,
}

impl CommandParser {
    pub fn new(message: Vec<u8>) -> CommandParser {
        CommandParser {
            raw: message,
        }
    }

    pub fn parse(self) {
        fn unescape_tag_value(value: &str) -> String {
            let escape_seqs = vec![("\\\\", "\\"),
                                   ("\\:", ";"),
                                   ("\\s", " "),
                                   ("\\r", "\r"),
                                   ("\\n", "\n")];

            escape_seqs.iter().fold(value.into(), |a, x| {
                a.replace(x.0, x.1)
            })
        }

        named!(tag_parser<&str, (&str, String)>, chain!(key: is_not_s!("= ")   ~
                                                      tag_s!("=")            ~
                                                      value: is_not_s!("; ") ~
                                                      tag_s!(";")? , 
                                                      || {(key, unescape_tag_value(value))}));

        named!(tags_parser<&str, Vec<(&str, String)> >, chain!(r: many0!(tag_parser) ~
                                                             tag_s!(" "), || { r }));

        named!(tag_prefix_parser<&str, Tags >, chain!(
                                       tag_s!("@")      ~
                                       tags: tags_parser, 
                                       || {
                                            Tags {
                                                data: tags
                                            }
                                       }));


        println!("Out: {:?}", tag_prefix_parser("@abc=def;aa=123\\sasd\\n\\\\\\s\\:;asdf=456 :nick!"));
        println!("Out: {:?}", tag_prefix_parser("@abc=def :nick!"));
        println!("Out: {:?}", tag_prefix_parser(":nick!"));

        unimplemented!();
    }
}
