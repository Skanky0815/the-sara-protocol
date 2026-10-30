# Personas lokal verwenden

Die verbindliche Beschreibung der Request-/Response-Schemas, Parameter, Statuscodes und Header kommt aus der [generierten OpenAPI-Referenz](api-reference.md). Diese Anleitung erklaert den Ablauf und die fachlichen Regeln.

## Persona anlegen und abrufen

Eine Persona hat eine stabile, von PostgreSQL erzeugte ID, einen eindeutigen Namen und einen Erstellungszeitpunkt. Das vorhandene Schema bleibt erhalten; für die API wurde keine Migration benötigt.

```http
POST /personas
Content-Type: application/json

{"name":"Example persona"}
```

Die erfolgreiche Antwort enthält die neue ID und einen `Location`-Header für den anschließenden Abruf, beispielsweise `GET /personas/42`. Namen werden vor der Validierung außen getrimmt; danach sind 1 bis 200 Unicode-Zeichen ohne Steuerzeichen erlaubt. Die Datenbank verhindert atomar doppelte Namen, auch bei gleichzeitigen Anfragen. Groß-/Kleinschreibung bleibt wie im bestehenden Schema relevant; zusätzliche Unicode-Normalisierung findet nicht statt.

IDs und Erstellungszeitpunkte können beim Anlegen nicht vorgegeben werden; unbekannte JSON-Felder werden abgelehnt. Bereits vorhandene Datensätze bleiben abrufbar. Die Persona-Antwort enthält keine anderen Personas und keine eingebetteten Memories. Memories werden separat unter `/personas/{id}/memories` gespeichert und abgerufen.

## Isolation und Logging

Persona- und Memory-Zuordnung erfolgen über die stabile ID; SQL-Eingaben sind parametrisiert. Namen oder vollständige Bodies werden nicht protokolliert. Beide Persona-Endpunkte verwenden das bestehende sichere Request-Tracing. Namenskonflikte erzeugen eine kontrollierte Warnung, andere Datenbankfehler freigegebene Fehlerkategorien; interne SQL-/Treiberdetails gelangen nicht an Clients.

Datenisolation ersetzt keine Benutzer-Authentifizierung oder Zugriffsautorisierung. Diese bleiben separate Architekturaufgaben; der lokale Dienst kennt derzeit keine Eigentümer- oder Clientrechte.

## Ausprobieren und testen

Vorbereitete Requests stehen in [`backend/dev/persona.http`](../backend/dev/persona.http). Zuerst eine Persona anlegen, die zurückgegebene ID in `@personaId` übernehmen, dann Persona und Memories abrufen. Bei erneutem Anlegen einen neuen Namen verwenden oder die bestehende Persona per ID nutzen.

```sh
# Aus backend/ mit dedizierter Testdatenbank:
DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara_test cargo test --locked --test persona_api
```

Die Persona-Tests liegen nach Verantwortung getrennt unter `backend/tests/personas/`. Jeder Datenbanktest verwendet eigene Namen und entfernt seine Daten nach erfolgreicher Prüfung. Für den vollständigen Nachweis muss `DATABASE_URL` gesetzt sein. Ergänzende Vertragstests validieren die echten Antworten gegen die generierten Schemas; siehe [API-Referenz](api-reference.md).
