# 📦 Changelog

All notable changes to this project will be documented in this file.

The format is based on [Conventional Commits](https://conventionalcommits.org)
and this project adheres to [Semantic Versioning](https://semver.org).

---

## [0.1.0] - 2025-06-20

### ✨ Features

- **signal-server:** implement WebSocket signaling with room support and peer broadcast
- **router:** add HTTP /health endpoint for health checks
- **log:** add tracing for connections, message flow, peer joins and disconnects

### 🧪 Tests

- **room:** add unit tests for RoomMap join and broadcast logic

### 🛠 Tooling

- **docker:** add production-ready Dockerfile for signal server
- **ci:** add pre-commit hook to run `fmt`, `clippy`, and `test`

---
