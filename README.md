![The Sara Protocol: dunkles Terminal mit Pixel-Katze und dampfendem Kaffee](docs/assets/branding/banner.png)

# <img src="docs/assets/branding/logo.png" width="48" height="48" alt="Pixel-Katze mit Kaffeetasse"> The Sara Protocol

[![Rust-QA auf main](https://github.com/Skanky0815/the-sara-protocol/actions/workflows/rust-qa.yml/badge.svg?branch=main&event=push)](https://github.com/Skanky0815/the-sara-protocol/actions/workflows/rust-qa.yml) [![Veröffentlichte Backend-Version](https://img.shields.io/github/v/release/Skanky0815/the-sara-protocol?filter=backend%2Fv*&label=Backend&color=58b879)](https://github.com/Skanky0815/the-sara-protocol/releases)

[![Rust – Backend](https://img.shields.io/badge/Rust-Backend-8C4B32?logo=rust&logoColor=white)](https://www.rust-lang.org/) [![PostgreSQL – Datenbank](https://img.shields.io/badge/PostgreSQL-Datenbank-4169E1?logo=postgresql&logoColor=white)](https://www.postgresql.org/) [![Node.js – Release-Werkzeuge](https://img.shields.io/badge/Node.js-Release--Werkzeuge-417E38?logo=nodedotjs&logoColor=white)](.github/release/)

**Geplantes Frontend:**
[![Next.js – geplant](https://img.shields.io/badge/Next.js-geplant-444444?logo=nextdotjs&logoColor=white)](https://nextjs.org/)
Noch nicht implementiert; es gibt keine Frontend-Version. Node.js dient derzeit den Release-Werkzeugen, die Backend-Laufzeit ist Rust.

Lokaler Persona- und Memory-Service mit Rust, Axum und PostgreSQL. Das Backend bietet `GET /health`, das Anlegen und Abrufen von Personas sowie persona-spezifische Memories mit Provenienz. Suche und LLM-Adapter folgen in [Sprint 1](https://github.com/Skanky0815/the-sara-protocol/milestone/1); weitere World-State- und Client-Funktionen stehen auf der [Roadmap](docs/roadmap.md).

## Was schon funktioniert

- Personas anlegen und gezielt abrufen.
- Erinnerungen mit Quelle, Ereigniszeit und Bestätigungsstatus speichern und lesen.
- Daten pro Persona isolieren und Zeitangaben vor dem Schreiben prüfen.
- Health und generierte OpenAPI-Referenz bereitstellen.

[Backend v0.1.0](https://github.com/Skanky0815/the-sara-protocol/releases/tag/backend/v0.1.0) ist veröffentlicht. Die technische Sprint-0-Abnahme ist abgeschlossen; Authentifizierung, Suche und LLM-Anbindung sind noch nicht implementiert.

## Repository-Struktur

```text
backend/
  Cargo.toml, Cargo.lock   Rust-Paket und verbindlicher Lockfile
  src/, tests/            Backend und API-Tests
  migrations/             SQLx-Migrationen
  dev/persona.http        IntelliJ HTTP Client
  .env.example            Lokale Backend-Konfiguration
  Dockerfile              Backend-Image
compose.yaml              Gemeinsame lokale Orchestrierung
.github/                  CI und Projekt-Automatisierung
docs/                     Gemeinsame Projekt- und Backend-Dokumentation
AGENTS.md                 Arbeitsregeln für Coding-Agenten (Englisch)
```

Ein späteres `frontend/` kann neben `backend/` liegen. Es gibt keinen Cargo-Workspace im Root; Cargo-Befehle laufen aus `backend/`, der einzige Lockfile liegt ebenfalls dort.

## Lokal starten

Voraussetzungen: aktuelle stabile Rust-Toolchain und Docker Compose.

```sh
# Im Repository-Root:
docker compose up -d postgres
cd backend
cp .env.example .env
cargo run --locked
```

Alternativ startet `docker compose up --build -d --wait` aus dem Root das Backend und PostgreSQL gemeinsam. Die API ist unter `http://127.0.0.1:3000/health` erreichbar. Beide Startvarianten verwenden standardmäßig denselben API-Port; nur eine gleichzeitig starten.

Die Konfiguration, Tests, SQLx-Migrationspfade und Container-Prüfungen beschreibt die [Entwicklungsanleitung](docs/getting-started.md).

## Start mit vorgebautem Image

Die [Release- und Abnahmeanleitung](docs/releases.md#abnahme-ohne-lokalen-build) beschreibt den Start über `compose.acceptance.yaml` mit einem GHCR-Digest, ohne lokalen Build. Versionierte Backend-Images liegen in [GHCR](https://github.com/Skanky0815/the-sara-protocol/pkgs/container/the-sara-protocol-backend). Für reproduzierbare Starts den zum geprüften Release gehörenden Digest verwenden.

## Beiträge

[Beitragsrichtlinien und Projektsprache](CONTRIBUTING.md) beschreiben Sprache, Issue-Verweise und Validierung.

## Dokumentation

- [Architektur und API-Planung](docs/architecture.md)
- [Generierte API-Referenz (OpenAPI)](docs/api-reference.md)
- [Personas lokal verwenden](docs/persona-api.md)
- [Technischer Sprint-0-Abnahmenachweis nach Bugfix](docs/acceptance/2026-10-30-sprint0-retest.md) – [Freigabe #43](https://github.com/Skanky0815/the-sara-protocol/issues/43)
- [Komponentenversionen und Releases](docs/releases.md)
- [Architekturentscheidungen](docs/adr/README.md)
- [Persona](docs/persona.md)
- [Fiktionaler World-State mit Pixel](docs/world-state.md)
- [Arbeitsregeln](AGENTS.md)

Personas und ihre Daten bleiben isoliert. Erinnerungen behalten ihre Herkunft; fiktionale Inhalte und operative Arbeitsstände werden getrennt behandelt.

Banner, Logo und Technikzeichen: [Herkunft und Gestaltung](docs/assets/branding/README.md).
