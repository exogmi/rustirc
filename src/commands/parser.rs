
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    Nick(String),
    User(String, String, String),
    Join(String),
    Part(String),
    PrivMsg(String, String),
    Quit(Option<String>),
    Ping(String),
    Pong(String),
    Mode(String, String, Option<String>),
    Topic(String, Option<String>),
    Names(String),
    List(Option<String>),
    Invite(String, String),
    Kick(String, String, Option<String>),
    Who(String),
    WhoisUser(String),
    WhoisServer(String),
    WhoisOperator(String),
    WhoisIdle(String),
    WhoisChannels(String),
    WhoisAuth(String),
    Whowas(String, Option<String>, Option<String>),
}

pub fn parse_command(input: &str) -> Option<Command> {
    let mut parts = input.splitn(2, ' ');
    let command = parts.next()?;
    let params = parts.next().unwrap_or("");

    match command.to_uppercase().as_str() {
        "NICK" => Some(Command::Nick(params.to_string())),
        "USER" => {
            let mut user_parts = params.splitn(4, ' ');
            Some(Command::User(
                user_parts.next()?.to_string(),
                user_parts.next()?.to_string(),
                user_parts.next()?.to_string(),
            ))
        }
        "JOIN" => Some(Command::Join(params.to_string())),
        "PART" => Some(Command::Part(params.to_string())),
        "PRIVMSG" => {
            let mut msg_parts = params.splitn(2, ':');
            let target = msg_parts.next()?.trim().to_string();
            let message = msg_parts.next()?.to_string();
            Some(Command::PrivMsg(target, message))
        }
        "QUIT" => Some(Command::Quit(if params.is_empty() { None } else { Some(params.to_string()) })),
        "PING" => Some(Command::Ping(params.to_string())),
        "PONG" => Some(Command::Pong(params.to_string())),
        "MODE" => {
            let mut mode_parts = params.splitn(3, ' ');
            Some(Command::Mode(
                mode_parts.next()?.to_string(),
                mode_parts.next()?.to_string(),
                mode_parts.next().map(|s| s.to_string()),
            ))
        }
        "TOPIC" => {
            let mut topic_parts = params.splitn(2, ':');
            let channel = topic_parts.next()?.trim().to_string();
            let topic = topic_parts.next().map(|s| s.to_string());
            Some(Command::Topic(channel, topic))
        }
        "NAMES" => Some(Command::Names(params.to_string())),
        "LIST" => Some(Command::List(if params.is_empty() { None } else { Some(params.to_string()) })),
        "INVITE" => {
            let mut invite_parts = params.splitn(2, ' ');
            Some(Command::Invite(
                invite_parts.next()?.to_string(),
                invite_parts.next()?.to_string(),
            ))
        }
        "KICK" => {
            let mut kick_parts = params.splitn(3, ' ');
            Some(Command::Kick(
                kick_parts.next()?.to_string(),
                kick_parts.next()?.to_string(),
                kick_parts.next().map(|s| s.to_string()),
            ))
        }
        "WHO" => Some(Command::Who(params.to_string())),
        "WHOIS" => Some(Command::WhoisUser(params.to_string())),
        "WHOWAS" => {
            let mut whowas_parts = params.splitn(3, ' ');
            Some(Command::Whowas(
                whowas_parts.next()?.to_string(),
                whowas_parts.next().map(|s| s.to_string()),
                whowas_parts.next().map(|s| s.to_string()),
            ))
        }
        _ => None,
    }
}
