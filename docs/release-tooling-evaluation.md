# Werkzeugvergleich für Komponenten-Releases

Stand: 2026-10-30 · Issue #44 · PR #46

## Ergebnis

**Empfehlung: Release Please im Manifest-Modus.** Die bisherige Eigenimplementierung in PR #46 soll vor einem Merge ersetzt werden. Versionsberechnung, Dateiaktualisierung, Changelog, Release-PR und Release-Verwaltung werden von dem etablierten Werkzeug übernommen. Eigene Logik bleibt auf projektspezifische Grenzen beschränkt: bewusster Start, QA-Nachweis für den veröffentlichten Commit, erste Abnahme #43 sowie Reviewer-/Issue-Zuordnung.

## Vergleich

| Werkzeug | Passung | Entscheidung |
| --- | --- | --- |
| [paulhatch/semantic-version](https://github.com/marketplace/actions/git-semantic-version) | Ermittelt Versionen aus Tags und Commit-Nachrichten; unterstützt Komponentenpfade und Regeln vor 1.0. Übernimmt nicht unseren gesamten Versions-PR-/Datei-/Release-Ablauf. | Für reine Versionsberechnung geeignet, hier allein zu wenig. |
| [Release Please](https://github.com/googleapis/release-please) | Manifest-Modus für mehrere Komponenten; Rust- und Node-Strategien, Versions-PRs, Changelogs und GitHub Releases. Die Rust-Strategie aktualisiert Cargo.toml und Cargo.lock. | Beste gemeinsame Grundlage für Backend und späteres Frontend. |
| [Release-plz](https://release-plz.dev/docs/config) | Rust-spezifischer Release-PR-/Changelog-Ablauf; Git-only und deaktivierbares Cargo-Publishing sind möglich. Das Rust-Werkzeug deckt das spätere Node-Frontend nicht mit derselben Sprachstrategie ab. | Gute Rust-Alternative, für unser gemischtes Monorepo weniger passend. |
| Bisheriges eigenes Python-Skript | Deckt unsere projektspezifischen Regeln ab, implementiert aber zusätzlich Standardaufgaben wie Versionsänderungen, Changelog und GitHub-Release-Verwaltung selbst. | Unnötige dauerhafte Wartung; nicht als endgültige Lösung übernehmen. |

## Technischer Nachweis

Release Please 17.6.0 wurde ausschließlich unter `/tmp` installiert und anschließend in einem Node-24-Container **ohne Netzwerk und ohne Zugangsdaten** ausgeführt. Das ist die Bibliotheksversion, auf der Release Please Action v5.0.0 aufbaut. Für die Action wurde der feste Commit `45996ed1f6d02564a971a2fa1b5860e934307cf7` ermittelt.

Der Offline-Nachweis prüft mit der Rust-Strategie und den tatsächlichen Cargo-Dateien dieses Repositorys:

- Erster Release ohne Vorgänger: `0.1.0`.
- Bei `0.1.0`: kompatibles `feat:` ergibt `0.1.1`; `feat!:` ergibt `0.2.0`.
- Ab `1.0.0`: `feat!:` ergibt `2.0.0`.
- Komponententag mit Separator `/`: `backend/v0.1.1`.
- Cargo.toml, Cargo.lock und CHANGELOG.md werden von den vorgesehenen Updatern bearbeitet. Beide Cargo-Versionen stimmen überein; die übrigen Lockfile-Pakete sind unverändert.

Dafür sind die Standardoptionen `bump-minor-pre-major`, `bump-patch-for-minor-pre-major`, `include-component-in-tag` und `tag-separator` relevant. Später wird `frontend` als separate `node`-Komponente ergänzt; noch kein leeres Frontend erzeugen.

## Konsequenzen für die Umstellung

- Versionsvorschläge folgen Conventional Commits; die bisherige manuelle Auswahl des Sprungs wird durch einen bewusst gestarteten Vorbereitungsablauf und Review des berechneten Vorschlags ersetzt. Bewusste Ausnahmen über die dokumentierte Release-As-Konvention prüfen, statt einen zweiten Versionsrechner zu behalten.
- Das Release-Please-Manifest ist zusätzliche Versionshistorie und muss im Versions-PR konsistent mit den Komponentenmanifesten bleiben.
- Die Action erlaubt getrennte Aufrufe für PR-Erstellung und Veröffentlichung. Die projektspezifischen QA-/Abnahmebedingungen müssen vor Veröffentlichung weiterhin geprüft werden; ein ungeprüfter Standardworkflow bei jedem Push erfüllt #44 nicht vollständig.
- GITHUB_TOKEN-Beschränkungen bleiben bestehen. Erforderliche Folgeprüfungen ausdrücklich anstoßen oder eine bewusst eingerichtete App-Identität verwenden. Keine zusätzliche Identität oder Berechtigung wurde eingerichtet.
- Reviewer `Skanky0815`, Verantwortlicher `Sara-Pixel42` und Issue-Bezug bleiben verbindlich. Nicht annehmen, dass das Release-Werkzeug diese projektspezifischen Zuordnungen automatisch kennt.

Der Offline-Nachweis bestätigt die Kernfunktionen, ersetzt aber keinen echten GitHub-End-to-End-Test. Dieser bleibt nach Umstellung, Review/Merge und Freigabe der benötigten Repository-Einstellung Teil von #44. Es wurden keine Tags oder Releases erstellt.

## Primärquellen

- [Release Please: Manifest-Konfiguration](https://github.com/googleapis/release-please/blob/main/docs/manifest-releaser.md)
- [Release Please: Rust-/Node-Strategien](https://github.com/googleapis/release-please/blob/main/docs/customizing.md)
- [Rust-Strategie einschließlich Cargo.lock-Updater](https://github.com/googleapis/release-please/blob/main/src/strategies/rust.ts)
- [Release Please Action: Ein-/Ausgaben und Token-Verhalten](https://github.com/googleapis/release-please-action)
- [GitHub: Auslöser mit GITHUB_TOKEN](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)
