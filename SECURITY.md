# Security Policy

Vision is an entirely local, offline application designed to process untrusted documents safely.

## Supported versions
Security fixes target the latest published release and the current default branch.

## Reporting a vulnerability
Please do not post exploit details or sensitive sample documents in a public issue. `Vision security report`.

A useful report includes:
- A short description of the issue and its potential impact.
- The smallest set of steps needed to reproduce it.
- The operating system and Vision version or commit.
- A minimal synthetic sample file, if one is necessary and safe to share.

Do not send a private or confidential document. A redacted or newly generated fixture is strongly preferred.

## Scope & Threat Model
Vision treats every PDF as **untrusted input**. Security-sensitive areas include:
- File handling and path validation (no path traversal).
- Subprocess invocation (strict argument separation, no shell interpolation).
- Temporary file cleanup.
- Dependency vulnerabilities (Supply Chain security).
- Anything that could break the offline, no-upload privacy promise.

**Flatpak permissions**: Vision requests minimal permissions. It must never request `--filesystem=host` for its core operation.

## Dependency Policy
Vision enforces strict dependency governance:
1. **Justification**: No library is added merely for convenience.
2. **Lockfiles**: `Cargo.lock` and `package-lock.json` must be committed and verified.
3. **Supply Chain**: External binaries (if any) must be validated via SHA256 checksums.
