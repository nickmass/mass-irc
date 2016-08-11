use std::str;

#[derive(Debug, PartialEq)]
enum Sender<'a> {
    User(&'a str, Option<&'a str>, Option<&'a str>),
    Server(&'a str)
}

#[derive(Debug, PartialEq)]
pub struct Tags<'a> {
    data: Vec<(&'a str, String)>,
}

#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    tags: Option<Tags<'a>>,
    prefix: Option<Sender<'a>>,
    command: &'a str,
    params: Vec<&'a str>,
}

pub struct CommandParser<'a> {
    raw: &'a Vec<u8>,
}

impl<'a> CommandParser<'a> {
    pub fn new(message: &'a Vec<u8>) -> CommandParser<'a> {
        CommandParser { raw: message }
    }

    pub fn parse(self) -> Command<'a> {
        fn unescape_tag_value(value: &str) -> String {
            let escape_seqs =
                vec![("\\\\", "\\"), ("\\:", ";"), ("\\s", " "), ("\\r", "\r"), ("\\n", "\n")];

            escape_seqs.iter().fold(value.into(), |a, x| a.replace(x.0, x.1))
        }

        fn host(c: char) -> bool {
            c == '.' || alphabetic(c) || c.is_digit(10)
        }

        fn nick_char(c: char) -> bool {
            alphabetic(c) || c.is_digit(10) || special(c)
        }

        fn special(c: char) -> bool {
            c == '-' || c =='[' || c == ']' || c == '\\' || c == '`' || c == '^' || c =='{' || c == '}'
        }

        fn alphabetic(c: char) -> bool {
            let u = c as u32;
            (u > 0x40 && u <= 0x5A) || (u > 0x60 && u <= 0x7A)
        }

        fn not_whitespace(c: char) -> bool {
            !whitespace(c)
        }

        fn whitespace(c: char) -> bool {
            c == ' ' || c == '\0' || c == '\r' || c == '\n'
        }

        fn user_char(c: char) -> bool {
            !whitespace(c) && c != '@'
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

        named!(user<&str, &str>, chain!(
                tag_s!("!") ~
                user: take_while1_s!(user_char),
                || user));

        named!(user_host<&str, &str>, chain!(
                tag_s!("@") ~
                host: take_while1_s!(host),
                || host));

        named!(nick<&str, &str>, chain!(
                tag_s!(":") ~
                nick: take_while1_s!(nick_char)~
                expr_opt!(
                    if alphabetic(nick.chars().next().unwrap()) {
                        Some(nick)
                    } else {
                        None
                    }),
                || nick));

        named!(user_string<&str, Sender>, chain!(
                                        nick: nick ~
                                        user: user? ~
                                        host: user_host? ~
                                        tag_s!(" "),
                                        || Sender::User(nick, user, host)));

        named!(server<&str, Sender>, chain!(
                                    tag_s!(":") ~
                                    host: take_while_s!(host) ~
                                    tag_s!(" "),
                                    || Sender::Server(host)));

        named!(prefix<&str, Sender >, chain!(
                       sender: alt!(user_string | server),
                       || sender ));

        named!(alpha<&str, &str>, take_while1_s!(alphabetic));

        named!(digits<&str, &str>, chain!(
                                    tri: take_s!(3) ~
                                    expr_opt!(
                                        if tri.chars().all(|c| c.is_digit(10)) {
                                            Some(tri)
                                        } else {
                                            None
                                        }),
                                    || tri ));

        named!(command<&str, &str>, chain! (
                                r: alt!(digits | alpha) ~
                                tag_s!(" "),
                                || r));

        named!(param<&str, &str >,
               chain!(
                   not!(tag_s!(":")) ~
                   param: is_not_s!(" \r\n\0") ~
                   tag_s!(" "),
                   || param));

        named!(params<&str, Vec<&str> >,
               chain!(
                   params: many0!(param) ~
                   trailing: preceded!(tag_s!(":"), is_not_s!("\r\n\0"))?,
                    || {
                            let mut params = params;
                            if trailing.is_some() {
                                params.push(trailing.unwrap())
                            }
                            params
                    }));

        named!(command_parser<&str, Command >, chain!(
                                        tags: tag_prefix_parser? ~
                                        prefix: prefix? ~
                                        command: command ~
                                        params: params ~
                                        tag_s!("\r\n"),
                                        || {
                                            Command { tags: tags, prefix: prefix, command: command, params: params }
                                        }));

        let r = command_parser(str::from_utf8(self.raw).unwrap());
        println!("{:?}", r);
        r.unwrap().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        /*
        let do_test = |i, o| {
            assert_eq!(CommandParser::new(&i).parse(),
            Command { tags: Some(Tags { data: o }) });
        };
/*
        let i = b"@abc=def;123=456:nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n".to_vec();
        let o = vec![("abc", "def".to_owned()), ("123", "456".to_owned())];
        do_test(i, o);
*/
        
        let i = b":nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n"
            .to_vec();
        assert_eq!(CommandParser::new(&i).parse(),
            Command { tags: None });
/*
        let i = b"@abc=def :nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n".to_vec();
        let o = vec![("abc", "def".to_owned())];
        do_test(i, o);

        let i = b"@abc=def;123=45\\:6 :nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n".to_vec();
        let o = vec![("abc", "def".to_owned()), ("123", "45;6".to_owned())];
        do_test(i, o);*/
 */
    }
}
