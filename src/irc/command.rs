use super::{CommandType};

#[derive(Clone, Debug, PartialEq)]
pub enum Sender {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Tag { 
    pub key: String,
    pub value: String
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct Tags {
    pub data: Vec<Tag>,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub data: Vec<String>,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub tags: Option<Tags>,
    pub prefix: Option<Sender>,
    pub command: CommandType,
    pub params: Params,
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
