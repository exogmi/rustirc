
use crate::commands::parser::Command;
use crate::models::user::User;
use crate::models::channel::Channel;
use crate::models::message::{Message, Recipient};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SharedState {
    pub users: Arc<Mutex<HashMap<usize, User>>>,
    pub channels: Arc<Mutex<HashMap<String, Channel>>>,
}

pub async fn handle_command(command: Command, client_id: usize, shared_state: &SharedState) -> Result<Vec<String>, String> {
    match command {
        Command::Nick(nickname) => handle_nick(client_id, nickname, shared_state),
        Command::User(username, _, realname) => handle_user(client_id, username, realname, shared_state),
        Command::Join(channel) => handle_join(client_id, channel, shared_state),
        Command::Part(channel) => handle_part(client_id, channel, shared_state),
        Command::PrivMsg(target, message) => handle_privmsg(client_id, target, message, shared_state),
        Command::Quit(message) => handle_quit(client_id, message, shared_state),
        Command::Ping(server) => handle_ping(server),
        Command::Pong(_) => handle_pong(client_id, shared_state),
        Command::Mode(_, _, _) => Err("MODE command not implemented yet".to_string()),
        Command::Topic(channel, topic) => handle_topic(client_id, channel, topic, shared_state),
        Command::Names(channel) => handle_names(channel, shared_state),
        Command::List(channel) => handle_list(channel, shared_state),
        Command::Invite(_, _) => Err("INVITE command not implemented yet".to_string()),
        Command::Kick(_, _, _) => Err("KICK command not implemented yet".to_string()),
        Command::Who(_) => Err("WHO command not implemented yet".to_string()),
        Command::WhoisUser(_) => Err("WHOIS command not implemented yet".to_string()),
        Command::WhoisServer(_) => Err("WHOIS command not implemented yet".to_string()),
        Command::WhoisOperator(_) => Err("WHOIS command not implemented yet".to_string()),
        Command::WhoisIdle(_) => Err("WHOIS command not implemented yet".to_string()),
        Command::WhoisChannels(_) => Err("WHOIS command not implemented yet".to_string()),
        Command::WhoisAuth(_) => Err("WHOIS command not implemented yet".to_string()),
        Command::Whowas(_, _, _) => Err("WHOWAS command not implemented yet".to_string()),
        Command::Cap(subcommand, param) => handle_cap(client_id, subcommand, param, shared_state),
    }
}

fn handle_nick(client_id: usize, nickname: String, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut users = shared_state.users.lock().unwrap();
    if let Some(user) = users.get_mut(&client_id) {
        let old_nick = user.nickname.clone().unwrap_or_else(|| "<unknown>".to_string());
        user.set_nickname(nickname.clone())?;
        Ok(vec![format!(":{} NICK :{}", old_nick, nickname)])
    } else {
        Err("User not found".to_string())
    }
}

fn handle_user(client_id: usize, username: String, realname: String, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut users = shared_state.users.lock().unwrap();
    if let Some(user) = users.get_mut(&client_id) {
        user.username = Some(username);
        user.realname = Some(realname);
        let nickname = user.nickname.clone().unwrap_or_else(|| format!("User{}", client_id));
        Ok(vec![
            format!("001 {} :Welcome to the IRC server!", nickname),
            format!("002 {} :Your host is rustirc2, running version 1.0", nickname),
            format!("003 {} :This server was created {}", nickname, chrono::Utc::now().format("%Y-%m-%d")),
            format!("004 {} rustirc2 1.0 o o", nickname),
            format!("005 {} CHANTYPES=# CHARSET=utf-8 :are supported by this server", nickname),
            format!("251 {} :There are {} users and 0 services on 1 server", nickname, users.len()),
        ])
    } else {
        Err("User not found".to_string())
    }
}

fn handle_join(client_id: usize, channel_name: String, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut channels = shared_state.channels.lock().unwrap();
    let mut users = shared_state.users.lock().unwrap();

    let channel = channels.entry(channel_name.clone()).or_insert_with(|| Channel::new(channel_name.clone()));
    channel.add_member(client_id);

    if let Some(user) = users.get_mut(&client_id) {
        user.join_channel(channel_name.clone());
        let nick = user.nickname.clone().unwrap_or_else(|| client_id.to_string());
        Ok(vec![
            format!(":{} JOIN :{}", nick, channel_name),
            format!(":{} 353 {} = {} :{}", "server", nick, channel_name, channel.members.iter().map(|&id| users.get(&id).and_then(|u| u.nickname.clone()).unwrap_or_else(|| id.to_string())).collect::<Vec<_>>().join(" ")),
            format!(":{} 366 {} {} :End of /NAMES list", "server", nick, channel_name),
        ])
    } else {
        Err("User not found".to_string())
    }
}

fn handle_part(client_id: usize, channel_name: String, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut channels = shared_state.channels.lock().unwrap();
    let mut users = shared_state.users.lock().unwrap();

    let user = users.get_mut(&client_id).ok_or_else(|| "User not found".to_string())?;
    let nick = user.nickname.clone().unwrap_or_else(|| client_id.to_string());

    if let Some(channel) = channels.get_mut(&channel_name) {
        channel.remove_member(&client_id);
        if channel.members.is_empty() {
            channels.remove(&channel_name);
        }
    } else {
        return Err(format!("Channel {} not found", channel_name));
    }

    user.leave_channel(&channel_name);
    Ok(vec![format!(":{} PART :{}", nick, channel_name)])
}

fn handle_privmsg(client_id: usize, target: String, message: String, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let users = shared_state.users.lock().unwrap();
    let channels = shared_state.channels.lock().unwrap();

    if let Some(sender) = users.get(&client_id) {
        let sender_nick = sender.nickname.clone().unwrap_or_else(|| client_id.to_string());
        let recipient = if target.starts_with('#') {
            if let Some(_) = channels.get(&target) {
                Recipient::Channel(target.clone())
            } else {
                return Err(format!("Channel {} not found", target));
            }
        } else {
            if let Some(target_user) = users.values().find(|u| u.nickname.as_ref() == Some(&target)) {
                Recipient::User(target_user.id)
            } else {
                return Err(format!("User {} not found", target));
            }
        };

        let _msg = Message::new(client_id, recipient.clone(), message.clone());

        match recipient {
            Recipient::Channel(channel_name) => {
                if let Some(channel) = channels.get(&channel_name) {
                    Ok(vec![format!(":{} PRIVMSG {} :{}", sender_nick, channel_name, message)])
                } else {
                    Err(format!("Channel {} not found", channel_name))
                }
            }
            Recipient::User(_target_id) => {
                Ok(vec![format!(":{} PRIVMSG {} :{}", sender_nick, target, message)])
            }
        }
    } else {
        Err("User not found".to_string())
    }
}

fn handle_quit(client_id: usize, message: Option<String>, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut users = shared_state.users.lock().unwrap();
    let mut channels = shared_state.channels.lock().unwrap();

    if let Some(user) = users.remove(&client_id) {
        let nick = user.nickname.unwrap_or_else(|| client_id.to_string());
        let quit_message = message.unwrap_or_else(|| "Client Quit".to_string());

        for channel_name in &user.channels {
            if let Some(channel) = channels.get_mut(channel_name) {
                channel.remove_member(&client_id);
            }
        }

        Ok(vec![format!(":{} QUIT :{}", nick, quit_message)])
    } else {
        Err("User not found".to_string())
    }
}

fn handle_ping(server: String) -> Result<Vec<String>, String> {
    Ok(vec![format!("PONG {}", server)])
}

fn handle_pong(client_id: usize, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut users = shared_state.users.lock().unwrap();
    if let Some(_user) = users.get_mut(&client_id) {
        // Update last activity timestamp
        // For now, we'll just return an OK message
        Ok(vec!["PONG received".to_string()])
    } else {
        Err("User not found".to_string())
    }
}

fn handle_topic(client_id: usize, channel_name: String, topic: Option<String>, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let mut channels = shared_state.channels.lock().unwrap();
    let users = shared_state.users.lock().unwrap();

    if let Some(channel) = channels.get_mut(&channel_name) {
        if let Some(user) = users.get(&client_id) {
            let nick = user.nickname.clone().unwrap_or_else(|| client_id.to_string());
            match topic {
                Some(new_topic) => {
                    channel.set_topic(new_topic.clone());
                    Ok(vec![format!(":{} TOPIC {} :{}", nick, channel_name, new_topic)])
                }
                None => {
                    match &channel.topic {
                        Some(current_topic) => Ok(vec![format!(":{} 332 {} {} :{}", "server", nick, channel_name, current_topic)]),
                        None => Ok(vec![format!(":{} 331 {} {} :No topic is set", "server", nick, channel_name)]),
                    }
                }
            }
        } else {
            Err("User not found".to_string())
        }
    } else {
        Err(format!("Channel {} not found", channel_name))
    }
}

fn handle_names(channel_name: String, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let channels = shared_state.channels.lock().unwrap();
    let users = shared_state.users.lock().unwrap();

    if let Some(channel) = channels.get(&channel_name) {
        let user_list = channel.members.iter()
            .filter_map(|&id| users.get(&id))
            .map(|user| user.nickname.clone().unwrap_or_else(|| user.id.to_string()))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(vec![
            format!(":{} 353 * = {} :{}", "server", channel_name, user_list),
            format!(":{} 366 * {} :End of /NAMES list", "server", channel_name),
        ])
    } else {
        Err(format!("Channel {} not found", channel_name))
    }
}

fn handle_list(channel: Option<String>, shared_state: &SharedState) -> Result<Vec<String>, String> {
    let channels = shared_state.channels.lock().unwrap();

    let mut response = vec![format!(":{} 321 Channel :Users Name", "server")];

    match channel {
        Some(channel_name) => {
            if let Some(channel) = channels.get(&channel_name) {
                response.push(format!(":{} 322 {} {} :{}", "server", channel_name, channel.members.len(), channel.topic.clone().unwrap_or_default()));
            } else {
                return Err(format!("Channel {} not found", channel_name));
            }
        }
        None => {
            let mut channel_list: Vec<_> = channels.iter().collect();
            channel_list.sort_by(|a, b| a.0.cmp(b.0));
            for (name, channel) in channel_list {
                response.push(format!(":{} 322 {} {} :{}", "server", name, channel.members.len(), channel.topic.clone().unwrap_or_default()));
            }
        }
    }

    response.push(format!(":{} 323 :End of /LIST", "server"));
    Ok(response)
}
fn handle_cap(client_id: usize, subcommand: String, param: Option<String>, shared_state: &SharedState) -> Result<Vec<String>, String> {
    match subcommand.as_str() {
        "LS" => Ok(vec!["CAP * LS :".to_string()]),
        "REQ" => Ok(vec!["CAP * ACK :".to_string()]),
        "END" => Ok(vec![]),
        _ => Err(format!("Unknown CAP subcommand: {}", subcommand)),
    }
}
