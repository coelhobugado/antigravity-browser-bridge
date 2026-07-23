use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageState {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySnapshot {
    pub nodes: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRef {
    pub id: String,
    pub role: String,
    pub accessible_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualObservation {
    pub screenshot_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub changed_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleSummary {
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub active_requests: usize,
    pub failed_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSummary {
    pub active: usize,
    pub completed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogSummary {
    pub has_active_dialog: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSummary {
    pub modified_inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssessment {
    pub is_trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityAssessment {
    pub is_stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyAssessment {
    pub ambiguity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationPacket {
    pub schema_version: String,
    pub observation_id: String,
    pub timestamp: String,
    pub task_id: String,
    pub session_id: String,
    pub page: PageState,
    pub tabs: Vec<TabState>,
    pub accessibility: AccessibilitySnapshot,
    pub interactive_elements: Vec<ElementRef>,
    pub visual: VisualObservation,
    pub changes: StateDiff,
    pub console: ConsoleSummary,
    pub network: NetworkSummary,
    pub downloads: DownloadSummary,
    pub dialogs: DialogSummary,
    pub forms: FormSummary,
    pub trust: TrustAssessment,
    pub stability: StabilityAssessment,
    pub uncertainty: UncertaintyAssessment,
}
