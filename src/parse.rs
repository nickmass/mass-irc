use std::str;
use command_type::CommandType;

#[derive(Debug, PartialEq)]
enum Sender {
    User(String, Option<String>, Option<String>),
    Server(String)
}

impl Sender {
    fn to_cmd(&self) -> String {
        match *self {
            Sender::User(ref n, None, None) => format!(":{} ", n),
            Sender::User(ref n, Some(ref u), None) => format!(":{}!{} ", n, u),
            Sender::User(ref n, None, Some(ref h)) => format!(":{}@{} ", n, h),
            Sender::User(ref n, Some(ref u), Some(ref h)) => format!(":{}!{}@{} ", n, u, h),
            Sender::Server(ref s) => format!(":{} ", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Tag { key: String, value: String }

impl Tag {
    fn to_cmd(&self) -> String {
        fn escape_tag_value(value: &str) -> String {
            let escape_seqs =
                vec![("\\\\", "\\"), ("\\:", ";"), ("\\s", " "), ("\\r", "\r"), ("\\n", "\n")];

            escape_seqs.iter().fold(value.into(), |a, x| a.replace(x.1, x.0))
        }
        format!("{}={}", self.key, escape_tag_value(&*self.value))
    }
}

#[derive(Debug, PartialEq)]
pub struct Tags {
    data: Vec<Tag>,
}

impl Tags {
    fn to_cmd(&self) -> String {
        if self.data.len() == 0 {
            return "".to_string();
        }

        let mut iter = self.data.iter();
        let first = iter.next().unwrap().to_cmd();
        let mut buf = String::new();
        buf.push('@');
        buf.push_str(&*first);

        for i in iter {
            buf.push_str(&*format!(";{}", i.to_cmd()));
        }

        buf.push(' ');
        buf
    }
}

#[derive(Debug, PartialEq)]
pub struct Params {
    data: Vec<String>,
}

impl Params {
    fn to_cmd(&self) -> String {
        let mut buf = String::new();
        buf.push(' ');
        
        let n = self.data.len();

        if n > 0 {
            for i in 0..n-1 {
                buf.push_str(&*format!("{} ", self.data[i]));
            }
            buf.push_str(&*format!(":{}", self.data[n-1]));
        }
        buf
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    tags: Option<Tags>,
    prefix: Option<Sender>,
    command: CommandType,
    params: Params,
}

impl Command {
    pub fn to_cmd(&self) -> String {
        let cmd: &str = self.command.into();
        format!("{}{}{}{}\r\n", self.tags.as_ref().map(|x|x.to_cmd()).unwrap_or("".to_string()),
                            self.prefix.as_ref().map(|x|x.to_cmd()).unwrap_or("".to_string()),
                            cmd,
                            self.params.to_cmd())
    }
}

pub struct CommandParser {
    raw: Vec<u8>,
}

impl CommandParser {
    pub fn new(message: Vec<u8>) -> CommandParser {
        CommandParser { raw: message }
    }

    pub fn parse(self) -> Command {
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

        fn whitespace(c: char) -> bool {
            c == ' ' || c == '\0' || c == '\r' || c == '\n'
        }

        fn user_char(c: char) -> bool {
            !whitespace(c) && c != '@'
        }

        named!(tag_parser<&str, Tag>, chain!(key: is_not_s!("= ")   ~
                                      tag_s!("=")            ~
                                      value: is_not_s!("; ") ~
                                      tag_s!(";")? ,
                                      || {
                                          Tag {
                                              key: key.to_string(),
                                              value: unescape_tag_value(value)
                                      }}));

        named!(tags_parser<&str, Vec<Tag> >, chain!(r: many0!(tag_parser) ~
                                     tag_s!(" "), || { r }));

        named!(tag_prefix_parser<&str, Tags >, chain!(
                                       tag_s!("@")      ~
                                       tags: tags_parser,
                                       || {
                                            Tags {
                                                data: tags
                                            }
                                       }));

        named!(user<&str, String>, chain!(
                tag_s!("!") ~
                user: take_while1_s!(user_char),
                || user.to_string()));

        named!(user_host<&str, String>, chain!(
                tag_s!("@") ~
                host: take_while1_s!(host),
                || host.to_string()));

        named!(nick<&str, String>, chain!(
                tag_s!(":") ~
                nick: take_while1_s!(nick_char)~
                expr_opt!(
                    if alphabetic(nick.chars().next().unwrap()) {
                        Some(nick)
                    } else {
                        None
                    }),
                || nick.to_string()));

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
                                    || Sender::Server(host.to_string())));

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

        named!(command<&str, CommandType >, chain! (
                                r: alt!(digits | alpha) ~
                                tag_s!(" "),
                                || r.into()));

        named!(param<&str, String >,
               chain!(
                   not!(tag_s!(":")) ~
                   param: is_not_s!(" \r\n\0") ~
                   tag_s!(" "),
                   || param.to_string()));

        named!(params<&str, Params >,
               chain!(
                   params: many0!(param) ~
                   trailing: preceded!(tag_s!(":"), is_not_s!("\r\n\0"))?,
                    || {
                            let mut params = params;
                            if trailing.is_some() {
                                params.push(trailing.unwrap().to_string())
                            }
                            Params { data: params }
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

        let r = command_parser(str::from_utf8(&*self.raw).unwrap());
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
        let o = vec![("abc", "def".to_owned()), ("123", "356".to_owned())];
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
        let o = vec![("abc", "def".to_owned()), ("123", "35;6".to_owned())];
        do_test(i, o);*/
 */
    }
}
