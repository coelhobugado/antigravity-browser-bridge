## What changed

Describe the user-facing and technical changes.

## Why

Explain the problem this pull request solves.

## Validation

- [ ] `cargo fmt --manifest-path cli\Cargo.toml -- --check`
- [ ] `cargo test --manifest-path cli\Cargo.toml`
- [ ] `node --check extension\background.js`
- [ ] `node --check extension\content.js`
- [ ] Relevant end-to-end or manual tests were completed

## Security impact

Describe any effect on tab authorization, native messaging, origins, credentials, destructive actions or confirmation behavior. Write `None` when not applicable.

## Documentation

- [ ] Documentation was updated when behavior, installation or permissions changed
- [ ] No cookies, tokens, browser profiles or private logs are included

## Screenshots or recordings

Add evidence for extension UI or browser behavior changes when relevant.