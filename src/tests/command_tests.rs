
use crate::commands::parser::Command;
use crate::commands::handler::{handle_command, SharedState};
use crate::models::user::User;
use crate::models::channel::Channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Existing tests...

#[tokio::test]
async fn test_handle_nick_command() {
    let mut users = HashMap::new();
    users.insert(1, User::new(1, "127.0.0.1".parse().unwrap()));
    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let command = Command::Nick("newname".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":<unknown> NICK :newname"]);

    let users = shared_state.users.lock().unwrap();
    assert_eq!(users.get(&1).unwrap().nickname, Some("newname".to_string()));
}

#[tokio::test]
async fn test_handle_user_command() {
    let mut users = HashMap::new();
    let mut user = User::new(1, "127.0.0.1".parse().unwrap());
    user.set_nickname("User1".to_string()).unwrap();
    users.insert(1, user);
    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let command = Command::User("username".to_string(), "0".to_string(), "realname".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages.len(), 6);
    assert!(messages[0].contains("001 User1 :Welcome to the IRC server!"));
    assert!(messages[1].contains("002 User1 :Your host is rustirc2"));
    assert!(messages[2].contains("003 User1 :This server was created"));
    assert!(messages[3].contains("004 User1 rustirc2 1.0 o o"));
    assert!(messages[4].contains("005 User1 CHANTYPES=# CHARSET=utf-8"));
    assert!(messages[5].contains("251 User1 :There are 1 users and 0 services on 1 server"));

    let users = shared_state.users.lock().unwrap();
    let user = users.get(&1).unwrap();
    assert_eq!(user.username, Some("username".to_string()));
    assert_eq!(user.realname, Some("realname".to_string()));
}

#[tokio::test]
async fn test_handle_join_command() {
    let mut users = HashMap::new();
    let mut user = User::new(1, "127.0.0.1".parse().unwrap());
    user.set_nickname("testuser".to_string()).unwrap();
    users.insert(1, user);
    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let command = Command::Join("#testchannel".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![
        ":testuser JOIN :#testchannel",
        ":server 353 testuser = #testchannel :testuser",
        ":server 366 testuser #testchannel :End of /NAMES list",
    ]);

    let channels = shared_state.channels.lock().unwrap();
    assert!(channels.contains_key("#testchannel"));
    assert!(channels.get("#testchannel").unwrap().members.contains(&1));
}

#[tokio::test]
async fn test_handle_part_command() {
    let mut users = HashMap::new();
    let mut user = User::new(1, "127.0.0.1".parse().unwrap());
    user.set_nickname("testuser".to_string()).unwrap();
    user.join_channel("#testchannel".to_string());
    users.insert(1, user);

    let mut channels = HashMap::new();
    let mut channel = Channel::new("#testchannel".to_string());
    channel.add_member(1);
    channels.insert("#testchannel".to_string(), channel);

    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(channels)),
    };

    let command = Command::Part("#testchannel".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":testuser PART :#testchannel"]);

    let channels = shared_state.channels.lock().unwrap();
    let channel = channels.get("#testchannel");
    assert!(channel.is_none() || !channel.unwrap().members.contains(&1));
}

#[tokio::test]
async fn test_handle_privmsg_command() {
    let mut users = HashMap::new();
    let mut user1 = User::new(1, "127.0.0.1".parse().unwrap());
    user1.set_nickname("user1".to_string()).unwrap();
    users.insert(1, user1);
    let mut user2 = User::new(2, "127.0.0.1".parse().unwrap());
    user2.set_nickname("user2".to_string()).unwrap();
    users.insert(2, user2);

    let mut channels = HashMap::new();
    let mut channel = Channel::new("#testchannel".to_string());
    channel.add_member(1);
    channel.add_member(2);
    channels.insert("#testchannel".to_string(), channel);

    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(channels)),
    };

    // Test private message
    let command = Command::PrivMsg("user2".to_string(), "Hello, user2!".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":user1 PRIVMSG user2 :Hello, user2!"]);

    // Test channel message
    let command = Command::PrivMsg("#testchannel".to_string(), "Hello, channel!".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":user1 PRIVMSG #testchannel :Hello, channel!"]);
}

#[tokio::test]
async fn test_handle_quit_command() {
    let mut users = HashMap::new();
    let mut user = User::new(1, "127.0.0.1".parse().unwrap());
    user.set_nickname("testuser".to_string()).unwrap();
    user.join_channel("#testchannel".to_string());
    users.insert(1, user);

    let mut channels = HashMap::new();
    let mut channel = Channel::new("#testchannel".to_string());
    channel.add_member(1);
    channels.insert("#testchannel".to_string(), channel);

    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(channels)),
    };

    let command = Command::Quit(Some("Goodbye!".to_string()));
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":testuser QUIT :Goodbye!"]);

    let users = shared_state.users.lock().unwrap();
    assert!(!users.contains_key(&1));

    let channels = shared_state.channels.lock().unwrap();
    assert!(!channels.get("#testchannel").unwrap().members.contains(&1));
}

#[tokio::test]
async fn test_handle_ping_command() {
    let shared_state = SharedState {
        users: Arc::new(Mutex::new(HashMap::new())),
        channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let command = Command::Ping("server1".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec!["PONG server1"]);
}

#[tokio::test]
async fn test_handle_topic_command() {
    let mut users = HashMap::new();
    let mut user = User::new(1, "127.0.0.1".parse().unwrap());
    user.set_nickname("testuser".to_string()).unwrap();
    users.insert(1, user);

    let mut channels = HashMap::new();
    let channel = Channel::new("#testchannel".to_string());
    channels.insert("#testchannel".to_string(), channel);

    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(channels)),
    };

    // Set topic
    let command = Command::Topic("#testchannel".to_string(), Some("New topic".to_string()));
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":testuser TOPIC #testchannel :New topic"]);

    // Get topic
    let command = Command::Topic("#testchannel".to_string(), None);
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![":server 332 testuser #testchannel :New topic"]);
}

#[tokio::test]
async fn test_handle_names_command() {
    let mut users = HashMap::new();
    let mut user1 = User::new(1, "127.0.0.1".parse().unwrap());
    user1.set_nickname("user1".to_string()).unwrap();
    users.insert(1, user1);
    let mut user2 = User::new(2, "127.0.0.1".parse().unwrap());
    user2.set_nickname("user2".to_string()).unwrap();
    users.insert(2, user2);

    let mut channels = HashMap::new();
    let mut channel = Channel::new("#testchannel".to_string());
    channel.add_member(1);
    channel.add_member(2);
    channels.insert("#testchannel".to_string(), channel);

    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(channels)),
    };

    let command = Command::Names("#testchannel".to_string());
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages.len(), 2);
    assert!(messages[0].starts_with(":server 353 * = #testchannel :"));
    assert!(messages[0].contains("user1"));
    assert!(messages[0].contains("user2"));
    assert_eq!(messages[1], ":server 366 * #testchannel :End of /NAMES list");
}

#[tokio::test]
async fn test_handle_list_command() {
    let users = HashMap::new();

    let mut channels = HashMap::new();
    let mut channel1 = Channel::new("#channel1".to_string());
    channel1.set_topic("Topic 1".to_string());
    channel1.add_member(1);
    channels.insert("#channel1".to_string(), channel1);

    let mut channel2 = Channel::new("#channel2".to_string());
    channel2.set_topic("Topic 2".to_string());
    channel2.add_member(1);
    channel2.add_member(2);
    channels.insert("#channel2".to_string(), channel2);

    let shared_state = SharedState {
        users: Arc::new(Mutex::new(users)),
        channels: Arc::new(Mutex::new(channels)),
    };

    // List all channels
    let command = Command::List(None);
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![
        ":server 321 Channel :Users Name",
        ":server 322 #channel1 1 :Topic 1",
        ":server 322 #channel2 2 :Topic 2",
        ":server 323 :End of /LIST",
    ]);

    // List specific channel
    let command = Command::List(Some("#channel1".to_string()));
    let result = handle_command(command, 1, &shared_state).await;
    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages, vec![
        ":server 321 Channel :Users Name",
        ":server 322 #channel1 1 :Topic 1",
        ":server 323 :End of /LIST",
    ]);
}
