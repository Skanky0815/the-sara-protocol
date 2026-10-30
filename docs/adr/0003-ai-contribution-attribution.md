# ADR 0003: Kennzeichnung von KI-Beiträgen

- Datum: 2026-10-27
- Status: Vorgeschlagen
- Vorgeschlagen von: Rico und Sara (KI-Assistentin)
- Dokumentiert von: Sara (KI-Assistentin)

## Kontext
Menschlich verfasste und KI-unterstützte Änderungen sollen unterscheidbar sein, einschließlich Fehlern und gelungenen Beiträgen. Git-Autor, Committer, GitHub-Konto und tatsächlicher Beitragender sind nicht zwangsläufig identisch.

## Vorgeschlagene Entscheidung
- Von KI verfasste Änderungen gegebenenfalls mit `AI-assisted-by: Sara (AI)` in Commit-Nachrichten kennzeichnen.
- Bei gemeinsamer Arbeit menschliches Review und KI-Beitrag in PR-Beschreibungen und ADRs zutreffend beschreiben.
- Git-Autor-/Committer-Identitäten wahrheitsgemäß angeben; nicht vorgeben, ein Commit des GitHub-Connectors stamme von einem separaten Sara-Konto.
- Eine eigene Bot-Identität später erwägen, sofern technisch unterstützt und genehmigt.

## Alternativen
Ein separates GitHub-Bot-Konto; Git-Trailer wie `Co-authored-by` mit verifizierter Identität; ausschließlich PR-Labels verwenden.

## Konsequenzen
Klarere Zuordnung ohne Umschreiben der Historie. Commit-Trailer allein garantieren weder zeilenweise Verantwortlichkeit noch eine zuverlässige Zuordnung zum GitHub-Profil.

## Offene Frage
Die genaue Commit-Identität und Trailer-Konvention in lokalen Codex- und GitHub-Connector-Abläufen vor Annahme dieses ADR bestätigen.

## Aktueller Hinweis zur Anwendung
Der obige Vorschlag bleibt als historischer Entwurf erhalten. Die inzwischen in [AGENTS.md](../../AGENTS.md) dokumentierte Vorgabe von Rico hat Vorrang: standardmäßig keine Attribution-Trailer in Commit-Nachrichten. Diese Übersetzung ändert keine Entscheidung oder Berechtigung.
