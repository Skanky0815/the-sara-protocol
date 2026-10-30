# ADR 0007: Konfigurierbarer Migrationspfad

- Datum: 2026-10-30
- Status: Vorgeschlagen
- Dokumentiert von: Sara (KI-Assistentin)

## Kontext
ADR 0004 lädt Migrationen zur Laufzeit. Der feste relative Pfad koppelt den Dienst an sein Arbeitsverzeichnis. Issue #24 verlangt einen zuverlässigen Deployment-Pfad.

## Entscheidung
`MIGRATIONS_DIR` konfiguriert den Pfad; Standard bleibt `./migrations`. Leere Werte sind ungültig. Relative Pfade beziehen sich auf das Arbeitsverzeichnis. Deployments verwenden absolute Pfade; das Image setzt `/app/backend/migrations`. Laden und Anwenden bleiben vor dem HTTP-Start verpflichtend. Dies ergänzt die Pfadentscheidung aus ADR 0004.

## Alternativen
Ein separater Migrationsjob würde zusätzliche Orchestrierung erfordern. Eingebettete Migrationen würden den bewusst gewählten Verzicht auf SQLx-Makros ändern.

## Folgen
Migrationen werden weiterhin mit ausgeliefert. Der Container benötigt sein Arbeitsverzeichnis nicht mehr zur Pfadauflösung. Fehler bleiben kategorisiert und beenden den Start. Tests prüfen Start aus fremdem Verzeichnis sowie fehlende und ungültige Migrationen.
