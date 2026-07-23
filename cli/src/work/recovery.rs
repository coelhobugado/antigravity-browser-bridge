use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureClass {
    TargetNotFound,
    PreconditionFailed,
    PostconditionFailed,
    Timeout,
    NetworkError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    Retry,
    Fallback,
    Abort,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    pub attempt_count: u32,
    pub next_action: RecoveryAction,
    pub reason: String,
}
