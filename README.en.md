# Antigravity Browser Bridge

> Open-source bridge between MCP-compatible agents and the user's already-authenticated Chrome browser, with explicit per-tab authorization, persistent work execution, and DOM-based verification.

[Documentação em português](README.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [Roadmap](docs/ANTIGRAVITY_APEX_PLAN.md) · [Changelog](CHANGELOG.md) · [Upstream relationship](UPSTREAM.md)

> **Status:** `0.1.0-beta.3`. Use it carefully with personal accounts and review important actions.

![Antigravity Browser Bridge icon](extension/icons/icon-128.png)

## Version overview

| Component | Current version |
|---|---:|
| Antigravity Browser Bridge | `0.1.0-beta.3` |
| Chrome extension | `1.2.0` |
| agent-browser foundation | upstream `0.33.x` line |

These versions are independent: the extension, the Bridge, and the upstream foundation do not necessarily share the same version number.

## The problem this project solves

Keyboard, PowerShell, VBS, and screen-coordinate automation depend on focus, loading time, and visual state. They can click the wrong window, miss interface changes, or report success without evidence.

Antigravity Browser Bridge provides a more deterministic layer for agents:

1. the MCP client starts the server;
2. the server communicates with a local native host;
3. the native host maintains an authenticated bridge to the extension;
4. the user explicitly authorizes each tab;
5. the extension observes the DOM and generates structured references;
6. the `WorkService` controls state, approval, execution, verification, and recovery;
7. the agent records the result and can resume interrupted work without repeating confirmed effects.

```text
MCP agent
   ↓
MCP server / Rust CLI
   ↓
Typed WorkService
   ↓
Native Messaging Host
   ↓
Chrome extension
   ↓
Authorized tab and DOM
```

## What makes this project different

The project is derived from [Vercel's agent-browser](https://github.com/vercel-labs/agent-browser), under Apache 2.0, but adds a separate layer for user-controlled operation of the user's everyday browser:

- the `antigravity-work` MCP profile;
- reuse of the already-open Chrome browser and existing authenticated sessions;
- a dedicated Chrome extension;
- Windows native messaging;
- explicit per-tab authorization;
- same-origin-only authorization persistence;
- automatic revocation after cross-origin navigation;
- a typed `WorkService` with a state machine;
- deadlines, idempotency, and cooperative cancellation;
- an append-only journal, checkpoints, and resume;
- explicit approval before sensitive actions;
- DOM-based post-action verification;
- redacted export to reduce sensitive-data exposure;
- automatic native-host and MCP installation and diagnostics;
- an independent roadmap for recovery, evidence of success, and risk control.

These additions turn the upstream foundation into a bridge for agents that need to operate real authenticated sessions without silently receiving access to the entire browser.

See [UPSTREAM.md](UPSTREAM.md) for a clear breakdown of inherited components, Bridge-specific work, and preserved upstream history.

## Work runtime

The `antigravity-work` profile uses a `WorkService` separated from the MCP adapter. Every work item has its own identity, state, attempts, deadline, idempotency key, and persistent journal.

Main flow:

```text
created → planning → waiting_for_tab → observing
observing → waiting_for_approval → executing → verifying → completed
executing/verifying → recovering → observing or executing
any non-terminal state → failed or cancelled
```

Available operations include:

- starting a session;
- observing an authorized tab;
- requesting approval;
- executing a step;
- verifying the result;
- reading status and journal entries;
- cancelling;
- creating a checkpoint;
- resuming;
- exporting redacted state.

Invalid transitions are rejected before the journal is changed. Confirmed effects are protected against repetition through idempotency. The complete contract is documented in [docs/WORK_CONTRACT.md](docs/WORK_CONTRACT.md).

## Use cases

- prepare a LinkedIn post and request confirmation before publishing;
- fill forms in already-authenticated tabs;
- inspect web systems without sharing cookies with the model;
- automate repetitive work with DOM verification afterward;
- resume interrupted workflows from checkpoints;
- connect Antigravity or another MCP client to browser workflows;
- test agents against real web applications with explicit authorization limits.

## Current status

Currently available:

- MCP integration through the `antigravity-work` profile;
- access to the already-open Chrome browser;
- authorized-tab listing;
- DOM observation with references;
- click, focus, fill, type, and text-reading operations;
- same-origin authorization persistence;
- automatic revocation after cross-origin navigation;
- a stable extension ID;
- a popup for authorizing and revoking tabs;
- connection status and recent activity;
- Windows native-host installation and diagnostics;
- automatic MCP configuration in Antigravity;
- WorkService journal, checkpoints, resume, idempotency, and verification.

Beta limitations:

- complex menus, dynamic dialogs, and some editors still need richer operations;
- website interfaces can change without notice;
- native-host installation currently focuses on Windows;
- not every planned tool is implemented;
- a `not_implemented` response must never be treated as success.

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

Register and validate the native host:

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
5. Select the directory containing `manifest.json`.
6. Confirm that extension version `1.2.0` is displayed.

The official extension ID is stable and is already authorized by the installer.

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

## Safe usage

1. Open the target website in Chrome.
2. Open the extension popup and authorize only the required tab.
3. Ask the agent to observe before acting.
4. Require confirmation before publishing, deleting, buying, or messaging.
5. Ask for another observation after the action to verify the result.

Example prompt:

```text
Use the agent_browser_work tools. List the authorized tabs, observe the LinkedIn tab, prepare the post, and ask for my confirmation before clicking Publish. Observe the page again afterward and verify the result.
```

The extension does not silently receive access to the entire browser. Never expose cookies, tokens, browser profiles, or local bridge state.

## Development and tests

```powershell
pnpm install
pnpm version:sync
cargo fmt --manifest-path cli\Cargo.toml -- --check
cargo test --manifest-path cli\Cargo.toml
node --check extension\background.js
node --check extension\content.js
```

Chrome end-to-end tests:

```powershell
cargo test e2e --manifest-path cli\Cargo.toml -- --ignored --test-threads=1
```

CI also validates version synchronization, Clippy, the extension, encoding, baselines, packaging, SBOM generation, and global package installation.

## Contributing

Contributions are welcome, especially for:

- testing against real applications;
- support for menus, dialogs, and editors;
- compatibility with other MCP clients;
- installers for other operating systems;
- security and recovery behavior;
- Portuguese and English documentation.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Report vulnerabilities according to [SECURITY.md](SECURITY.md).

## Roadmap

Current priorities include:

- risk-level confirmation policies;
- richer operations for dynamic interfaces;
- recovery after DOM changes;
- automatic evidence of success or failure;
- signed installers and packages;
- coverage for X, LinkedIn, and generic applications;
- broader MCP-client and operating-system support.

The detailed plan is available in [docs/ANTIGRAVITY_APEX_PLAN.md](docs/ANTIGRAVITY_APEX_PLAN.md).

## Credits and license

Based on [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser). This project preserves the Apache 2.0 license and applicable notices. See [LICENSE](LICENSE) and [UPSTREAM.md](UPSTREAM.md).

Antigravity Browser Bridge is not an official Vercel, Google, X, LinkedIn, or Anthropic product.
