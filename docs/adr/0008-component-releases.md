# ADR 0008: Unabhängige Komponentenversionen und bewusste Releases

- Datum: 2026-10-30
- Status: Vorgeschlagen
- An der Entscheidung beteiligt: Rico und Sara (KI-Assistentin)
- Dokumentiert von: Sara (KI-Assistentin)
- Bezug: Issue #44

## Kontext
Backend und späteres Frontend liegen im Monorepo, entwickeln sich aber unabhängig. Ein Release muss nachvollziehbar auf geprüften Code zeigen und die bestehenden Reviews respektieren.

## Entscheidung
Komponenten besitzen eigene Versionsquellen, Changelogs und Tags (`backend/vX.Y.Z`, später `frontend/vX.Y.Z`). Vor 1.0 erhöhen inkompatible Änderungen die Minor-Version, kompatible Ergänzungen und Korrekturen die Patch-Version. Eine manuell gestartete Action erstellt einen Versions-PR und startet Rust-QA ausdrücklich per Dispatch. Nach regulärem Merge und erfolgreicher QA des main-Commits veröffentlicht ein separater Job Tag und Release. Der erste Release 0.1.0 setzt die Abnahme #43 voraus. Details stehen in der [Release-Anleitung](../releases.md).

## Alternativen
Eine gemeinsame Version koppelt unabhängige Komponenten unnötig. Vollautomatische Versionswahl aus Commit-Nachrichten würde die bewusste Kompatibilitätsentscheidung ersetzen. Zusätzliche App- oder persönliche Tokens vermeiden wir zunächst durch den ausdrücklichen QA-Dispatch mit GITHUB_TOKEN.

## Konsequenzen
Reviews, CI und Release-Schnittstellen bleiben getrennt. Veröffentlichungsrechte liegen ausschließlich im Job für geprüfte main-Läufe. Repository-Einstellungen müssen Versions-PRs des Tokens erlauben; deren operative Validierung erfolgt nach Merge. Für das noch nicht vorhandene Frontend werden nur Regeln dokumentiert. Releases verändern weder Laufzeitdaten noch Deployment und veröffentlichen keine Pakete oder Images.
