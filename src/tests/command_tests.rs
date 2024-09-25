
use crate::commands::parser::{Command, parse_command};

#[test]
fn test_parse_nick_command() {
    assert_eq!(parse_command("NICK johndoe"), Some(Command::Nick("johndoe".to_string())));
}

#[test]
fn test_parse_user_command() {
    assert_eq!(
        parse_command("USER guest 0 * :John Doe"),
        Some(Command::User("guest".to_string(), "0".to_string(), "*".to_string()))
    );
}

#[test]
fn test_parse_join_command() {
    assert_eq!(parse_command("JOIN #rust"), Some(Command::Join("#rust".to_string())));
}

#[test]
fn test_parse_part_command() {
    assert_eq!(parse_command("PART #rust"), Some(Command::Part("#rust".to_string())));
}

#[test]
fn test_parse_privmsg_command() {
    assert_eq!(
        parse_command("PRIVMSG #rust :Hello, Rustaceans!"),
        Some(Command::PrivMsg("#rust".to_string(), "Hello, Rustaceans!".to_string()))
    );
}

#[test]
fn test_parse_quit_command() {
    assert_eq!(parse_command("QUIT"), Some(Command::Quit(None)));
    assert_eq!(
        parse_command("QUIT :Goodbye!"),
        Some(Command::Quit(Some("Goodbye!".to_string())))
    );
}

#[test]
fn test_parse_ping_command() {
    assert_eq!(parse_command("PING server1"), Some(Command::Ping("server1".to_string())));
}

#[test]
fn test_parse_pong_command() {
    assert_eq!(parse_command("PONG server1"), Some(Command::Pong("server1".to_string())));
}

#[test]
fn test_parse_mode_command() {
    assert_eq!(
        parse_command("MODE #channel +o user1"),
        Some(Command::Mode("#channel".to_string(), "+o".to_string(), Some("user1".to_string())))
    );
}

#[test]
fn test_parse_topic_command() {
    assert_eq!(
        parse_command("TOPIC #rust :Rust Programming Language"),
        Some(Command::Topic("#rust".to_string(), Some("Rust Programming Language".to_string())))
    );
    assert_eq!(
        parse_command("TOPIC #rust"),
        Some(Command::Topic("#rust".to_string(), None))
    );
}

#[test]
fn test_parse_names_command() {
    assert_eq!(parse_command("NAMES #rust"), Some(Command::Names("#rust".to_string())));
}

#[test]
fn test_parse_list_command() {
    assert_eq!(parse_command("LIST"), Some(Command::List(None)));
    assert_eq!(parse_command("LIST #rust"), Some(Command::List(Some("#rust".to_string()))));
}

#[test]
fn test_parse_invite_command() {
    assert_eq!(
        parse_command("INVITE user1 #rust"),
        Some(Command::Invite("user1".to_string(), "#rust".to_string()))
    );
}

#[test]
fn test_parse_kick_command() {
    assert_eq!(
        parse_command("KICK #rust user1"),
        Some(Command::Kick("#rust".to_string(), "user1".to_string(), None))
    );
    assert_eq!(
        parse_command("KICK #rust user1 :Reason for kick"),
        Some(Command::Kick("#rust".to_string(), "user1".to_string(), Some("Reason for kick".to_string())))
    );
}

#[test]
fn test_parse_who_command() {
    assert_eq!(parse_command("WHO #rust"), Some(Command::Who("#rust".to_string())));
}

#[test]
fn test_parse_whois_command() {
    assert_eq!(parse_command("WHOIS user1"), Some(Command::WhoisUser("user1".to_string())));
}

#[test]
fn test_parse_whowas_command() {
    assert_eq!(
        parse_command("WHOWAS user1"),
        Some(Command::Whowas("user1".to_string(), None, None))
    );
    assert_eq!(
        parse_command("WHOWAS user1 1 server1"),
        Some(Command::Whowas("user1".to_string(), Some("1".to_string()), Some("server1".to_string())))
    );
}

#[test]
fn test_parse_invalid_command() {
    assert_eq!(parse_command("INVALID_COMMAND"), None);
}
