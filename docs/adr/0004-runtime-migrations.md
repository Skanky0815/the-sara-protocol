# ADR 0004: Migrationen zur Laufzeit ohne SQLx-Makros laden

- Datum: 2026-10-27
- Status: Vorgeschlagen
- Dokumentiert von: Codex (KI-Assistent)

## Kontext
Die SQLx-0.8-Fassade nimmt optionale Datenbankabhängigkeiten in Cargo.lock auf, darunter sqlx-mysql und rsa. Der aktive PostgreSQL-Build verwendet sie nicht; cargo audit meldet rsa dennoch unter RUSTSEC-2023-0071, für das keine korrigierte Version verfügbar ist.

## Entscheidung
Direkt von sqlx-core und sqlx-postgres ohne Makros abhängen und `./migrations` über die Migrator-API zur Laufzeit laden. PostgreSQL, Tokio, rustls mit ring und WebPKI-Wurzeln sowie Migrationsunterstützung bleiben aktiviert. Beide Komponenten auf 0.8.6 festlegen, da die Laufzeit-/TLS-Feature-Flags interne SQLx-Schnittstellen sind; diese Flags bei einem Upgrade gemeinsam prüfen. Die Sicherheitsmeldung nicht unterdrücken.

## Alternativen
Die Sicherheitsmeldung für den ungenutzten Treiber ignorieren. Das Deaktivieren der Makros auf der Fassade wurde ebenfalls getestet, entfernt die optionalen Treiber aber nicht aus dem Lockfile.

## Konsequenzen
Der Lockfile enthält die ungenutzten Treiber und rsa nicht. Migrationsdateien sind nicht mehr in die Binary eingebettet: Deployments müssen das Migrationsverzeichnis mitliefern und den Prozess aus dessen übergeordnetem Verzeichnis starten. Laden und Anwenden müssen erfolgreich sein, bevor der Dienst seinen HTTP-Port öffnet. Ein Test prüft das Laden des Ausgangsschemas.
