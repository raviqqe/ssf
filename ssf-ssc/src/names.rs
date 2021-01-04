use std::sync::atomic::{AtomicU64, Ordering};

static name_id: AtomicU64 = AtomicU64::new(0);

pub fn generate_name() -> String {
    format!("x{}", name_id.fetch_add(1, Ordering::SeqCst))
}
