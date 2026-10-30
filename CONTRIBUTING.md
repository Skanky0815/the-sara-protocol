# Beiträge zu The Sara Protocol

## Projektsprache
Deutsch ist die Standardsprache für menschenlesbare Projektdokumentation, neue Issue-Titel und -Beschreibungen, Akzeptanzkriterien, PR-Titel und -Beschreibungen sowie die Zusammenarbeit auf GitHub. Historische Kommentare und Diskussionen bleiben unverändert.

`AGENTS.md` bleibt aufgrund seiner ausdrücklichen Vorgabe Englisch. Code, Bezeichner, API-Verträge, Dateipfade, CLI-Befehle, Konfigurationsschlüssel und etablierte technische Fachbegriffe behalten ihre technische Schreibweise. Commit-Nachrichten dürfen weiterhin Englisch sein. Bei redaktionellen Übersetzungen Aussagen, Entscheidungsstatus und Verweise erhalten.

## Änderungen vorbereiten
Die [README](README.md), [Architektur](docs/architecture.md) und [Arbeitsregeln](AGENTS.md) lesen. Änderungen auf einem eigenen Branch entwickeln und auf einen nachvollziehbaren Umfang begrenzen. Wesentliche Entscheidungen mit der [ADR-Vorlage](docs/adr/template.md) dokumentieren und im [Verzeichnis](docs/adr/README.md) eintragen.

Jeder Implementierungs-PR verweist auf sein Issue: `Refs #N` für Teilschritte, `Closes #N` nur bei vollständiger Umsetzung mit Merge in den Standardbranch. Zusätzlich den Issue-Link angeben. PR-Beschreibungen erklären Problem, resultierendes Verhalten und tatsächliche Validierung.

## Prüfen und Review
Die in der [Entwicklungsanleitung](docs/getting-started.md) beschriebenen Prüfungen passend zur Änderung ausführen. Backend-Tests mit dedizierter PostgreSQL-Datenbank prüfen und übersprungene Datenbankfälle ausdrücklich benennen. Bei reinen Dokumentationsänderungen Sprache, fachliche Aussagen und Links prüfen; dafür sind keine zusätzlichen Rust-Tests nötig.

Keine Secrets oder echten Gesprächsprotokolle committen. Git-Identitäten wahrheitsgemäß angeben und standardmäßig keine Attribution-Trailer ergänzen. Änderungen werden per Pull Request geprüft; geltende Review- und Branch-Schutzregeln bleiben verbindlich.
