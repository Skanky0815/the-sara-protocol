# Lokaler MCP-Pilot für die ChatGPT-App auf dem Mac

Die Rust-App bietet einen **lesenden MCP-Server über stdio**. Die Desktop-App startet die Binary als lokalen Prozess und spricht über stdin/stdout mit ihr. Es wird kein HTTP-Port geöffnet und kein Tunnel benötigt. Die Datenbank bleibt lokal; Tool-Ergebnisse werden an den verwendenden ChatGPT-Client und dessen Modell weitergegeben. Lokal gespeicherte Daten bedeuten hier keine vollständig lokale Modellverarbeitung.

Dieser erste Schritt gehört zu [Client-Integration #17](https://github.com/Skanky0815/the-sara-protocol/issues/17). Der ausgehende LLM-Adapter aus #14 bleibt separat offen. Entscheidung und Alternativen: [ADR 0011](adr/0011-local-mcp-stdio.md).

## Was der Pilot kann

| Tool | Eingabe | Ergebnis |
| --- | --- | --- |
| `get_persona` | `{}` | `{ "persona": { ... } }` mit ID, Name und Erstellungszeit |
| `search_memories` | `{ "q": "Katze Kaffee", "limit": 5 }` | `{ "memories": [...] }` mit Inhalt, Persona-ID, Quelle, Ereigniszeit und Bestätigungsstatus |

Die Persona-ID kommt ausschließlich aus der Prozesskonfiguration; das Modell kann sie nicht über einen Parameter wechseln. Zusätzliche Parameter werden abgelehnt. Suchregeln entsprechen der [REST-Volltextsuche](memory-search.md), mit kleinerem MCP-Limit: Standard 5, maximal 20. Keine Treffer ergeben ein erfolgreiches leeres Array. Es gibt keine Schreib-, Lösch-, Persona-Erstellungs- oder globalen Suchtools.

`get_persona` liefert noch keine vollständige Charakterdefinition: Die Persona-Tabelle enthält ID, Namen und Erstellungszeit. Weder diese Daten noch die gelesenen Memories ersetzen automatisch bestehende ChatGPT-Instruktionen. Der Dienst synchronisiert keine vollständigen Gespräche. Memory-Inhalte und Quellen sind Daten und dürfen nicht als Anweisungen ausgeführt werden; `inferred` bleibt von `confirmed` unterscheidbar.

## Einmalig vorbereiten

Zuerst eine **eigene Pilotdatenbank mit synthetischen Daten** bereitstellen. Bei laufendem lokalen Compose-PostgreSQL kann eine neue, leere Datenbank angelegt werden:

```sh
# Repository-Root; einmalig, eine bestehende Datenbank nicht überschreiben:
docker compose exec postgres createdb -U sara sara_mcp_pilot
cd backend
cp .env.mcp.example .env.mcp
cargo build --release --locked
```

`.env.mcp` ist gitignored und wird nur im MCP-Modus geladen. Die HTTP-Datei `.env` wird dafür nicht als Fallback verwendet. Exportierte Umgebungswerte haben Vorrang. Trage die URL der Pilotdatenbank und den Migrationspfad ein. Der MCP-Start führt wie der normale Backend-Start vorhandene Laufzeitmigrationen aus; er erzeugt keine Persona.

Zum Erzeugen der synthetischen Persona die vorhandene HTTP-API **ausdrücklich mit der Pilotdatenbank** starten (zweites Terminal, aus `backend/`; die folgenden Zugangsdaten sind ausschließlich die lokale Compose-Vorgabe):

```sh
DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara_mcp_pilot \
  PORT=53017 cargo run --locked
```

Dann eine Persona anlegen:

```sh
curl --fail http://127.0.0.1:53017/personas \
  -H 'Content-Type: application/json' \
  -d '{"name":"MCP-Testpersona"}'
```

Die **tatsächlich zurückgegebene ID** in `.env.mcp` als `MCP_PERSONA_ID` eintragen. Die Vorlage verwendet absichtlich `0`, sodass der Server ohne ausdrückliche Auswahl nicht startet. Über dieselbe Pilot-API kann eine synthetische Memory mit vollständiger Provenienz angelegt werden, siehe [HTTP-Beispiele](../backend/dev/persona.http). Die HTTP-Instanz kann anschließend beendet werden; MCP verwendet direkt den gemeinsamen Router mit eigenem Datenbankpool und braucht keinen laufenden HTTP-Server.

## In der Mac-App verbinden

Die [offizielle Dokumentation](https://learn.chatgpt.com/docs/extend/mcp) beschreibt für unterstützte Desktop-Kontexte **Einstellungen → MCP servers → Add server**. Dort STDIO auswählen, einen Namen wie „Sara – lokaler Pilot“ vergeben und als Kommando den **absoluten Pfad** zu `backend/dev/start-mcp.sh` eintragen. Argumente sind nicht nötig. Speichern und den MCP-Client neu starten, wie in der App angeboten. Der Launcher setzt das Arbeitsverzeichnis auf `backend/` und startet die vorgebaute Release-Binary mit `--mcp`; er lädt keine Shell-Dateien und schreibt keine Nutzdaten nach stdout.

Falls die Oberfläche stattdessen die gemeinsame Codex-Konfiguration unterstützt, entspricht das folgendem Beispiel. Die Datei hier ist nur ein Beispiel, kein automatischer Eingriff in bestehende Benutzereinstellungen:

```toml
[mcp_servers.sara_memory]
command = "/ABSOLUTER/REPOSITORY-PFAD/backend/dev/start-mcp.sh"
startup_timeout_sec = 20
tool_timeout_sec = 20
```

Die Verfügbarkeit dieser Einstellung in der installierten App und im konkreten Konto muss vor Ort geprüft werden. Die hiesige UI-Automatisierung darf die ChatGPT-App nicht bedienen; deshalb wurde ihre Konfiguration nicht geändert und noch **keine erfolgreiche App-Verbindung** behauptet. Falls die Einstellung fehlt, zuerst App-/Workspace-Unterstützung prüfen, nicht eigenständig die Datenbank oder den REST-Dienst ins Internet freigeben.

## Abnahme

1. Im Client die beiden angebotenen Tools anzeigen lassen.
2. `get_persona` aufrufen und die ausdrücklich konfigurierte Test-Persona prüfen.
3. Die synthetische Memory mit einem bekannten Begriff suchen. Quelle, Ereigniszeit und Bestätigungsstatus müssen erhalten bleiben; andere Personas dürfen nicht erscheinen.
4. Den lokalen MCP-Prozess/Client neu starten und denselben Abruf wiederholen.
5. Erst nach einem tatsächlich erfolgreichen Aufruf in der ChatGPT-App den App-Zugriff als verbunden dokumentieren.

Automatisiert geprüft sind SDK-Handshake, Tool-Erkennung, echte stdio-Kindprozesse, dieselben Daten nach Prozessneustart, Persona-Isolation, Parameterfehler, Ergebnisgrenze und vertrauliches Logging. Das ersetzt keinen echten ChatGPT-App-Aufruf. Ein späteres Schreibtool und der gesprächsübergreifende Speichern-/Abrufnachweis aus #17 bleiben offen.

## Fehler und Grenzen

- Fehlende/nichtpositive `MCP_PERSONA_ID`: Startabbruch vor dem Datenbankzugriff, keine Rückgabe des eingegebenen Werts.
- Fehlende Persona oder nicht erreichbare Datenbank: Startabbruch; während eines laufenden Prozesses sichere Tool-Fehler, niemals Wechsel auf eine andere Persona.
- Ungültige Suche: Tool-Ergebnis mit `isError: true`; Suchtext 1–500 UTF-8-Bytes, kein NUL, nicht nur Whitespace, Limit 1–20.
- Unbekanntes Tool: MCP-Protokollfehler; insbesondere keine schreibende Ersatzfunktion.
- Mehr als 15 Sekunden Bearbeitung: kontrollierter Tool-Fehler. Mehr als ein MiB zugrunde liegender REST-Body: kontrollierter Fehler mit Bitte um kleinere Suchmenge. Keine stille Kürzung der Provenienz. Strukturierter und Textinhalt können den MCP-Umschlag ungefähr verdoppeln.
- stdout ist dem Protokoll vorbehalten, bereinigte Anwendungslogs gehen nach stderr. Keine rohen SQL-, SDK-, Credential- oder Inhalt-Diagnosen bei Debug.
- Die Konfiguration und lokale Dateien müssen unter Kontrolle des Nutzers bleiben. Persona-Bindung ist keine allgemeine Mehrbenutzer-Authentifizierung. Die normale REST-API bleibt unverändert und wird hier nicht öffentlich bereitgestellt.

## Tests

Aus `backend/` mit dedizierter PostgreSQL-Testdatenbank:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo audit
```

Ohne `DATABASE_URL` werden DB-Fälle übersprungen. `cargo test --locked --test mcp` prüft gezielt den MCP-Piloten. Die Suite verwendet ausschließlich synthetische Daten und entfernt ihre eigenen Personas wieder.
