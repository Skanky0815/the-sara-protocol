# Persona-API

Das Backend bietet das Anlegen und gezielte Abrufen von Personas. Das vorhandene Datenbankschema bleibt unverändert; eine neue Migration ist nicht erforderlich.

## Datenmodell

| Feld | Typ | Bedeutung |
|---|---|---|
| `id` | positive 64-Bit-Ganzzahl | Von PostgreSQL erzeugte, stabile Persona-ID |
| `name` | String | Eindeutiger Name; beim Anlegen außen getrimmt, danach 1 bis 200 Unicode-Zeichen ohne Steuerzeichen |
| `created_at` | String | Von PostgreSQL gesetzter Erstellungszeitpunkt, als UTC-Zeitstempel mit Mikrosekunden und `Z` ausgegeben |

Die Namenseindeutigkeit wird atomar durch die vorhandene Datenbank-Constraint geprüft. Groß-/Kleinschreibung bleibt wie im bestehenden Schema relevant; es findet keine zusätzliche Unicode-Normalisierung statt. Bereits vorhandene Datensätze bleiben abrufbar, auch wenn sie vor Einführung der Eingabevalidierung angelegt wurden.

## Persona anlegen

```http
POST /personas
Content-Type: application/json

{"name":"Example persona"}
```

Ergebnis: HTTP `201 Created`, `Content-Type: application/json`, Header `Location: /personas/42` und beispielsweise:

```json
{"id":42,"name":"Example persona","created_at":"2026-10-29T12:00:00.000000Z"}
```

Nur `name` ist als Eingabefeld erlaubt. Clientseitig vorgegebene IDs und Erstellungszeitpunkte werden ebenso wie andere unbekannte Felder abgelehnt. Wiederholtes oder gleichzeitiges Anlegen desselben getrimmten Namens erzeugt keine zweite Persona; die kollidierende Anfrage erhält `409 Conflict`.

## Persona abrufen

```http
GET /personas/42
```

Ergebnis: HTTP `200 OK` mit genau den drei Feldern der angefragten Persona. Die SQL-Abfrage ist durch die ID eingeschränkt; sie enthält weder andere Personas noch eingebettete Memories. Memories bleiben über `POST` und `GET /personas/{id}/memories` erreichbar.

## Fehlerantworten

| Fall | HTTP-Status |
|---|---|
| Leerer Name nach Trimmen, mehr als 200 Zeichen oder enthaltene Steuerzeichen | `400 Bad Request` |
| Ungültiges JSON | `400 Bad Request` |
| Fehlendes `name`, falscher Datentyp, `null` oder unbekannte JSON-Felder | `422 Unprocessable Entity` |
| Fehlender bzw. nicht unterstützter JSON-Content-Type bei POST | `415 Unsupported Media Type` |
| Name bereits vorhanden | `409 Conflict` |
| ID nicht numerisch, außerhalb des 64-Bit-Bereichs oder nicht positiv | `400 Bad Request` |
| Positive, gültige ID existiert nicht | `404 Not Found` |
| Datenbank nicht verfügbar oder unerwarteter Datenbankfehler | `500 Internal Server Error` |

JSON- und Pfadvalidierung übernehmen die Axum-Extractors. Deren Fehlerantworten können Validierungstext enthalten. Fachliche Fehler und Datenbankfehler liefern einen leeren Body; interne SQL-/Treiberdetails werden nicht an Clients ausgegeben. Der bestehende Axum-Body-Limit bleibt aktiv.

## Isolation und Logging

Persona- und Memory-Zuordnung erfolgen über die stabile ID; Eingaben werden als SQL-Parameter gebunden. Namen oder vollständige Request-Bodies werden nicht protokolliert. Die Endpunkte verwenden das bestehende Request-Tracing mit den Vorlagen `/personas` und `/personas/{id}`. Namenskonflikte erzeugen eine kontrollierte Warnung, andere Datenbankfehler nur freigegebene Fehlerkategorien.

Die Datenisolation ersetzt keine Benutzer-Authentifizierung oder Zugriffsautorisierung. Diese bleiben separate Architekturaufgaben; der lokale Dienst kennt derzeit keine Eigentümer- oder Clientrechte.

## Lokal ausprobieren und testen

Vorbereitete Requests stehen in [`backend/dev/persona.http`](../backend/dev/persona.http). Zuerst die Persona anlegen und die zurückgegebene ID in `@personaId` übernehmen, dann Persona und Memories abrufen. Bei erneutem Anlegen einen neuen Namen verwenden oder die bestehende Persona per ID nutzen.

```sh
# Aus backend/ mit dedizierter, erreichbarer Testdatenbank:
DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara_test cargo test --locked --test persona_api
```

Die Tests liegen nach Verantwortung getrennt unter `backend/tests/personas/`: Anlage und Validierung, Abruf und Fehlerfälle sowie Isolation einschließlich der Memory-Endpunkte. Gemeinsame Request-/Datenbankhelfer liegen in `support.rs`. Jeder Datenbanktest verwendet eigene Namen und entfernt seine Daten nach erfolgreicher Prüfung. Ohne `DATABASE_URL` werden die Datenbankfälle übersprungen; für den vollständigen Nachweis muss die Variable gesetzt sein.

Die vollständige OpenAPI-Beschreibung folgt separat in Issue #12.
