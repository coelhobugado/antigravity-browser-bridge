pub const SKILL_MD_TEMPLATE: &str = r#"# agent-browser-work
// TODO: skill template
"#;

pub const MCP_CONFIG_TEMPLATE: &str = r#"{
  "mcpServers": {
    "agent-browser-work": {
      "command": "agent-browser",
      "args": ["mcp", "--profile", "antigravity-work"]
    }
  }
}"#;
