# Architektur – The Sara Protocol

Status: Entwurf, 2026-10-27. Noch kein implementierter Dienst.

## Ziel
Portabler Persona- und Memory-Service für ChatGPT, lokale LLMs und IDE-Clients. REST zum Abrufen, Suchen und Persistieren. Trennung von Persona, Erinnerung und World-State.

## Geplanter Stack
- Rust, Axum, Tokio, Serde, SQLx
- PostgreSQL mit JSONB und perspektivisch pgvector; MongoDB als diskutierte Alternative
- Docker Compose auf macOS; Entwicklung mit IntelliJ IDEA Ultimate
- LLM-Anbindung später, zunächst unabhängige API

## MVP
- GET /health
- POST /personas, GET /personas/{id}
- POST /personas/{id}/memories, GET /personas/{id}/memories
- Migrationen, Tests, lokale OpenAPI-Dokumentation

## Designprinzipien
- Isolation mehrerer Personas und ihrer Daten
- Memory Provenance: Quelle, Zeitpunkt, bestätigt vs. abgeleitet
- Administrative Änderungen nur autorisiert, auditierbar, versioniert und rücksetzbar; keine echten System-Root-Rechte
- Suchindex zunächst in PostgreSQL; keine separate Suchmaschine im MVP
- Orte und NPCs als spätere World-State-Entitäten

## Offene Punkte
Authentifizierung, API-Schema, Datenmodell, Embedding-Provider, Backup und Rechtekonzept sind noch zu entscheiden.
