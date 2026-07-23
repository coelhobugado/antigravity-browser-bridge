use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentOrigin {
    SystemPolicy,
    UserInstruction,
    UntrustedWebContent,
    AgentGenerated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub origin: ContentOrigin,
    pub is_sanitized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretReference {
    pub key_id: String,
    pub description: String,
}

pub struct SecretManager;

impl SecretManager {
    pub fn get_secret(_ref: &SecretReference) -> Option<String> {
        // Obter segredo armazenado com segurança, sem expor em logs
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalReceipt {
    pub action_hash: String,
    pub user_id: String,
    pub approved_at: String,
    pub policy_version: String,
}
