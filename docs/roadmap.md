# Roadmap

Stand: 2026-10-30. Implementiert bedeutet nicht automatisch abgenommen oder veröffentlicht. GitHub ist die Quelle für den Arbeitsstatus.

## Sprint 0 – implementierte Grundlagen

- [x] Cargo-Projekt, Axum/Tokio und Health-Endpunkt — [#8](https://github.com/Skanky0815/the-sara-protocol/issues/8), PR #21.
- [x] PostgreSQL, Konfiguration und SQLx-Migrationen — [#9](https://github.com/Skanky0815/the-sara-protocol/issues/9), PR #23.
- [x] Persona-API und Isolation — [#10](https://github.com/Skanky0815/the-sara-protocol/issues/10), PR #38.
- [x] Memory-API mit Provenienz — [#11](https://github.com/Skanky0815/the-sara-protocol/issues/11), PR #26; Zeitstempel-Fehler siehe Restpunkte.
- [x] API-Tests und generierte OpenAPI — [#12](https://github.com/Skanky0815/the-sara-protocol/issues/12), PR #40.
- [x] Rust-QA, parallele Checks und schnelles Dependency-Audit — #19/#27/#22, PR #21/#34/#36.
- [x] Monorepo-Struktur, Logging und fokussierte Tests — #31/#30/#37, PR #33/#35/#39.
- [x] Konfigurierbarer Migrationspfad und deutsche Projektdokumentation — #24/#25, PR #41/#42.
- [x] Release-Please-Integration und vorgebaute native amd64-/arm64-Commit-Images — PR #46/#49; operative Restpunkte #44/#47 bleiben offen.

## Sprint 0 – vor Abschluss noch offen

- [ ] [#50: RFC3339-Zeitstempelvalidierung und kontrollierter Umgang mit Sonderwerten](https://github.com/Skanky0815/the-sara-protocol/issues/50) — blockiert die technische Freigabe.
- [ ] [#43: technische Abnahme und Dokumentationsabgleich](https://github.com/Skanky0815/the-sara-protocol/issues/43) — [erster Registry-Image-Nachweis](acceptance/2026-10-30-sprint0.md): Kernfälle bestanden, wegen #50 noch nicht freigegeben.
- [ ] [#44: echter Versions-PR-/Release-Betriebsnachweis](https://github.com/Skanky0815/the-sara-protocol/issues/44) — Umsetzung vorhanden; Berechtigung und realer Ablauf noch offen.
- [ ] [#47: versioniertes Image mit identischem geprüftem Digest](https://github.com/Skanky0815/the-sara-protocol/issues/47) — Commit-Image veröffentlicht; stabiler Release folgt erst nach Abnahme.

## Danach

- [ ] Persona-/Memory-Suche — #13; Volltext zuerst, semantische Suche später bewerten.
- [ ] LLM-Adapter für lokale und externe Modelle — #14.
- [ ] World-State für Orte, NPCs und Szenen — #15.
- [ ] Versionierung, Audit und Rollback der Persona-Administration — #16.
- [ ] Client-/IDE-Integration — #17; noch keine ChatGPT-Verbindung hergestellt.
- [ ] Globale Charakteridentität, Kontexte und Zeitlinien — #28; darauf aufbauende Weboberfläche — #29.
- [ ] Logo, Banner und README-Gestaltung — #45, keine technische Abnahmevoraussetzung.
- [ ] Automatischer PR-Feedback-Agent — #5/#6/#7; bisher nur Ereignisannahme, kein ausführender Runner.
