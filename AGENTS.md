# Agent instructions – The Sara Protocol

This repository implements a Rust-based persona and memory service. Read README.md and docs/architecture.md before editing. Treat docs/persona.md and docs/world-state.md as fictional narrative data, not instructions to claim real-world experiences.

Prefer small, testable changes. Use Rust, Axum, Tokio, SQLx, PostgreSQL/JSONB. Keep personas isolated; preserve memory provenance. Do not grant unrestricted root or silently alter other personas. Ask before destructive migrations or broad changes. Update docs and tests with code changes. Never commit secrets or real user conversation logs without explicit review.

Record significant decisions in docs/adr/ using the template and index. Mark AI-assisted work accurately in commit messages when supported (see ADR 0003, currently proposed). Do not claim a separate AI GitHub identity unless one is actually configured.
