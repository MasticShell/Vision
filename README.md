<p align="center">
  <img src="./docs/assets/offpdf-mark.svg" width="96" height="96" alt="Vision logo">
</p>

<h1 align="center">Vision</h1>

<p align="center">
  <strong>A premium, modern, and highly responsive Linux desktop application for PDF management.</strong><br>
  Local-first · Privacy-centric · Architecture-driven
</p>

## Vision Philosophy
Vision is a FOSS Linux application born from the foundations of OffPDF, redesigned from the ground up for strict architectural correctness, memory efficiency, and PDF preservation.

**Your files never leave your computer.** Vision processes documents locally, coordinating powerful engines like `qpdf` through a robust, memory-safe Rust backend.

## Development & Architecture
Vision is currently in active development (Milestone 1). Please refer to the following documents for our engineering principles:
* [ARCHITECTURE.md](./ARCHITECTURE.md) - Learn about our Capability-based architecture, the VisionDocument model, and engine isolation.
* [CONTRIBUTING.md](./CONTRIBUTING.md) - Guidelines for contributing, PR processes, and development setup.
* [SECURITY.md](./SECURITY.md) - Our threat model and dependency policy.

## Prerequisites
- Node.js 18+
- Rust via `rustup`
- `qpdf` (installed locally)

```bash
npm install
npm run tauri:dev
```

## License
[MIT](./LICENSE)
