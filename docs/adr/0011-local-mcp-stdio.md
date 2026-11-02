# ADR 0011: Lokaler, lesender MCP-Pilot über stdio

- Datum: 2026-11-02
- Status: Vorgeschlagen
- An der Entscheidung beteiligt: Rico (lokale Mac-App-Anbindung beauftragt), Sara (Umsetzung)
- Dokumentiert von: Sara (KI-Assistentin)

## Kontext

Rico möchte den Persona-Service zunächst lokal als Werkzeug in der ChatGPT-App auf dem Mac nutzen. Das ist Client-Integration aus [#17](https://github.com/Skanky0815/the-sara-protocol/issues/17), kein ausgehender LLM-Adapter aus [#14](https://github.com/Skanky0815/the-sara-protocol/issues/14). Die vorhandene REST-API kann Personas lesen und Memories suchen; sie speichert bisher keine vollständige Charakterdefinition.

## Entscheidung

Die bestehende Rust-Binary erhält den separaten Startmodus `--mcp`. Das offizielle Rust-SDK `rmcp 3.8.0` übernimmt Protokoll, Initialisierung und stdio-Transport. Der Prozess öffnet keinen HTTP-Port; die Desktop-App startet ihn als Kindprozess. `MCP_PERSONA_ID` ist eine positive, serverseitig festgelegte ID und kein Toolparameter. Vor dem Handshake wird ihre Existenz geprüft. Es gibt zunächst ausschließlich `get_persona` und `search_memories`, beide als lesend annotiert. Unbekannte oder zusätzliche Parameter werden abgelehnt. Gespeicherte Inhalte bleiben Daten, keine ausführbaren Anweisungen.

Die Tools verwenden intern feste GET-Requests auf denselben Axum-Router wie REST (`tower::ServiceExt`), ohne Netzwerk-Rundweg. Das vermeidet doppelte SQL-/Validierungslogik und benötigt keine umfassende Refaktorierung. `form_urlencoded` kodiert den Suchtext. Aufrufe sind auf 15 Sekunden begrenzt; Ergebnisse oberhalb eines MiB REST-Body werden als Fehler zurückgegeben statt unvollständige Provenienz zu liefern. Die Suche erlaubt 1–20 Treffer, Standard 5. Erfolgreiche MCP-Ergebnisse enthalten sowohl strukturiertes JSON als auch den kompatiblen Textblock des SDK.

Der MCP-Modus lädt ausschließlich `.env.mcp`, niemals automatisch die HTTP-Datei `.env`. Exportierte Werte haben Vorrang. Die Beispieldatei enthält eine absichtlich ungültige ID, um eine unbewusste Auswahl der ersten Persona zu verhindern. Der lokale Launcher setzt ein festes Arbeitsverzeichnis. Anwendungslogs gehen als bereinigtes JSON nach stderr, stdout bleibt MCP vorbehalten. SDK-/Treiberdiagnosen werden wie bisher vom Anwendungsfilter ausgeschlossen.

## Alternativen

Streamable HTTP über dasselbe SDK wäre passend für mehrere Prozesse oder einen späteren Remote-Client, benötigt aber zusätzlich Zugriffsschutz, Origin-Prüfung und sichere Erreichbarkeit. Ein Tunnel ist für diesen direkten Desktop-stdio-Piloten unnötig. Das alternative Rust-MCP-SDK wäre ebenfalls möglich; die offizielle Implementierung deckt stdio und einen späteren HTTP-Transport bereits ab. Ein eigener JSON-RPC-Stack hätte zusätzliche Wartung für Verhandlung und Protokolländerungen. Ein eigener Modelladapter ist für ChatGPT-Toolzugriffe nicht nötig und bleibt getrennte Arbeit.

## Konsequenzen

Eine eigene Pilotdatenbank und ausschließlich synthetische Daten sind für die erste Abnahme vorgesehen. Die Prozesskonfiguration ist eine Vertrauensgrenze: Wer sie ändern kann, kann eine andere Persona wählen. Es gibt keine Mehrbenutzer-Authentifizierung und keinen öffentlichen MCP-Endpunkt. Die reguläre REST-API behält ihren bestehenden Funktionsumfang; ihre zusätzlichen Schreibmöglichkeiten werden nicht als MCP-Tools angeboten.

Die vorhandenen Laufzeitmigrationen laufen auch vor dem MCP-Start. „Lesend“ beschreibt die Tool-Aufrufe, nicht den Schemaaufbau beim Start. Der Ergebnis-Body wird vom SDK für strukturiertes JSON und Text verdoppelt; ein MiB ist die Grenze der zugrunde liegenden REST-Antwort, kein Tokenbudget. Kleine Limits und gezielte Suchtexte bleiben nötig.

Tests belegen echte SDK-Initialisierung, Tool-Erkennung, Parameterprüfung, Persona-Isolation, Provenienz, Prozessneustart und die Trennung von Logs und Protokoll. Das beweist noch keine Verbindung zur installierten ChatGPT-App. Verfügbarkeit der lokalen MCP-Einstellungen und ein echter Toolaufruf in dieser App müssen gesondert verifiziert werden. Ein Schreibtool mit ausdrücklicher Freigabe und der gesprächsübergreifende End-to-End-Nachweis aus #17 bleiben offen; der Implementierungs-PR verwendet deshalb `Refs #17`.

Quellen: [offizielles Rust-SDK](https://github.com/modelcontextprotocol/rust-sdk), [MCP in ChatGPT Desktop und Codex](https://learn.chatgpt.com/docs/extend/mcp). Die App-Dokumentation beschreibt lokale stdio- und Streamable-HTTP-Konfiguration; unterstützte Oberflächen und Workspace-Regeln können sich unterscheiden.
