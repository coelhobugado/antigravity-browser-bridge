pub fn validate_permissions() -> Result<String, String> {
    let extension_id = std::env::var("AGENT_BROWSER_EXTENSION_ID")
        .unwrap_or_else(|_| "menkdnglfaljkgofohmhpblgiaehdibc".to_string());
    if extension_id.len() != 32 || !extension_id.bytes().all(|byte| matches!(byte, b'a'..=b'p')) {
        return Err(
            "AGENT_BROWSER_EXTENSION_ID must be a 32-character Chrome extension ID".to_string(),
        );
    }
    Ok(format!(
        "Native Messaging accepts the stable bundled extension ID chrome-extension://{extension_id}/. No ID needs to be copied during normal installation. Browser tabs still require explicit authorization through the extension icon."
    ))
}
