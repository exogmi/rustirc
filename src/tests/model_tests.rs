
use crate::utils::generate_client_id;
use std::sync::{Arc, Barrier};
use std::thread;

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
