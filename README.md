# The Sara Protocol

Lokaler Persona- und Memory-Service mit Rust, Axum und PostgreSQL. Das Backend bietet `GET /health`, das Anlegen und Abrufen von Personas sowie persona-spezifische Memories mit Provenienz. Weitere Such-, World-State- und Client-Funktionen stehen auf der [Roadmap](docs/roadmap.md).

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

## Beiträge

[Beitragsrichtlinien und Projektsprache](CONTRIBUTING.md) beschreiben Sprache, Issue-Verweise und Validierung.

## Dokumentation

- [Architektur und API-Planung](docs/architecture.md)
- [Generierte API-Referenz (OpenAPI)](docs/api-reference.md)
- [Personas lokal verwenden](docs/persona-api.md)
- [Architekturentscheidungen](docs/adr/README.md)
- [Persona](docs/persona.md)
- [Fiktionaler World-State mit Pixel](docs/world-state.md)
- [Arbeitsregeln](AGENTS.md)

Personas und ihre Daten bleiben isoliert. Erinnerungen behalten ihre Herkunft; fiktionale Inhalte und operative Arbeitsstände werden getrennt behandelt.
