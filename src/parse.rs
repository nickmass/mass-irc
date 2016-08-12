use std::str;


#[derive(Debug, PartialEq)]
enum Sender<'a> {
    User(&'a str, Option<&'a str>, Option<&'a str>),
    Server(&'a str)
}

impl<'a> Sender<'a> {
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
pub struct Tag<'a> { key: &'a str, value: String }

impl<'a> Tag<'a> {
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
pub struct Tags<'a> {
    data: Vec<Tag<'a>>,
}

impl<'a> Tags<'a> {
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
pub struct Params<'a> {
    data: Vec<&'a str>,
}

impl<'a> Params<'a> {
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
pub struct Command<'a> {
    tags: Option<Tags<'a>>,
    prefix: Option<Sender<'a>>,
    command: CommandType,
    params: Params<'a>,
}

impl<'a> Command<'a> {
    pub fn to_cmd(&self) -> String {
        let cmd: &str = self.command.into();
        format!("{}{}{}{}", self.tags.as_ref().map(|x|x.to_cmd()).unwrap_or("".to_string()),
                            self.prefix.as_ref().map(|x|x.to_cmd()).unwrap_or("".to_string()),
                            cmd,
                            self.params.to_cmd())
    }
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
                                              key: key,
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

        named!(command<&str, CommandType >, chain! (
                                r: alt!(digits | alpha) ~
                                tag_s!(" "),
                                || r.into()));

        named!(param<&str, &str >,
               chain!(
                   not!(tag_s!(":")) ~
                   param: is_not_s!(" \r\n\0") ~
                   tag_s!(" "),
                   || param));

        named!(params<&str, Params >,
               chain!(
                   params: many0!(param) ~
                   trailing: preceded!(tag_s!(":"), is_not_s!("\r\n\0"))?,
                    || {
                            let mut params = params;
                            if trailing.is_some() {
                                params.push(trailing.unwrap())
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

        let r = command_parser(str::from_utf8(self.raw).unwrap());
        println!("{:?}", r);
        r.unwrap().1
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum CommandType {
    Pass,
    Nick,
    User,
    Server,
    Oper,
    Quit,
    SQuit,
    Join,
    Part,
    Mode,
    Topic,
    Names,
    List,
    Invite,
    Kick,
    Version,
    Stats,
    Links,
    Time,
    Connect,
    Trace,
    Admin,
    Info,
    PrivMsg,
    Notice,
    Who,
    WhoIs,
    WhoWas,
    Kill,
    Ping,
    Pong,
    Error,
    Away,
    Rehash,
    Restart,
    Summon,
    Users,
    WAllOps,
    UserHost,
    IsOn,

    Err_NoSuchNick,
    Err_NoSuchServer,
    Err_NoSuchChannel,
    Err_CannotSendToChan,
    Err_TooManyChannels,
    Err_WasNoSuchNick,
    Err_TooManyTargets,
    Err_NoOrigin,
    Err_NoRecipient,
    Err_NoTextToSend,
    Err_NoTopLevel,
    Err_WildTopLevel,
    Err_UnknownCommand,
    Err_NoMOTD,
    Err_NoAdminInfo,
    Err_FileError,
    Err_NonNickmameGiven,
    Err_ErroneusNickname,
    Err_NicknameInUse,
    Err_NickCollision,
    Err_UserNotInChannel,
    Err_NotOnChannel,
    Err_UserOnChannel,
    Err_NoLogin,
    Err_SummonDisabled,
    Err_UsersDisabled,
    Err_NotRegistered,
    Err_NeedMoreParams,
    Err_AlreadyRegistered,
    Err_NoPermForHost,
    Err_PasswdMismatch,
    Err_YoureBannedCreep,
    Err_KeySet,
    Err_ChannelIsFull,
    Err_UnknownModel,
    Err_InviteOnlyChan,
    Err_BannedFromChan,
    Err_BadChannelKey,
    Err_NoPrivileges,
    Err_ChanOPrivsNeeded,
    Err_CantKillServer,
    Err_NoOperHost,
    Err_UModeUnknownFlag,
    Err_UsersDontMatch,

    Rpl_None,
    Rpl_UserHost,
    Rpl_IsOn,
    Rpl_Away,
    Rpl_UnAway,
    Rpl_NoAway,
    Rpl_WhoIsUser,
    Rpl_WhoIsServer,
    Rpl_WhoIsOperator,
    Rpl_WhoIsIdle,
    Rpl_EndOfWhoIs,
    Rpl_WhoIsChannels,
    Rpl_WhoWasUser,
    Rpl_EndOfWhoWas,
    Rpl_ListStart,
    Rpl_List,
    Rpl_ListEnd,
    Rpl_ChannelModeIs,
    Rpl_NoTopic,
    Rpl_Topic,
    Rpl_Inviting,
    Rpl_Summoning,
    Rpl_Version,
    Rpl_WhoReply,
    Rpl_EndOfWho,
    Rpl_NamReply,
    Rpl_EndOfNames,
    Rpl_Links,
    Rpl_EndOfLinks,
    Rpl_BanList,
    Rpl_EndOfBanList,
    Rpl_Info,
    Rpl_EndOfInfo,
    Rpl_MOTDStart,
    Rpl_MOTD,
    Rpl_EndOfMOTD,
    Rpl_YoureOper,
    Rpl_Rehashing,
    Rpl_Time,
    Rpl_UsersStart,
    Rpl_Users,
    Rpl_EndOfUsers,
    Rpl_NoUsers,
    Rpl_TraceLink,
    Rpl_TraceConnecting,
    Rpl_TraceHandShake,
    Rpl_TraceUnkown,
    Rpl_TraceOperator,
    Rpl_TraceUser,
    Rpl_TraceServer,
    Rpl_TraceNewType,
    Rpl_TaceLog,
    Rpl_StatsLinkInfo,
    Rpl_StatsCommands,
    Rpl_StatsCLine,
    Rpl_StatsNLine,
    Rpl_StatsILine,
    Rpl_StatsKLine,
    Rpl_StatsYLine,
    Rpl_EndOfStats,
    Rpl_StatsLLine,
    Rpl_StatsUptime,
    Rpl_StatsOLine,
    Rpl_StatsHLine,
    Rpl_UModeIs,
    Rpl_LUserClient,
    Rpl_LUserOp,
    Rpl_LUserUnkown,
    Rpl_LUserChannels,
    Rpl_LUserMe,
    Rpl_AdminMe,
    Rpl_AdminLoc1,
    Rpl_AdminLoc2,
    Rpl_AdminEmail,

    Unknown
}

impl<'a> From<CommandType> for &'a str {
    fn from(s: CommandType) -> Self {
        match s {
            CommandType::Pass => "PASS",
            CommandType::Nick => "NICK",
            CommandType::User => "USER",
            CommandType::Server => "SERVER",
            CommandType::Oper => "OPER",
            CommandType::Quit => "QUIT",
            CommandType::SQuit => "SQUIT",
            CommandType::Join => "JOIN",
            CommandType::Part => "PART",
            CommandType::Mode => "MODE",
            CommandType::Topic => "TOPIC",
            CommandType::Names => "NAMES",
            CommandType::List => "LIST",
            CommandType::Invite => "INVITE",
            CommandType::Kick => "KICK",
            CommandType::Version => "VERSION",
            CommandType::Stats => "STATS",
            CommandType::Links => "LINKS",
            CommandType::Time => "TIME",
            CommandType::Connect => "CONNECT",
            CommandType::Trace => "TRACE",
            CommandType::Admin => "ADMIN",
            CommandType::Info => "INFO",
            CommandType::PrivMsg => "PRIVMSG",
            CommandType::Notice => "NOTICE",
            CommandType::Who => "WHO",
            CommandType::WhoIs => "WHOIS",
            CommandType::WhoWas => "WHOWAS",
            CommandType::Kill => "KILL",
            CommandType::Ping => "PING",
            CommandType::Pong => "PONG",
            CommandType::Error => "ERROR",
            CommandType::Away => "AWAY",
            CommandType::Rehash => "REHASH",
            CommandType::Restart => "RESTART",
            CommandType::Summon => "SUMMON",
            CommandType::Users => "USERS",
            CommandType::WAllOps => "WALLOPS",
            CommandType::UserHost => "USERHOST",
            CommandType::IsOn => "ISON",

            CommandType::Err_NoSuchNick => "401",
            CommandType::Err_NoSuchServer => "402",
            CommandType::Err_NoSuchChannel => "403",
            CommandType::Err_CannotSendToChan => "404",
            CommandType::Err_TooManyChannels => "405",
            CommandType::Err_WasNoSuchNick => "406",
            CommandType::Err_TooManyTargets => "407",
            CommandType::Err_NoOrigin => "409",
            CommandType::Err_NoRecipient => "411",
            CommandType::Err_NoTextToSend => "412",
            CommandType::Err_NoTopLevel => "413",
            CommandType::Err_WildTopLevel => "414",
            CommandType::Err_UnknownCommand => "421",
            CommandType::Err_NoMOTD => "422",
            CommandType::Err_NoAdminInfo => "423",
            CommandType::Err_FileError => "424",
            CommandType::Err_NonNickmameGiven => "431",
            CommandType::Err_ErroneusNickname => "432",
            CommandType::Err_NicknameInUse => "433",
            CommandType::Err_NickCollision => "436",
            CommandType::Err_UserNotInChannel => "441",
            CommandType::Err_NotOnChannel => "442",
            CommandType::Err_UserOnChannel => "443",
            CommandType::Err_NoLogin => "444",
            CommandType::Err_SummonDisabled => "445",
            CommandType::Err_UsersDisabled => "446",
            CommandType::Err_NotRegistered => "451",
            CommandType::Err_NeedMoreParams => "461",
            CommandType::Err_AlreadyRegistered => "462",
            CommandType::Err_NoPermForHost => "463",
            CommandType::Err_PasswdMismatch => "464",
            CommandType::Err_YoureBannedCreep => "465",
            CommandType::Err_KeySet => "467",
            CommandType::Err_ChannelIsFull => "471",
            CommandType::Err_UnknownModel => "472",
            CommandType::Err_InviteOnlyChan => "473",
            CommandType::Err_BannedFromChan => "474",
            CommandType::Err_BadChannelKey => "475",
            CommandType::Err_NoPrivileges => "481",
            CommandType::Err_ChanOPrivsNeeded => "482",
            CommandType::Err_CantKillServer => "483",
            CommandType::Err_NoOperHost => "491",
            CommandType::Err_UModeUnknownFlag => "501",
            CommandType::Err_UsersDontMatch => "502",
 
            CommandType::Rpl_None => "300",
            CommandType::Rpl_UserHost => "302",
            CommandType::Rpl_IsOn => "303",
            CommandType::Rpl_Away => "301",
            CommandType::Rpl_UnAway => "305",
            CommandType::Rpl_NoAway => "306",
            CommandType::Rpl_WhoIsUser => "311",
            CommandType::Rpl_WhoIsServer => "312",
            CommandType::Rpl_WhoIsOperator => "313",
            CommandType::Rpl_WhoIsIdle => "317",
            CommandType::Rpl_EndOfWhoIs => "318",
            CommandType::Rpl_WhoIsChannels => "319",
            CommandType::Rpl_WhoWasUser => "314",
            CommandType::Rpl_EndOfWhoWas => "369",
            CommandType::Rpl_ListStart => "321",
            CommandType::Rpl_List => "322",
            CommandType::Rpl_ListEnd => "323",
            CommandType::Rpl_ChannelModeIs => "324",
            CommandType::Rpl_NoTopic => "331",
            CommandType::Rpl_Topic => "332",
            CommandType::Rpl_Inviting => "341",
            CommandType::Rpl_Summoning => "342",
            CommandType::Rpl_Version => "351",
            CommandType::Rpl_WhoReply => "352",
            CommandType::Rpl_EndOfWho => "315",
            CommandType::Rpl_NamReply => "353",
            CommandType::Rpl_EndOfNames => "366",
            CommandType::Rpl_Links => "364",
            CommandType::Rpl_EndOfLinks => "365",
            CommandType::Rpl_BanList => "367",
            CommandType::Rpl_EndOfBanList => "368",
            CommandType::Rpl_Info => "371",
            CommandType::Rpl_EndOfInfo => "374",
            CommandType::Rpl_MOTDStart => "375",
            CommandType::Rpl_MOTD => "372",
            CommandType::Rpl_EndOfMOTD => "376",
            CommandType::Rpl_YoureOper => "381",
            CommandType::Rpl_Rehashing => "382",
            CommandType::Rpl_Time => "391",
            CommandType::Rpl_UsersStart => "392",
            CommandType::Rpl_Users => "393",
            CommandType::Rpl_EndOfUsers => "394",
            CommandType::Rpl_NoUsers => "395",
            CommandType::Rpl_TraceLink => "200",
            CommandType::Rpl_TraceConnecting => "201",
            CommandType::Rpl_TraceHandShake => "202",
            CommandType::Rpl_TraceUnkown => "203",
            CommandType::Rpl_TraceOperator => "204",
            CommandType::Rpl_TraceUser => "205",
            CommandType::Rpl_TraceServer => "206",
            CommandType::Rpl_TraceNewType => "208",
            CommandType::Rpl_TaceLog => "261",
            CommandType::Rpl_StatsLinkInfo => "211",
            CommandType::Rpl_StatsCommands => "212",
            CommandType::Rpl_StatsCLine => "213",
            CommandType::Rpl_StatsNLine => "214",
            CommandType::Rpl_StatsILine => "215",
            CommandType::Rpl_StatsKLine => "216",
            CommandType::Rpl_StatsYLine => "218",
            CommandType::Rpl_EndOfStats => "219",
            CommandType::Rpl_StatsLLine => "241",
            CommandType::Rpl_StatsUptime => "242",
            CommandType::Rpl_StatsOLine => "243",
            CommandType::Rpl_StatsHLine => "244",
            CommandType::Rpl_UModeIs => "221",
            CommandType::Rpl_LUserClient => "251",
            CommandType::Rpl_LUserOp => "252",
            CommandType::Rpl_LUserUnkown => "253",
            CommandType::Rpl_LUserChannels => "254",
            CommandType::Rpl_LUserMe => "255",
            CommandType::Rpl_AdminMe => "256",
            CommandType::Rpl_AdminLoc1 => "257",
            CommandType::Rpl_AdminLoc2 => "258",
            CommandType::Rpl_AdminEmail => "259",
            _ => "ERROR",
        }
    }
}

impl<'a> From<&'a str> for CommandType {
    fn from(s: &'a str) -> Self {
        match s {
            "PASS" => CommandType::Pass,
            "NICK" => CommandType::Nick,
            "USER" => CommandType::User,
            "SERVER" => CommandType::Server,
            "OPER" => CommandType::Oper,
            "QUIT" => CommandType::Quit,
            "SQUIT" => CommandType::SQuit,
            "JOIN" => CommandType::Join,
            "PART" => CommandType::Part,
            "MODE" => CommandType::Mode,
            "TOPIC" => CommandType::Topic,
            "NAMES" => CommandType::Names,
            "LIST" => CommandType::List,
            "INVITE" => CommandType::Invite,
            "KICK" => CommandType::Kick,
            "VERSION" => CommandType::Version,
            "STATS" => CommandType::Stats,
            "LINKS" => CommandType::Links,
            "TIME" => CommandType::Time,
            "CONNECT" => CommandType::Connect,
            "TRACE" => CommandType::Trace,
            "ADMIN" => CommandType::Admin,
            "INFO" => CommandType::Info,
            "PRIVMSG" => CommandType::PrivMsg,
            "NOTICE" => CommandType::Notice,
            "WHO" => CommandType::Who,
            "WHOIS" => CommandType::WhoIs,
            "WHOWAS" => CommandType::WhoWas,
            "KILL" => CommandType::Kill,
            "PING" => CommandType::Ping,
            "PONG" => CommandType::Pong,
            "ERROR" => CommandType::Error,
            "AWAY" => CommandType::Away,
            "REHASH" => CommandType::Rehash,
            "RESTART" => CommandType::Restart,
            "SUMMON" => CommandType::Summon,
            "USERS" => CommandType::Users,
            "WALLOPS" => CommandType::WAllOps,
            "USERHOST" => CommandType::UserHost,
            "ISON" => CommandType::IsOn,

            "401" => CommandType::Err_NoSuchNick,
            "402" => CommandType::Err_NoSuchServer,
            "403" => CommandType::Err_NoSuchChannel,
            "404" => CommandType::Err_CannotSendToChan,
            "405" => CommandType::Err_TooManyChannels,
            "406" => CommandType::Err_WasNoSuchNick,
            "407" => CommandType::Err_TooManyTargets,
            "409" => CommandType::Err_NoOrigin,
            "411" => CommandType::Err_NoRecipient,
            "412" => CommandType::Err_NoTextToSend,
            "413" => CommandType::Err_NoTopLevel,
            "414" => CommandType::Err_WildTopLevel,
            "421" => CommandType::Err_UnknownCommand,
            "422" => CommandType::Err_NoMOTD,
            "423" => CommandType::Err_NoAdminInfo,
            "424" => CommandType::Err_FileError,
            "431" => CommandType::Err_NonNickmameGiven,
            "432" => CommandType::Err_ErroneusNickname,
            "433" => CommandType::Err_NicknameInUse,
            "436" => CommandType::Err_NickCollision,
            "441" => CommandType::Err_UserNotInChannel,
            "442" => CommandType::Err_NotOnChannel,
            "443" => CommandType::Err_UserOnChannel,
            "444" => CommandType::Err_NoLogin,
            "445" => CommandType::Err_SummonDisabled,
            "446" => CommandType::Err_UsersDisabled,
            "451" => CommandType::Err_NotRegistered,
            "461" => CommandType::Err_NeedMoreParams,
            "462" => CommandType::Err_AlreadyRegistered,
            "463" => CommandType::Err_NoPermForHost,
            "464" => CommandType::Err_PasswdMismatch,
            "465" => CommandType::Err_YoureBannedCreep,
            "467" => CommandType::Err_KeySet,
            "471" => CommandType::Err_ChannelIsFull,
            "472" => CommandType::Err_UnknownModel,
            "473" => CommandType::Err_InviteOnlyChan,
            "474" => CommandType::Err_BannedFromChan,
            "475" => CommandType::Err_BadChannelKey,
            "481" => CommandType::Err_NoPrivileges,
            "482" => CommandType::Err_ChanOPrivsNeeded,
            "483" => CommandType::Err_CantKillServer,
            "491" => CommandType::Err_NoOperHost,
            "501" => CommandType::Err_UModeUnknownFlag,
            "502" => CommandType::Err_UsersDontMatch,
 
            "300" => CommandType::Rpl_None,
            "302" => CommandType::Rpl_UserHost,
            "303" => CommandType::Rpl_IsOn,
            "301" => CommandType::Rpl_Away,
            "305" => CommandType::Rpl_UnAway,
            "306" => CommandType::Rpl_NoAway,
            "311" => CommandType::Rpl_WhoIsUser,
            "312" => CommandType::Rpl_WhoIsServer,
            "313" => CommandType::Rpl_WhoIsOperator,
            "317" => CommandType::Rpl_WhoIsIdle,
            "318" => CommandType::Rpl_EndOfWhoIs,
            "319" => CommandType::Rpl_WhoIsChannels,
            "314" => CommandType::Rpl_WhoWasUser,
            "369" => CommandType::Rpl_EndOfWhoWas,
            "321" => CommandType::Rpl_ListStart,
            "322" => CommandType::Rpl_List,
            "323" => CommandType::Rpl_ListEnd,
            "324" => CommandType::Rpl_ChannelModeIs,
            "331" => CommandType::Rpl_NoTopic,
            "332" => CommandType::Rpl_Topic,
            "341" => CommandType::Rpl_Inviting,
            "342" => CommandType::Rpl_Summoning,
            "351" => CommandType::Rpl_Version,
            "352" => CommandType::Rpl_WhoReply,
            "315" => CommandType::Rpl_EndOfWho,
            "353" => CommandType::Rpl_NamReply,
            "366" => CommandType::Rpl_EndOfNames,
            "364" => CommandType::Rpl_Links,
            "365" => CommandType::Rpl_EndOfLinks,
            "367" => CommandType::Rpl_BanList,
            "368" => CommandType::Rpl_EndOfBanList,
            "371" => CommandType::Rpl_Info,
            "374" => CommandType::Rpl_EndOfInfo,
            "375" => CommandType::Rpl_MOTDStart,
            "372" => CommandType::Rpl_MOTD,
            "376" => CommandType::Rpl_EndOfMOTD,
            "381" => CommandType::Rpl_YoureOper,
            "382" => CommandType::Rpl_Rehashing,
            "391" => CommandType::Rpl_Time,
            "392" => CommandType::Rpl_UsersStart,
            "393" => CommandType::Rpl_Users,
            "394" => CommandType::Rpl_EndOfUsers,
            "395" => CommandType::Rpl_NoUsers,
            "200" => CommandType::Rpl_TraceLink,
            "201" => CommandType::Rpl_TraceConnecting,
            "202" => CommandType::Rpl_TraceHandShake,
            "203" => CommandType::Rpl_TraceUnkown,
            "204" => CommandType::Rpl_TraceOperator,
            "205" => CommandType::Rpl_TraceUser,
            "206" => CommandType::Rpl_TraceServer,
            "208" => CommandType::Rpl_TraceNewType,
            "261" => CommandType::Rpl_TaceLog,
            "211" => CommandType::Rpl_StatsLinkInfo,
            "212" => CommandType::Rpl_StatsCommands,
            "213" => CommandType::Rpl_StatsCLine,
            "214" => CommandType::Rpl_StatsNLine,
            "215" => CommandType::Rpl_StatsILine,
            "216" => CommandType::Rpl_StatsKLine,
            "218" => CommandType::Rpl_StatsYLine,
            "219" => CommandType::Rpl_EndOfStats,
            "241" => CommandType::Rpl_StatsLLine,
            "242" => CommandType::Rpl_StatsUptime,
            "243" => CommandType::Rpl_StatsOLine,
            "244" => CommandType::Rpl_StatsHLine,
            "221" => CommandType::Rpl_UModeIs,
            "251" => CommandType::Rpl_LUserClient,
            "252" => CommandType::Rpl_LUserOp,
            "253" => CommandType::Rpl_LUserUnkown,
            "254" => CommandType::Rpl_LUserChannels,
            "255" => CommandType::Rpl_LUserMe,
            "256" => CommandType::Rpl_AdminMe,
            "257" => CommandType::Rpl_AdminLoc1,
            "258" => CommandType::Rpl_AdminLoc2,
            "259" => CommandType::Rpl_AdminEmail,

            _ => CommandType::Unknown
        }
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
