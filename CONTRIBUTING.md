# Contributing to Antigravity Browser Bridge

Thank you for helping improve open browser automation for MCP-compatible agents.

## Good first contributions

- Reproduce and document failures on real websites.
- Add regression tests for DOM changes, dynamic menus and dialogs.
- Improve Windows installation and diagnostic messages.
- Add examples for MCP clients other than Antigravity.
- Review security boundaries around tab authorization and destructive actions.
- Improve Portuguese or English documentation.

## Development setup

Requirements:

- Windows 10 or 11
- Google Chrome
- Node.js 24+
- pnpm 11+
- stable Rust

```powershell
git clone https://github.com/coelhobugado/antigravity-browser-bridge.git
cd antigravity-browser-bridge
pnpm install
pnpm build:native
cargo test --manifest-path cli\Cargo.toml
node --check extension\background.js
node --check extension\content.js
```

End-to-end tests require Chrome:

```powershell
cargo test e2e --manifest-path cli\Cargo.toml -- --ignored --test-threads=1
```

## Before opening a pull request

1. Open or reference an issue for behavior changes.
2. Keep changes focused and explain the user-facing problem.
3. Add or update tests when behavior changes.
4. Update documentation for new commands, permissions or security boundaries.
5. Run formatting, unit tests and JavaScript syntax checks.
6. Never commit cookies, tokens, browser profiles, local state or private logs.

## Pull request expectations

A good pull request includes:

- the problem being solved;
- how the implementation works;
- security and compatibility implications;
- test evidence;
- screenshots or recordings for extension UI changes.

## Project direction

The project prioritizes user-controlled automation, observable outcomes and explicit confirmation for sensitive actions. Contributions that silently widen browser access, weaken authorization or hide failure states will not be accepted.

## Conduct

Participation is governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Security issues must follow [SECURITY.md](SECURITY.md) instead of being reported publicly.