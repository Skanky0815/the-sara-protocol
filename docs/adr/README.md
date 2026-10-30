# Architekturentscheidungen (ADRs)

Wesentliche technische Entscheidungen werden als unveränderliche, nummerierte Markdown-Dokumente festgehalten. Neue Entscheidungen beginnen als „Vorgeschlagen“; nach Annahme wird der Status auf „Akzeptiert“ gesetzt. Die bisherigen Statuswerte `Proposed` und `Accepted` werden bei Übersetzungen als „Vorgeschlagen“ und „Akzeptiert“ wiedergegeben. Abgelöste Entscheidungen bleiben als Historie erhalten und verweisen auf ihre Nachfolger. Redaktionelle Übersetzungen ändern den Entscheidungsinhalt und den Status nicht.

Vorlage: `NNNN-short-title.md`

Jeder ADR hält Datum, Status, Kontext, Entscheidung, Alternativen, Konsequenzen und Urheberschaft/Herkunft fest. Die [ADR-Vorlage](template.md) enthält die deutsche Grundstruktur.

## Verzeichnis
- [0001 – Rust-Webdienst mit Axum](0001-rust-axum.md) — Akzeptiert (nachträglich dokumentiert)
- [0002 – PostgreSQL mit JSONB und optionalem pgvector](0002-postgresql-jsonb.md) — Akzeptiert (nachträglich dokumentiert)
- [0003 – Kennzeichnung von KI-Beiträgen](0003-ai-contribution-attribution.md) — Vorgeschlagen
- [0004 – Migrationen zur Laufzeit ohne SQLx-Makros laden](0004-runtime-migrations.md) — Vorgeschlagen
- [0005 – Backend-Verzeichnis im Monorepo](0005-backend-directory.md) — Vorgeschlagen
- [0006 – OpenAPI aus Handlern und Datentypen generieren](0006-generated-openapi.md) — Vorgeschlagen
- [0007 – Konfigurierbarer Migrationspfad](0007-configurable-migrations.md) — Vorgeschlagen
- [0008 – Unabhängige Komponentenversionen und bewusste Releases](0008-component-releases.md) — Vorgeschlagen

- [0009 – Native parallele Backend-Builds und Runtime-Packaging](0009-native-backend-builds.md) — Vorgeschlagen

Die Angabe der Urheberschaft in einem ADR ist eine Dokumentationsaussage und kein Beweis der Git-Commit-Urheberschaft. GitHub-Connector-Commits können die Identität der Integration verwenden; keine menschliche Identität vortäuschen oder die Existenz eines KI-GitHub-Kontos behaupten.
