# Sprint-0-Abnahme: Wiederholung nach Bugfix #50

**Technische Tests bestanden: 85/85 Prüfvermerke grün. Ricos Freigabe des Nachweises steht aus.** Dies ist kein Release-Auftrag und noch kein Sprint-Gesamtabschluss. Der [erste fehlgeschlagene Lauf](2026-10-30-sprint0.md) bleibt unverändert als Historie erhalten.

## Artefakt und Umgebung

| Merkmal | Nachweis |
| --- | --- |
| Quellcommit / Merge von PR #52 | [`6494a123520b4be5bc5329ebad6dcdde8683a816`](https://github.com/Skanky0815/the-sara-protocol/commit/6494a123520b4be5bc5329ebad6dcdde8683a816) |
| Backend-Image | `ghcr.io/skanky0815/the-sara-protocol-backend@sha256:f1f4beed7f952b580dd269a483235b37602ae7f71bda4c030116fe437895e807` |
| PostgreSQL | `postgres@sha256:31ba9c94a03ada9988f2bb493a22a75eef3a185884533e5cdbcd498ea835b08f` |
| Delivery | [41098326215](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41098326215), alle sechs Jobs erfolgreich |
| Lokaler Lauf | Linux/arm64 auf Docker Desktop; ausschließlich Registry-Images, kein lokaler Build |
| Start | UID 10001, Arbeitsverzeichnis `/tmp`, `MIGRATIONS_DIR=/app/backend/migrations` |
| Isolation | Compose-Projekt `sara-sprint0-retest-20261030`, eigenes frisches Volume; Backend nur `127.0.0.1:53043`; kein DB-Host-Port |

OCI-Revision des tatsächlich gezogenen Images entspricht dem Quellcommit. Der Registry-Index enthält amd64 und arm64. Beide Architekturen haben native CI-Start-/Neustartprüfungen bestanden; die hier dokumentierten fachlichen HTTP-Tests wurden lokal auf arm64 ausgeführt. [main-QA](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41098182460) führt alle 79 Rust-Tests mit PostgreSQL erfolgreich aus.

## Ergebnisse

Die [strukturierten Einzelbelege](2026-10-30-sprint0-retest-results.json) enthalten die tatsächlich erfassten Resultate. Prüfvermerke sind Beobachtungen, keine Anzahl unabhängiger Testfälle.

- Frische Datenbank: beide Migrationen erfolgreich, vor HTTP-Tests 0 Personas / 0 Memories.
- Health und generiertes OpenAPI, Persona-Erstellung und Abruf, Namensnormalisierung, Location, Konflikte und Eingabefehler bestehen.
- Memories erhalten Inhalt, Quelle, Persona, confirmed/inferred und normalisierte UTC-Zeit. Beide Personas sehen ausschließlich ihre eigenen Daten.
- Datum ohne Uhrzeit, fehlende Zeitzone, `now`, `today`, `tomorrow`, `yesterday`, `infinity`, `-infinity`, Jahr 0000 und UTC-Überlauf über 9999 erhalten 400. Die Alpha-Liste bleibt unverändert lesbar; keine zusätzlichen Zeilen.
- `2026-10-30T16:00:00.123456789+01:00` wird als `2026-10-30T15:00:00.123456Z` gespeichert und identisch wieder gelesen.
- Vier ausschließlich synthetische Altdaten-Personas wurden direkt in der Testdatenbank mit `infinity`, `-infinity`, Jahr 10000 bzw. einem BC-Zeitpunkt befüllt. Ihre Listen antworten kontrolliert mit leerem HTTP 500. Jede Zeile bleibt erhalten; Beta und Health bleiben erreichbar. Kein Verbindungsabbruch oder Panic.
- Vollständiges `down` ohne Volumenlöschung und erneutes `up --no-build --pull never`: beide regulären Personas und alle drei gültigen Memories bleiben identisch, einschließlich Mikrosekunden. Auch die vier absichtlichen Altzeilen behalten ihr kontrolliertes Verhalten. Migrationen bleiben erfolgreich; insgesamt 6 Personas / 7 Memories.
- Erfasste Logs nach Neustart enthalten weder Testpasswort noch geprüfte synthetische Memory-Inhalte und keine Panic. Eine zusätzliche verworfene Memory dient als Logging-Probe; keine realen Gesprächsdaten.
- Eigene Container, Volume und Netzwerk sowie temporäres Passwortfile wurden anschließend entfernt. Vorhandener Entwicklungs-PostgreSQL blieb unberührt.

## Wiederholung

Der Ablauf entspricht dem [ersten Bericht](2026-10-30-sprint0.md#reproduzierbarer-ablauf), mit neuem Digest und eigenem Projektnamen. Die Compose-Datei wurde aus einem sauberen Checkout von `6494a123520b4be5bc5329ebad6dcdde8683a816` verwendet. Kein Cargo-/Image-Build, kein Quellcode-Mount. Temporäres Env-File mit Modus 0600 und zufälligem URL-sicherem Testpasswort verwenden; nicht einchecken:

```dotenv
BACKEND_IMAGE=ghcr.io/skanky0815/the-sara-protocol-backend@sha256:f1f4beed7f952b580dd269a483235b37602ae7f71bda4c030116fe437895e807
POSTGRES_PASSWORD=EIGENER_ZUFAELLIGER_HEX_WERT
BACKEND_PORT=53043
```

Override bleibt wie im ersten Bericht: Backend-`working_dir: /tmp`, festes PostgreSQL-Image aus obiger Tabelle. Verwendete Dateien: `/tmp/sara43-retest.env`, `/tmp/sara43.override.yaml`.

```sh
docker compose --env-file /tmp/sara43-retest.env -p sara-sprint0-retest-20261030 \
  -f compose.acceptance.yaml -f /tmp/sara43.override.yaml pull
docker compose --env-file /tmp/sara43-retest.env -p sara-sprint0-retest-20261030 \
  -f compose.acceptance.yaml -f /tmp/sara43.override.yaml \
  up --no-build --pull never -d --wait --wait-timeout 120
```

Reguläre Requests entsprechen `backend/dev/persona.http`. Für Grenzproben nur `observed_at` einer ansonsten gültigen Memory ersetzen, HTTP 400 erwarten und anschließend die unveränderte Liste prüfen. Für den Präzisionsfall den obigen Offsetwert verwenden. Vollständige Antworten temporär speichern und nach dem Stack-Neustart exakt vergleichen.

Altdaten ausschließlich in der isolierten Testdatenbank erzeugen, je Wert eine separate per API angelegte Persona verwenden und deren tatsächliche ID übernehmen:

```sql
INSERT INTO memories(persona_id, content, source, observed_at)
VALUES (PERSONA_ID, 'synthetic legacy fact', 'synthetic regression', 'infinity'::timestamptz);
-- Wiederholen mit -infinity, 10000-01-01T00:00:00Z und 0001-01-01 00:00:00+00 BC.
SELECT count(*) FROM memories WHERE persona_id = PERSONA_ID;
```

Den Listenabruf jeder Alt-Persona auf 500 mit leerem Body prüfen; Zeilenzahl bleibt 1. Beta-Liste und Health müssen 200 liefern. Dies bestätigt kontrollierte Fehlerbehandlung, keine automatische Reparatur bestehender Altdaten.

Persistenz prüfen mit `down` ohne `--volumes`, erneutem obigen `up` und denselben GET-Requests. Zum Schluss ausschließlich das eigene Projekt mit `down --volumes` entfernen und das Passwortfile löschen. Der Testtreiber nutzte temporär die Python-Standardbibliothek; es wurde keine zusätzliche Testsuite ins Projekt eingeführt.

## Nächste Freigabe

Bug #50 ist behoben und am neuen Registry-Image nachgeprüft. Rico prüft diesen Nachweis in PR #51. Erst nach dessen Freigabe und Merge kann die technische Abnahme #43 geschlossen werden. Die betrieblichen Versions-/Release-Nachweise #44/#47 bleiben separate offene Schritte; es wurde kein Release und keine Rechteänderung ausgelöst.
