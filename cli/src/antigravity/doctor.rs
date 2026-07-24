use serde_json::Value;
use std::fs;

pub fn check_installation() -> Result<String, String> {
    let path = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("agent-browser")
        .join("native-messaging")
        .join("host-manifest.json");
    let bytes = fs::read(&path)
        .map_err(|_| format!("native host manifest was not found at {}", path.display()))?;
    let manifest: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("native host manifest is invalid: {error}"))?;
    let executable = manifest
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "native host manifest has no executable path".to_string())?;
    if !std::path::Path::new(executable).is_file() {
        return Err(format!(
            "native host executable does not exist: {executable}"
        ));
    }
    Ok(format!(
        "Antigravity native host is installed at {}",
        path.display()
    ))
}
