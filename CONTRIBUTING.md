# Contributing to Vision

Thank you for your interest in contributing to Vision! Vision is a premium FOSS Linux desktop application for PDF management, designed with strict architecture, security, and maintenance goals.

## Architecture & Principles
Before contributing, please read the `ARCHITECTURE.md` file. Vision relies on a capability-based architecture where the Rust backend holds the document state (`VisionDocument`), and engines like qpdf provide specific capabilities. 
* Do not introduce new engines or heavy dependencies without proposing an Architecture Decision Record (ADR).
* Follow the Linux-first philosophy: Vision targets Linux and Flatpak environments natively.

## Setup Development Environment
To build Vision locally, you need:
1. Rust toolchain (`rustup`)
2. Node.js and NPM
3. `qpdf` installed on your host system (required for PDF structural operations)
4. `flatpak` and `flatpak-builder` (optional, for packaging)

```bash
# Install dependencies
npm install

# Run in dev mode
npm run dev
```

## Pull Request Process
1. **Tests are mandatory**: Any structural PDF modification must have an accompanying test in the Preservation Test Lab (`tests/fixtures/`).
2. **Format and Lint**: Run `cargo fmt`, `cargo clippy -- -D warnings`, and `npm run lint` before committing.
3. **Commit Messages**: Write clear, descriptive commit messages.

## Dependency Policy
Vision adheres to a strict dependency policy (see `SECURITY.md`). Do not add new NPM or Cargo dependencies unless the benefit significantly outweighs the maintenance and security costs. All new dependencies require explicit justification.
