use std::sync::{Arc, Mutex};

/// Backing type for the `buffer` resource: host-resident bytes.
pub struct Buffer(pub Vec<u8>);

/// State of a host-side async task, shared with its background worker.
pub enum TaskState {
    Pending,
    Complete(Vec<u8>),
    Failed(String),
}

/// Backing type for the `task` resource. The worker thread holds its own clone
/// of `state` and writes the outcome; `task.result` reads it non-blocking.
pub struct Task {
    pub state: Arc<Mutex<TaskState>>,
}
