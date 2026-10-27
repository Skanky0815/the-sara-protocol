# The Sara Protocol

A planned local-first **persona, memory and world-state service** for multiple AI characters and LLM clients. Early architecture phase: the Rust MVP has not yet been verified as implemented.

## Vision
Store personas and memories with provenance, retrieve context through a REST API, and later manage fictional places, NPCs and scene state. Persona isolation and auditable administrative changes are core requirements.

## Proposed stack
Rust · Axum · Tokio · Serde · SQLx · PostgreSQL (JSONB, later pgvector) · Docker Compose · IntelliJ IDEA Ultimate on macOS.

## Documentation
- [Architecture and proposed API](docs/architecture.md)
- [Persona summary](docs/persona.md)
- [Fictional world-state: Sara's apartment and Pixel](docs/world-state.md)
- [MVP roadmap](docs/roadmap.md)
- [Architecture Decision Records](docs/adr/README.md)
- [Coding-agent guidelines](AGENTS.md)

## Getting started (planned)
1. Install Rust/Cargo and Docker on macOS.
2. Open the repository in IntelliJ with Rust support.
3. Initialize the Cargo service if not yet present.
4. Implement and test GET /health before adding persistence.

No executable setup commands are documented yet because the implementation state has not been verified.

## Principles
Local-first development; clear distinction between fictional persona content and operational system behavior; source-aware memories; multi-persona isolation; versioned, reversible administrative changes.
