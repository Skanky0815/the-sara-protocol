# Architektur – The Sara Protocol

Status: Architekturentwurf mit implementiertem Health-Endpunkt, Persona- und Memory-API; Monorepo-Struktur gemäß Issue #31, 2026-10-28.

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
Authentifizierung, weitere Datenmodell-Erweiterungen, Embedding-Provider, Backup und Rechtekonzept sind noch zu entscheiden. Der vorhandene HTTP-Vertrag wird als OpenAPI am Code gepflegt.

## Repository und Betrieb

Das Rust-Paket einschließlich Lockfile, Quellcode, Tests, SQLx-Migrationen, lokaler Konfiguration und Dockerfile liegt unter `backend/`. Ein späteres `frontend/` kann als Geschwisterverzeichnis ergänzt werden. Es gibt keinen Root-Cargo-Workspace. Gemeinsame Orchestrierung (`compose.yaml`), CI, Arbeitsregeln und Projektdokumentation bleiben im Root bzw. unter `docs/`. Backend-spezifische Entwicklungs- und Deployment-Anweisungen bleiben zentral in [getting-started.md](getting-started.md).

Lokal wird aus `backend/` gestartet. Im Backend-Image liegt `migrations/` unter dem festen Arbeitsverzeichnis `/app/backend`; Compose startet das Backend erst nach gesundem PostgreSQL. Das bestehende Laufzeit-Migrationsmodell bleibt erhalten; ein konfigurierbarer Pfad aus Issue #24 wird hier nicht eingeführt. Siehe [ADR 0005](adr/0005-backend-directory.md).

Die [Persona-API](persona-api.md) verwendet das bestehende Schema ohne Migration: Namen werden validiert und parametrisiert gespeichert, Abrufe durch die Persona-ID eingeschränkt. Memory-Zuordnung und Request-Tracing bleiben erhalten.

Die [OpenAPI-Referenz](api-reference.md) wird gemeinsam mit den Axum-Routen registriert und unter `/openapi.json` ausgeliefert. Handler- und DTO-Annotationen bilden den Vertrag; JSON-Schema-Vertragstests vergleichen echte Antworten damit. Siehe [ADR 0006](adr/0006-generated-openapi.md).
