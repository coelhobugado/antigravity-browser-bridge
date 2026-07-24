# Security Policy

Antigravity Browser Bridge interacts with authenticated browser sessions and must be treated as security-sensitive software.

## Supported versions

Security fixes are currently provided for the latest published beta release and the default branch.

## Reporting a vulnerability

Do not open a public issue for vulnerabilities involving:

- tab authorization bypass;
- native messaging authentication;
- exposure of cookies, tokens or browser state;
- command execution outside the intended native host flow;
- cross-origin authorization persistence;
- silent access to unauthorized tabs;
- unsafe execution of destructive or public actions.

Instead, use GitHub's private vulnerability reporting feature when available. Include:

- affected version or commit;
- operating system and Chrome version;
- reproduction steps;
- expected and observed behavior;
- impact assessment;
- proof of concept, logs or screenshots with secrets removed.

Please do not include real credentials, cookies, tokens, private messages or personal browser data.

## Security principles

The project follows these principles:

1. Browser access must be explicitly authorized by the user.
2. Authorization should be scoped to the selected tab and origin.
3. Cross-origin navigation should revoke authorization.
4. Public, destructive, financial or irreversible actions should require confirmation.
5. Structured evidence should be preferred over unverified success claims.
6. Secrets and browser profiles must never be sent to the model.
7. Failures and `not_implemented` responses must remain explicit.

## Disclosure

Please allow a reasonable period for investigation and remediation before public disclosure. Confirmed reporters may be credited unless they prefer to remain anonymous.