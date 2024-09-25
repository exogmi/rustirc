
use std::sync::atomic::{AtomicUsize, Ordering};

static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn generate_client_id() -> usize {
    CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}
