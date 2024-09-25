
use crate::utils::generate_client_id;
use std::sync::{Arc, Barrier};
use std::thread;
use std::net::IpAddr;
use std::str::FromStr;
use crate::models::user::{User, UserStatus};
use crate::models::channel::Channel;
use crate::models::message::{Message, Recipient};
use chrono::Utc;

#[test]
fn test_generate_client_id_uniqueness() {
    let id1 = generate_client_id();
    let id2 = generate_client_id();
    assert_ne!(id1, id2, "Generated IDs should be unique");
}

#[test]
fn test_generate_client_id_thread_safety() {
    let thread_count = 100;
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = vec![];

    for _ in 0..thread_count {
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            barrier.wait();
            generate_client_id()
        });
        handles.push(handle);
    }

    let ids: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let unique_ids: std::collections::HashSet<_> = ids.iter().cloned().collect();

    assert_eq!(ids.len(), unique_ids.len(), "All generated IDs should be unique across threads");
}

#[test]
fn test_user_nickname_validation() {
    let mut user = User::new(1, IpAddr::from_str("127.0.0.1").unwrap());
    
    assert!(user.set_nickname("valid_nick".to_string()).is_ok());
    assert!(user.set_nickname("".to_string()).is_err());
    assert!(user.set_nickname("too_long_nickname".to_string()).is_err());
    assert!(user.set_nickname("invalid!".to_string()).is_err());
}

#[test]
fn test_user_join_leave_channel() {
    let mut user = User::new(1, IpAddr::from_str("127.0.0.1").unwrap());
    
    user.join_channel("#test".to_string());
    assert!(user.channels.contains("#test"));
    
    user.leave_channel("#test");
    assert!(!user.channels.contains("#test"));
}

#[test]
fn test_user_status_changes() {
    let mut user = User::new(1, IpAddr::from_str("127.0.0.1").unwrap());
    
    assert_eq!(user.status, UserStatus::Online);
    
    user.set_away(Some("Gone fishing".to_string()));
    assert_eq!(user.status, UserStatus::Away(Some("Gone fishing".to_string())));
    
    user.set_online();
    assert_eq!(user.status, UserStatus::Online);
}

#[test]
fn test_channel_members() {
    let mut channel = Channel::new("#test".to_string());
    
    channel.add_member(1);
    assert!(channel.members.contains(&1));
    
    channel.remove_member(&1);
    assert!(!channel.members.contains(&1));
}

#[test]
fn test_channel_topic() {
    let mut channel = Channel::new("#test".to_string());
    
    channel.set_topic("Test Topic".to_string());
    assert_eq!(channel.topic, Some("Test Topic".to_string()));
}

#[test]
fn test_message_creation() {
    let user_message = Message::new(1, Recipient::User(2), "Hello".to_string());
    let channel_message = Message::new(1, Recipient::Channel("#test".to_string()), "Hello all".to_string());
    
    assert_eq!(user_message.sender_id, 1);
    assert_eq!(channel_message.sender_id, 1);
    
    match user_message.recipient {
        Recipient::User(id) => assert_eq!(id, 2),
        _ => panic!("Expected User recipient"),
    }
    
    match channel_message.recipient {
        Recipient::Channel(name) => assert_eq!(name, "#test"),
        _ => panic!("Expected Channel recipient"),
    }
    
    assert!(user_message.timestamp <= Utc::now());
    assert!(channel_message.timestamp <= Utc::now());
}
