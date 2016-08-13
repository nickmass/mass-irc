use std::str;
use super::{CommandType};
use super::command::*;

pub struct CommandParser {
    raw: Vec<u8>,
}

impl CommandParser {
    pub fn new(message: Vec<u8>) -> CommandParser {
        println!("Parsing `{:?}`", String::from_utf8(message.clone()));
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
