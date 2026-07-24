# Antigravity Browser Bridge

> Beta software. Use it carefully with personal accounts and review important actions.

[Documentação em português](README.md)

Antigravity Browser Bridge connects MCP-compatible agents to the Chrome browser you already use. Its extension works with user-authorized tabs and preserves existing authenticated sessions on X, LinkedIn, and other web applications.

The project is built on [Vercel's agent-browser](https://github.com/vercel-labs/agent-browser), licensed under Apache 2.0. The Rust engine, browser automation, and parts of the CLI come from that foundation. This version adds the `antigravity-work` MCP integration, a generic Chrome extension, Windows native messaging, and explicit per-tab authorization.

![Antigravity Browser Bridge icon](extension/icons/icon-128.png)

## Why it exists

Keyboard, PowerShell, VBS, and screen-coordinate automation depend on focus, loading time, and Windows permissions. They can type into the wrong window or incorrectly report success.

Antigravity Browser Bridge uses a different architecture:

1. Antigravity starts the `agent-browser` MCP server.
2. The server communicates with the native host installed on Windows.
3. The native host maintains an authenticated bridge to the extension.
4. The extension observes the authorized tab's DOM and generates stable references.
5. The agent reads, fills, focuses, and clicks elements through those references.

This is more deterministic, but it does not make every model a perfect agent. The model must still observe the page, verify results, and handle interface changes.

## Current status

This release is `0.1.0-beta.2`.

Currently available:

- MCP integration through the `antigravity-work` profile;
- access to an already-open Chrome browser and its authenticated sessions;
- explicit per-tab authorization;
- authorized-tab listing;
- DOM observation with references;
- click, focus, fill, type, and text-reading operations;
- same-origin authorization persistence;
- automatic revocation after cross-origin navigation;
- a stable extension ID with no manual ID copying;
- Windows native-host installation and diagnostics.

Beta limitations:

- complex menus, dynamic dialogs, and some editors need richer operations;
- website interfaces can change without notice;
- destructive or public actions should require confirmation;
- native-host installation currently focuses on Windows;
- the connector still depends on site-specific DOM changes;
- publishing, replying, and deleting still require explicit confirmation.

A `not_implemented` response must never be treated as success.

The `agent_browser_work_*` tools now use the typed `WorkService`: every work item has state, deadlines, idempotency, cooperative cancellation, an append-only journal, checkpoints, resume, and redacted export. See the [Stage 1 contract](docs/WORK_CONTRACT.md).

## Requirements

- Windows 10 or 11;
- Google Chrome Stable, Beta, Dev, or Canary;
- Node.js 24 or newer;
- pnpm 11 or newer;
- stable Rust;
- Antigravity or another compatible MCP client.

## Build from source

```powershell
git clone https://github.com/coelhobugado/antigravity-browser-bridge.git
cd antigravity-browser-bridge
pnpm install
pnpm build:native
```

The Windows executable is generated at `bin/agent-browser-win32-x64.exe`.

Register the native host:

```powershell
.\bin\agent-browser-win32-x64.exe antigravity install
.\bin\agent-browser-win32-x64.exe antigravity doctor
.\bin\agent-browser-win32-x64.exe antigravity permissions
```

## Install the extension

Download the extension from the [Releases page](https://github.com/coelhobugado/antigravity-browser-bridge/releases) or use the repository's `extension` directory.

1. Extract the ZIP into a permanent directory.
2. Open `chrome://extensions`.
3. Enable Developer mode.
4. Select **Load unpacked**.
5. Select the extracted directory containing `manifest.json`.
6. Confirm that extension version `1.2.0` is displayed.

The official extension ID is stable. Users and agents do not need to copy it because the installer already authorizes it.

## Configure MCP in Antigravity

The `antigravity install` command registers the native host and automatically configures the MCP server in:

```text
C:\Users\YOUR_USER\.gemini\config\mcp_config.json
```

The installer preserves existing servers and refuses to overwrite malformed JSON. The resulting entry uses the absolute executable path:

```json
{
  "mcpServers": {
    "agent-browser": {
      "command": "C:\\PATH\\antigravity-browser-bridge\\bin\\agent-browser-win32-x64.exe",
      "args": ["mcp", "--tools", "antigravity-work"]
    }
  }
}
```

No extension ID needs to be copied. Restart Antigravity after installation and verify that the `agent_browser_work_*` tools appear.

## Usage

1. Open the target website in Chrome.
2. Click the extension icon and select **Authorize Tab** in the popup.
3. Confirm that the badge shows `ON`.
4. Ask the agent to observe before acting.
5. Require a second observation to verify posts, deletions, and messages.

Example prompt:

```text
Use the agent_browser_work tools. List the authorized tabs, observe the LinkedIn tab, prepare the post, and ask for my confirmation before clicking Publish. Observe the page again afterward and verify the result.
```

## Security

The extension does not silently receive access to the entire browser. The user must authorize each tab in the popup. Authorization is removed when the tab changes origin, closes, is explicitly revoked, or the browser session ends.

Recommended practices:

- authorize only required tabs;
- require confirmation before publishing, deleting, buying, or messaging;
- never expose tokens, cookies, or bridge state;
- run only trusted builds and releases;
- inspect structured results when success is uncertain.

## Troubleshooting

```powershell
.\bin\agent-browser-win32-x64.exe antigravity doctor
.\bin\agent-browser-win32-x64.exe antigravity permissions
```

For `Access to the specified native messaging host is forbidden`, reinstall the native host, reload the extension, and fully restart Chrome.

For `Receiving end does not exist`, reload the target tab and authorize it again. Internal pages such as `chrome://extensions` do not support content-script injection.

## Development

```powershell
pnpm install
pnpm version:sync
cargo fmt --manifest-path cli\Cargo.toml -- --check
cargo test --manifest-path cli\Cargo.toml
node --check extension\background.js
node --check extension\content.js
```

The technical roadmap is available at [docs/ANTIGRAVITY_APEX_PLAN.md](docs/ANTIGRAVITY_APEX_PLAN.md). See [known beta issues](docs/BETA_KNOWN_ISSUES.md) and [build artifacts](docs/BUILD_ARTIFACTS.md) for coverage, local caches, and safe cleanup.

The Stage 1 work contract is documented in [docs/WORK_CONTRACT.md](docs/WORK_CONTRACT.md).

## Credits and license

Based on [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser). This project preserves the Apache 2.0 license and applicable notices. See [LICENSE](LICENSE).

Antigravity Browser Bridge is not an official Vercel, Google, X, or LinkedIn product.
