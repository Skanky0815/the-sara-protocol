# ADR 0008: Komponenten-Releases mit Release Please und CI-Images

- Datum: 2026-10-30
- Status: Vorgeschlagen
- An der Entscheidung beteiligt: Rico und Sara (KI-Assistentin)
- Dokumentiert von: Sara (KI-Assistentin)
- Bezug: #44 und #47

## Kontext
Backend und späteres Frontend benötigen unabhängige Releases im Monorepo. Die Abnahme soll vorgebaute Docker-Images verwenden und keinen lokalen Rust-Build erfordern. Reviews, QA und die erste Abnahme müssen vor einem stabilen Release bestehen bleiben.

## Entscheidung
Release Please im Manifest-Modus übernimmt Versionsvorschläge aus Conventional Commits, Cargo-/Lockfile-Änderungen, Changelogs, Versions-PRs und GitHub-Releases. Die Bibliothek ist mit npm-Lockfile auf 17.6.0 festgelegt. Tags heißen `backend/vX.Y.Z`, später `frontend/vX.Y.Z`. Die 0.x-Regel ist konfiguriert: Breaking Changes erhöhen Minor, kompatible Features Patch. Der erste Release ist 0.1.0.

Eine kleine Integration ergänzt ausschließlich projektspezifische Review-/Issue-Zuordnungen, ausdrücklichen QA-Dispatch und Release-Grenzen. Offizielle Docker-Actions bauen Commit-Images für amd64/arm64 nach main-QA und geben deren Digest aus. Die separate Compose-Datei startet diese ohne Build. Ein versioniertes Image übernimmt denselben geprüften Digest; GitHub Release und Versionstag erfordern den gemergten Versions-PR auf genau diesem QA-Commit und die abgeschlossene Abnahme #43.

## Alternativen
Siehe [Werkzeugvergleich](../release-tooling-evaluation.md). PaulHatch/semantic-version liefert Versionsberechnung, aber keinen vollständigen Release-PR-Ablauf. Release-plz passt besonders zu Rust; Release Please deckt auch das spätere Node-Frontend ab. Der erste eigene Python-Entwurf wird ersetzt, weil er Standardfunktionen unnötig selbst implementierte. Eine gemeinsame Produktversion würde unabhängige Komponenten koppeln; ein erneuter Image-Build beim Release würde ein anderes Artefakt erzeugen.

## Konsequenzen
Conventional Commits und Versionsvorschläge werden im Review geprüft. Repository- und Paketberechtigungen müssen bewusst eingerichtet werden; der operative Nachweis erfolgt nach Merge. Commit-Abnahmeimages können vor #43 entstehen, stabile Versionen erst danach. GHCR-Paket-Sichtbarkeit ist separat zu prüfen. Kein automatischer Merge, kein Deployment und keine Veröffentlichung in Cargo/npm. Bedienung und Fehlerbehandlung stehen in der [Release-Anleitung](../releases.md).
