use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Click,
    Type,
    Navigate,
    Extract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDescriptor {
    pub element_id: Option<String>,
    pub selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretOrValue {
    pub is_secret: bool,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub description: String,
    pub is_met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskClass {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAction {
    pub id: String,
    pub idempotency_key: String,
    pub observation_id: String,
    pub action_type: ActionType,
    pub target: TargetDescriptor,
    pub inputs: Vec<SecretOrValue>,
    pub preconditions: Vec<Condition>,
    pub postconditions: Vec<Condition>,
    pub risk: RiskClass,
}
