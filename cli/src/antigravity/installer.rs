//! Installs the current executable as Chrome's Native Messaging host. The
//! generated manifest contains the absolute executable path required by Chrome.

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

const HOST_NAME: &str = "com.antigravity.agent_browser";
const OFFICIAL_EXTENSION_ID: &str = "menkdnglfaljkgofohmhpblgiaehdibc";
const LEGACY_DEVELOPMENT_EXTENSION_ID: &str = "gaenafhipmoehmnockpmmgjhgbkhodhg";

fn integration_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("agent-browser")
        .join("native-messaging")
}

fn extension_ids() -> Vec<String> {
    let mut ids = vec![
        OFFICIAL_EXTENSION_ID.to_string(),
        LEGACY_DEVELOPMENT_EXTENSION_ID.to_string(),
    ];
    if let Ok(id) = std::env::var("AGENT_BROWSER_EXTENSION_ID") {
        if !id.is_empty() && !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

fn host_manifest_path() -> PathBuf {
    integration_dir().join("host-manifest.json")
}

fn write_host_manifest(executable: &Path) -> Result<PathBuf, String> {
    let executable = executable
        .canonicalize()
        .map_err(|error| format!("failed to resolve executable path: {error}"))?;
    let executable = executable.to_string_lossy();
    let executable = executable
        .strip_prefix(r"\\?\")
        .unwrap_or(&executable)
        .to_string();
    let path = host_manifest_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create integration directory: {error}"))?;
    }

    let manifest = json!({
        "name": HOST_NAME,
        "description": "Antigravity Agent Browser Native Messaging Host",
        "path": executable,
        "type": "stdio",
        "allowed_origins": extension_ids()
            .into_iter()
            .map(|id| format!("chrome-extension://{id}/"))
            .collect::<Vec<_>>()
    });
    fs::write(
        &path,
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("failed to write native host manifest: {error}"))?;
    Ok(path)
}

#[cfg(windows)]
fn register_native_host(manifest: &Path) -> Result<(), String> {
    let key = format!(
        r"HKCU\Software\Google\Chrome\NativeMessagingHosts\{}",
        HOST_NAME
    );
    let status = Command::new("reg.exe")
        .args([
            "add",
            &key,
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &manifest.to_string_lossy(),
            "/f",
        ])
        .status()
        .map_err(|error| format!("failed to start reg.exe: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("reg.exe exited with status {status}"))
    }
}

#[cfg(not(windows))]
fn register_native_host(_manifest: &Path) -> Result<(), String> {
    Err("native host installation is currently implemented only for Windows".to_string())
}

pub fn install_global() -> Result<PathBuf, String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("failed to locate executable: {error}"))?;
    let manifest = write_host_manifest(&executable)?;
    register_native_host(&manifest)?;
    Ok(manifest)
}

pub fn install_workspace(_workspace_path: &str) -> Result<PathBuf, String> {
    install_global()
}

#[cfg(windows)]
fn unregister_native_host() -> Result<(), String> {
    let key = format!(
        r"HKCU\Software\Google\Chrome\NativeMessagingHosts\{}",
        HOST_NAME
    );
    let status = Command::new("reg.exe")
        .args(["delete", &key, "/f"])
        .status()
        .map_err(|error| format!("failed to start reg.exe: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("reg.exe exited with status {status}"))
    }
}

#[cfg(not(windows))]
fn unregister_native_host() -> Result<(), String> {
    Err("native host uninstall is currently implemented only for Windows".to_string())
}

pub fn uninstall() -> Result<(), String> {
    unregister_native_host()?;
    let manifest = host_manifest_path();
    if manifest.exists() {
        fs::remove_file(&manifest)
            .map_err(|error| format!("failed to remove native host manifest: {error}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_extension_id_is_valid_and_stable() {
        assert_eq!(OFFICIAL_EXTENSION_ID.len(), 32);
        assert!(OFFICIAL_EXTENSION_ID
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'p')));
        assert!(extension_ids().contains(&OFFICIAL_EXTENSION_ID.to_string()));
    }
}
