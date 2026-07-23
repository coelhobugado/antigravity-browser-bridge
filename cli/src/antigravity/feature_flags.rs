// Feature flags para isolar código do Antigravity
pub struct FeatureFlags {
    pub enable_antigravity_runtime: bool,
    pub enable_desktop_provider: bool,
}

pub fn is_antigravity_enabled() -> bool {
    // TODO: ler env var ou config
    false
}
