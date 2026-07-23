use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopAction {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: String },
    KeyPress { key: String },
    TypeString { text: String },
    LaunchApp { app_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopObservation {
    pub active_window_title: String,
    pub screenshot_hash: Option<String>,
    pub os_name: String,
}

pub struct DesktopProvider;

impl DesktopProvider {
    pub fn execute_action(_action: &DesktopAction) -> Result<DesktopObservation, String> {
        // Implement OS-level bindings (e.g. OSWorld, PyAutoGUI equivalents in Rust)
        Err("Desktop automation not yet implemented natively".to_string())
    }
}
