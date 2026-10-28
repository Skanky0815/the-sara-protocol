# Lokale Backend-Entwicklung

Voraussetzungen: aktuelle stabile Rust-Toolchain mit `rustfmt` und Clippy sowie Docker Compose. Alle folgenden Cargo- und optionalen SQLx-CLI-Befehle laufen aus `backend/`. Compose-Befehle laufen aus dem Repository-Root. Ein Root-Workspace wird nicht verwendet; `backend/Cargo.lock` ist der einzige verbindliche Lockfile.

## PostgreSQL und lokaler Prozess

```sh
# Repository-Root:
docker compose up -d postgres
cd backend
cp .env.example .env
cargo run --locked
```

Der Dienst lädt `.env` für lokale Entwicklung. Bereits exportierte Umgebungsvariablen haben Vorrang. `HOST` ist standardmäßig `127.0.0.1`, `PORT` ist `3000`; `DATABASE_URL` ist erforderlich. Ungültige Konfiguration, Datenbankverbindungsfehler und fehlende oder unlesbare Migrationen verhindern den Start vor dem Öffnen des HTTP-Ports.

Die SQLx-Migrationen werden zur Laufzeit aus `./migrations` geladen, also lokal aus `backend/migrations`. Deshalb zuerst nach `backend/` wechseln: `cargo run --manifest-path backend/Cargo.toml` aus dem Root ändert das Arbeitsverzeichnis des gestarteten Programms nicht. `cargo build --locked` baut ebenfalls aus `backend/`. Eine separat verteilte Binary benötigt weiterhin ein mitgeliefertes `migrations/` im Arbeitsverzeichnis. Ein konfigurierbarer Migrationspfad oder separater Deployment-Migrationsschritt bleibt Gegenstand von [Issue #24](https://github.com/Skanky0815/the-sara-protocol/issues/24).

SQLx wird direkt über `sqlx-core` und `sqlx-postgres` ohne Makros verwendet; siehe [ADR 0004](adr/0004-runtime-migrations.md). Die SQLx-CLI ist nicht für Start oder Tests erforderlich. Falls installiert, verwendet `sqlx migrate run --source migrations` aus `backend/` denselben Pfad und eine exportierte `DATABASE_URL`.

```sh
# Anderes Terminal:
curl -i http://127.0.0.1:3000/health
```

Erwartet: HTTP `200 OK`, JSON `{"status":"ok"}`.

## Prüfungen

Für Datenbanktests eine dedizierte Testdatenbank verwenden. Die Suite legt temporäre Persona-Zeilen an und entfernt sie wieder. Ohne `DATABASE_URL` werden Datenbanktests übersprungen.

```sh
# Aus backend/; URL auf eine dedizierte Testdatenbank setzen:
export DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara_test
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo audit
```

Die Datenbank `sara_test` muss zuvor existieren; auf einer frischen lokalen Compose-Datenbank kann sie aus dem Root mit `docker compose exec postgres createdb -U sara sara_test` angelegt werden. `cargo audit` benötigt das separat installierte Werkzeug `cargo-audit` und Zugriff auf die aktuelle RustSec-Datenbank. Es prüft `backend/Cargo.lock`; der Lockfile wird für das Audit nicht neu erzeugt.

CI führt dieselben Prüfungen aus `backend/` aus, mit einem frischen PostgreSQL-Service und einem Cache für `backend/target`. Die separate Aufteilung in parallele Jobs bleibt [Issue #27](https://github.com/Skanky0815/the-sara-protocol/issues/27).

## Containerbetrieb

```sh
# Repository-Root:
docker build -t sara-backend:local backend
# Oder mit Compose bauen und starten:
docker compose up --build -d --wait
docker compose ps
curl --fail http://127.0.0.1:3000/health
```

Der Build-Kontext ist ausschließlich `backend/`. Die dortige `.dockerignore` lässt nur Manifest, Lockfile, Quellcode und Migrationen zu; lokale `.env`-Dateien und Build-Artefakte gelangen nicht in den Kontext. Das mehrstufige Image enthält die Release-Binary und Migrationen, läuft als unprivilegierter Benutzer und verwendet `/app/backend` als Arbeitsverzeichnis. Der HTTP-Healthcheck prüft `/health`.

Compose wartet auf den PostgreSQL-Healthcheck. Das Backend bindet im Container an `0.0.0.0:3000` und verbindet sich intern mit `postgres:5432`; Host-Zugriffe auf API und Datenbank sind auf Loopback beschränkt. PostgreSQL behält das bisherige benannte Volume `postgres_data`. Migrationen laufen vor dem HTTP-Start und sind bei erneutem Start idempotent.

Die Zugangsdaten in Compose und `.env.example` sind ausschließlich für lokale Entwicklung. Für Deployments Konfiguration über Umgebungsvariablen injizieren; keine lokale `.env` kopieren oder Zugangsdaten committen.

### Isolierter Container-Smoke-Test

Damit eine bestehende Entwicklungsdatenbank und belegte Host-Ports unverändert bleiben, kann eine temporäre Override-Datei verwendet werden (Docker Compose mit Unterstützung für `!override`):

```yaml
# /tmp/sara-smoke.yaml
services:
  postgres:
    ports: !override
      - "127.0.0.1:55431:5432"
  backend:
    ports: !override
      - "127.0.0.1:53031:3000"
```

```sh
# Repository-Root:
docker compose -p sara-smoke -f compose.yaml -f /tmp/sara-smoke.yaml up --build -d --wait
curl --fail http://127.0.0.1:53031/health
docker compose -p sara-smoke -f compose.yaml -f /tmp/sara-smoke.yaml restart backend
docker compose -p sara-smoke -f compose.yaml -f /tmp/sara-smoke.yaml up -d --wait
docker compose -p sara-smoke -f compose.yaml -f /tmp/sara-smoke.yaml down
```

Das Projekt `sara-smoke` hat ein eigenes Datenbankvolume. `down` behält Daten; `down -v` löscht die Volumes des ausgewählten Projekts und darf nur für entbehrliche Testdaten verwendet werden.

## Memories und IntelliJ HTTP Client

Die Requests liegen jetzt unter [`backend/dev/persona.http`](../backend/dev/persona.http). Diese Vorgabe aus [Issue #31](https://github.com/Skanky0815/the-sara-protocol/issues/31) ersetzt den früheren Root-Pfad `dev/persona.http` aus [Issue #11](https://github.com/Skanky0815/the-sara-protocol/issues/11). `@personaId` auf die ID einer vorhandenen Persona in der lokalen Datenbank setzen; die Persona-Erstellung ist eine separate Funktion.

- `POST /personas/{id}/memories` erwartet `content`, `source`, `observed_at` (ISO-8601 mit Zeitzone) und `confirmation_status` (`confirmed` oder `inferred`). Erfolgreiche Erstellung: HTTP 201.
- `GET /personas/{id}/memories` liefert nur die Memories der gewählten Persona, nach ID geordnet.
- Unbekannte Persona: HTTP 404. Leerer Inhalt oder Quelle, ungültiger Zeitstempel und nichtpositive Persona-ID: HTTP 400. Nicht unterstützter Bestätigungsstatus: HTTP 422.
- `created_at` bleibt die Datenbank-Einfügezeit; `observed_at` beschreibt den Zeitpunkt des Quellereignisses.

Einzelne API-Tests aus `backend/` ausführen, zum Beispiel `cargo test --locked --test memory_api unknown_persona_returns_not_found`, mit gesetzter `DATABASE_URL` wie oben. Die unabhängigen Tests verwenden jeweils eigene Personas und können parallel laufen.
