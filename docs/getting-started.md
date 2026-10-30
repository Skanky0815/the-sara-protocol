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

Die SQLx-Migrationen werden zur Laufzeit standardmäßig aus `./migrations` geladen, also lokal aus `backend/migrations`. Deshalb ohne weitere Konfiguration zuerst nach `backend/` wechseln: `cargo run --manifest-path backend/Cargo.toml` aus dem Root ändert das Arbeitsverzeichnis des gestarteten Programms nicht. Alternativ `MIGRATIONS_DIR` auf das mitgelieferte Migrationsverzeichnis setzen, für Deployments vorzugsweise als absoluten Pfad. Eine separat verteilte Binary benötigt die SQL-Dateien weiterhin; siehe [Migrationspfad bei Deployments](#migrationspfad-bei-deployments).

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

CI führt diese Prüfungen aus `backend/` in vier unabhängigen Jobs aus: `Format`, `Clippy`, `Tests (PostgreSQL)` und `Dependency audit`. Ein Formatfehler überspringt weder Linting noch Tests oder Audit. Nur der Test-Job startet einen frischen PostgreSQL-Service und setzt `DATABASE_URL`; Clippy und Tests cachen `backend/target` mit getrennten Cache-Schlüsseln. Der eingecheckte Lockfile bleibt verbindlich.

Der Workflow startet bei Pull Requests und Pushes auf `main` nur für Änderungen unter `backend/**` oder an `.github/workflows/rust-qa.yml`. Reine Dokumentations- oder Frontend-Änderungen lösen keine Rust-Prüfungen aus. Werden künftig gemeinsam genutzte Rust-/CI-Konfigurationen außerhalb von `backend/` ergänzt, muss der Pfadfilter erweitert werden.

Der aktuelle Repository-Regelsatz verlangt keine Rust-Statuschecks. Falls diese später verpflichtend werden, darf ein auf Workflow-Ebene übersprungener Check reine Doku-/Frontend-PRs nicht blockieren: dafür zuerst einen immer ausgeführten Ergebnis-Job mit interner Änderungserkennung einführen oder die Pflichtcheck-Strategie entsprechend anpassen. Die Review-Pflicht bleibt unverändert.

CI installiert `cargo-audit` in Version `0.22.2` als vorgebautes Release-Binary mit der auf einen Commit gepinnten `taiki-e/install-action`. Die SHA256-Prüfung bleibt aktiv; ein Fallback auf Quellcode-Kompilierung ist deaktiviert. Werkzeugversion und Action-Commit werden bei Updates gemeinsam geprüft. Die Version wird im Job ausgegeben. `cargo audit` lädt die aktuelle RustSec-Datenbank weiterhin selbst; Audit-Ergebnisse und Advisory-Datenbank werden nicht gecacht. Findings oder Download-/Prueffehler lassen den Job fehlschlagen.

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

Die Requests liegen jetzt unter [`backend/dev/persona.http`](../backend/dev/persona.http). Diese Vorgabe aus [Issue #31](https://github.com/Skanky0815/the-sara-protocol/issues/31) ersetzt den früheren Root-Pfad `dev/persona.http` aus [Issue #11](https://github.com/Skanky0815/the-sara-protocol/issues/11). Zuerst den vorbereiteten `POST /personas`-Request ausführen und die Antwort-ID in `@personaId` übernehmen. Datenmodell und Fehlerantworten beschreibt die [Persona-API](persona-api.md).

- `POST /personas/{id}/memories` erwartet `content`, `source`, `observed_at` (ISO-8601 mit Zeitzone) und `confirmation_status` (`confirmed` oder `inferred`). Erfolgreiche Erstellung: HTTP 201.
- `GET /personas/{id}/memories` liefert nur die Memories der gewählten Persona, nach ID geordnet.
- Unbekannte Persona: HTTP 404. Leerer Inhalt oder Quelle, ungültiger Zeitstempel und nichtpositive Persona-ID: HTTP 400. Nicht unterstützter Bestätigungsstatus: HTTP 422.
- `created_at` bleibt die Datenbank-Einfügezeit; `observed_at` beschreibt den Zeitpunkt des Quellereignisses.

Einzelne API-Tests aus `backend/` ausführen, zum Beispiel `cargo test --locked --test memory_api unknown_persona_returns_not_found`, mit gesetzter `DATABASE_URL` wie oben. Die unabhängigen Tests verwenden jeweils eigene Personas und können parallel laufen.

## Strukturiertes Logging

Das Backend initialisiert beim Start genau einen `tracing-subscriber` und schreibt JSON-Zeilen nach stderr. Ohne `RUST_LOG` (oder bei leerem Wert) gilt `the_sara_protocol=info`. `.env` wird vorher geladen; exportierte Werte haben Vorrang. Ungültige Filter fallen mit einer Warnung ohne Wiedergabe des Eingabewerts auf den Standard zurück.

```sh
# Aus backend/:
RUST_LOG=debug cargo run --locked
RUST_LOG=the_sara_protocol=info,the_sara_protocol::memories=debug cargo run --locked
# Aus dem Root, mit explizitem Filter im Container:
docker compose run --rm --service-ports -e RUST_LOG=debug backend
```

Filter gelten innerhalb der Anwendungsziele `the_sara_protocol` und `the_sara_protocol::*`. Fremdbibliotheks-Logs werden bewusst auch bei globalem `debug` nicht ausgegeben, weil SQL-, Protokoll- oder Treiberdiagnosen sensible Inhalte enthalten können. SQLx-Statement-Logging wird bei der produktiven Pool-Erstellung zusätzlich deaktiviert. Modulspezifische Filter funktionieren; ein Debug-Filter erzeugt aber keine zusätzlichen Payload-Logs.

Jeder HTTP-Request erhält einen `http.request`-Span mit prozesslokaler `request_id`, Methode und gematchter Routenvorlage. Unbekannte Routen heißen `<unmatched>`; konkrete Pfadparameter, URLs und Query-Strings werden nicht ausgegeben. Das Abschlussereignis enthält HTTP-Status und `duration_ms` bis zur Erstellung der Response-Header (nicht bis zum vollständigen Streamen des Bodys). Beispiel ohne Nutzdaten:

```json
{"level":"INFO","fields":{"message":"request completed","status":200,"duration_ms":0.12},"span":{"name":"http.request","request_id":1,"method":"GET","route":"/health"}}
```

Erfolgreiche Requests werden mit `INFO`, Clientfehler mit `WARN` und Serverfehler mit `ERROR` erfasst. Start, Datenbankverbindung, Migrationen und Listener erhalten eigene Phasenfelder. Datenbankfehler enthalten nur Operation und freigegebene `error_kind`-Kategorie (z. B. `pool_closed` oder `invalid_timestamp`), niemals rohe Fehlermeldungen, SQL oder Parameter. Request-Fehler lassen sich über denselben Span dem Abschluss zuordnen. Für vollständigen Request-Kontext den Standard-Info-Filter beibehalten; ein höherer Mindestlevel kann Info-Spans unterdrücken.

Authorization-/Cookie-Header, Tokens, Passwörter, Verbindungs-URLs, Bodies und Persona-/Memory-Inhalte gehören nicht in Logs, auch nicht bei Debug. Konfigurations-Debugausgaben schwärzen die Datenbank-URL. Auf eine pauschale `instrument`-Aufzeichnung von Funktionsargumenten wird verzichtet. Die bestehenden öffentlichen API-Fehlerantworten ändern sich nicht.

Der Integrationstest-Einstieg `backend/tests/logging.rs` bindet die getrennten Module unter `backend/tests/logging_cases/` ein (Filter, Requests, Datenbank, Konfiguration und Start; gemeinsame Helfer in `support.rs`). Die Tests erfassen echte JSON-Ausgaben und prüfen Filter, Request-Zuordnung, Status/Dauer, Start-/Datenbankfehler sowie das Ausbleiben von Test-Secrets. Die PostgreSQL-Fälle benötigen wie die übrigen Integrationstests eine dedizierte `DATABASE_URL`.

## OpenAPI lokal abrufen

Nach dem Backend-Start liefert `GET /openapi.json` den aus Handlern und Datentypen generierten Vertrag. Abruf, Pflege am Code und Tests sind in der [API-Referenz](api-reference.md) beschrieben. `cargo test --locked --test openapi_contract` prüft die HTTP-Antworten gegen die Schemas; für die Datenbankfälle muss eine dedizierte `DATABASE_URL` gesetzt sein.

## Migrationspfad bei Deployments

`MIGRATIONS_DIR` legt das Verzeichnis der SQLx-Migrationen fest. Ohne Angabe bleibt `./migrations` relativ zum Arbeitsverzeichnis erhalten. Leere Werte werden als Konfigurationsfehler abgewiesen. Relative Pfade werden relativ zum Arbeitsverzeichnis ausgewertet; für Dienste außerhalb von `backend/` einen absoluten Pfad verwenden:

```sh
MIGRATIONS_DIR=/opt/sara/backend/migrations /opt/sara/bin/the-sara-protocol
```

Das Deployment muss sämtliche versionierten SQL-Dateien aus `backend/migrations/` mit ausliefern. Der Container setzt `MIGRATIONS_DIR=/app/backend/migrations` und funktioniert dadurch auch bei überschriebenem Arbeitsverzeichnis. Bei einem anderen Mount-Pfad die Variable entsprechend überschreiben. Es gibt keinen separaten Migrationsjob: Laden und Anwenden erfolgen vor Öffnen des HTTP-Ports. Fehlende, unlesbare oder ungültige Migrationen bzw. fehlgeschlagene Anwendungen beenden den Prozess mit Fehlerstatus und strukturiertem Fehlerereignis. Details und Verbindungsdaten werden nicht protokolliert. Siehe [ADR 0007](adr/0007-configurable-migrations.md).
