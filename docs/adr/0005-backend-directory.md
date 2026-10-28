# ADR 0005: Backend-Verzeichnis im Monorepo

- Datum: 2026-10-28
- Status: Proposed
- Dokumentiert von: Sara
- Bezug: [Issue #31](https://github.com/Skanky0815/the-sara-protocol/issues/31)

## Kontext

Das Rust-Backend lag im Repository-Root. Eine spätere Weboberfläche benötigt einen eigenen Bereich, ohne Backend-Quellen und gemeinsame Projektdateien zu vermischen.

## Entscheidung

Das vollständige Rust-Paket mit `Cargo.toml`, `Cargo.lock`, `src/`, `tests/`, `migrations/`, `dev/` und `.env.example` wird nach `backend/` verschoben. Kein Cargo-Workspace im Root: Es gibt derzeit genau ein Rust-Paket und einen Lockfile unter `backend/`. Gemeinsame Dokumentation, CI und Compose bleiben zentral. Ein späteres `frontend/` wird ein Geschwisterverzeichnis.

Das Backend erhält einen Docker-Build mit Kontext `backend/`. Das Runtime-Image läuft als unprivilegierter Benutzer aus `/app/backend` und enthält dort `migrations/`. Lokal und in CI laufen Cargo-Befehle aus `backend/`. CI nutzt den vorhandenen Lockfile unverändert und cacht das neue Target-Verzeichnis.

## Alternativen

Ein Root-Cargo-Workspace wäre für mehrere Rust-Pakete sinnvoll, fügt derzeit aber eine zweite Manifest-Ebene ohne Nutzen hinzu. Nur Quellcode zu verschieben würde Paket-, Test- und Konfigurationsdateien weiterhin über das Root verteilen.

## Konsequenzen

API und Datenbankschema ändern sich nicht. Relative Runtime-Migrationen gemäß ADR 0004 bleiben erhalten, haben aber einen ausdrücklich festgelegten Startort. Ein konfigurierbarer Pfad oder separater Deployment-Migrationsschritt aus Issue #24 bleibt separat. Die CI-Aufteilung aus Issue #27 muss die neuen Backend-Pfade verwenden. Die frühere Root-Pfad-Vorgabe `dev/persona.http` aus Issue #11 wird durch `backend/dev/persona.http` abgelöst.
