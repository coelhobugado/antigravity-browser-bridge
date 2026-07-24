//! Thin MCP adapter for the Antigravity work application layer.
//!
//! This module intentionally contains no business logic and no bridge
//! transport code. All work state, persistence and cancellation live in
//! [`work_service`].

use serde_json::Value;

use super::work_service::{call_global, response_from, WorkOperation};

fn call(operation: WorkOperation, args: &Value) -> Value {
    response_from(call_global(operation, args.clone()))
}

pub fn work_session_start(args: &Value) -> Value {
    call(WorkOperation::Start, args)
}

pub fn work_observe(args: &Value) -> Value {
    call(WorkOperation::Observe, args)
}

pub fn work_execute(args: &Value) -> Value {
    call(WorkOperation::Execute, args)
}

pub fn work_verify(args: &Value) -> Value {
    call(WorkOperation::Verify, args)
}

pub fn work_checkpoint(args: &Value) -> Value {
    call(WorkOperation::Checkpoint, args)
}

pub fn work_resume(args: &Value) -> Value {
    call(WorkOperation::Resume, args)
}

pub fn work_export(args: &Value) -> Value {
    call(WorkOperation::Export, args)
}

pub fn work_request_approval(args: &Value) -> Value {
    call(WorkOperation::RequestApproval, args)
}

pub fn work_status(args: &Value) -> Value {
    call(WorkOperation::Status, args)
}

pub fn work_cancel(args: &Value) -> Value {
    call(WorkOperation::Cancel, args)
}

pub fn work_journal(args: &Value) -> Value {
    call(WorkOperation::Journal, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_is_structured_and_does_not_claim_unimplemented_success() {
        let result = work_execute(&serde_json::json!({"workId": "work-test", "action": "publish"}));
        assert_eq!(result["isError"], true);
        assert_ne!(
            result["structuredContent"]["error"]["code"],
            "not_implemented"
        );
    }
}
