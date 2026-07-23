use serde_json::{json, Value};
use crate::work::observation::ObservationPacket;
use crate::work::action::WorkAction;

pub fn work_session_start(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Work session started stub" }] })
}
pub fn work_observe(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "ObservationPacket stub" }] })
}
pub fn work_execute(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "WorkAction executed stub" }] })
}
pub fn work_verify(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Work verified stub" }] })
}
pub fn work_checkpoint(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Checkpoint saved stub" }] })
}
pub fn work_resume(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Resumed from checkpoint stub" }] })
}
pub fn work_export(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Exported artifacts stub" }] })
}
pub fn work_request_approval(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Approval requested stub" }] })
}
pub fn work_status(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Work status stub" }] })
}
pub fn work_cancel(_args: &Value) -> Value {
    json!({ "isError": false, "content": [{ "type": "text", "text": "Work canceled stub" }] })
}
