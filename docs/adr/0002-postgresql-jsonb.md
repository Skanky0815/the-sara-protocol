# ADR 0002: PostgreSQL mit JSONB

- Datum: 2026-10-27 (nachträglich dokumentiert)
- Status: Akzeptiert
- An der Entscheidung beteiligt: Rico und Sara (KI-Assistentin)
- Dokumentiert von: Sara (KI-Assistentin)

## Kontext
Persona- und Erinnerungsdatensätze benötigen flexible, dokumentartige Felder sowie Beziehungen, Transaktionen und nachvollziehbare Herkunft.

## Entscheidung
PostgreSQL mit JSONB für das MVP verwenden; pgvector bleibt eine optionale spätere Ergänzung. Zunächst kein separater Suchdienst.

## Alternativen
MongoDB und CouchDB wurden diskutiert.

## Konsequenzen
Eine Datenbank kann relationale und flexible Daten abdecken. Schema und Migrationen müssen weiterhin sorgfältig entworfen werden; die Vektorsuche ist noch nicht implementiert.
