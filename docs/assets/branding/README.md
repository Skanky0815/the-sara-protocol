# Projektauftritt und Bildherkunft

Stand: 2026-10-31, Issue [#45](https://github.com/Skanky0815/the-sara-protocol/issues/45).

## Eigene Projektmotive

| Datei | Format / Größe | Verwendung |
| --- | --- | --- |
| [banner.png](banner.png) | PNG, 2172 × 724, dunkler Hintergrund | README-Kopf, Seitenverhältnis 3:1 |
| [logo.png](logo.png) | PNG mit Alphakanal, 1254 × 1254 | kompaktes Projektzeichen, in der README mit 48 × 48 dargestellt |

Rico hat die Richtung „dunkles Terminal mit Pixel-Katze und Kaffee“ und den Bannerentwurf im gemeinsamen Chat bestätigt. Das Logo wurde daraus abgeleitet. Die finalen Dateien werden zusammen mit der README im normalen PR-Review geprüft.

Beide Bilder wurden mit dem integrierten OpenAI-Imagegen-Werkzeug im Projektauftrag erzeugt; keine menschlich gezeichneten oder offiziellen Techniklogos. Für das Banner wurde kein fremdes Referenzbild verwendet, für das Logo ausschließlich das hier abgelegte Banner. Die vollständigen Eingaben stehen in [banner-prompt.txt](banner-prompt.txt) und [logo-prompt.txt](logo-prompt.txt). Generierte Originaldateien wurden unverändert übernommen; das Logo besitzt echte Transparenz. Es wird kein bestimmtes Modell behauptet, da die Werkzeugantwort keinen Modellnamen ausweist.

Nutzungsgrundlage sind die für den verwendeten OpenAI-Dienst geltenden Bedingungen, für EWR-Nutzer insbesondere die [europäischen Nutzungsbedingungen](https://openai.com/policies/eu-terms-of-use/). Die Bilder werden als KI-generiert gekennzeichnet. Daraus wird keine Exklusivität oder Garantie einer markenrechtlichen Schutzfähigkeit abgeleitet. Diese Dokumentation erteilt keine zusätzliche Lizenz für das gesamte Repository.

## Technikzeichen und Badges

Die Technikzeichen sind von den Projektmotiven getrennt und werden über [Shields.io](https://shields.io/docs/logos) eingebunden. Shields verwendet die Slugs `rust`, `postgresql`, `nodedotjs` und `nextdotjs` aus [Simple Icons](https://github.com/simple-icons/simple-icons). Die Icon-Sammlung steht unter [CC0 1.0](https://github.com/simple-icons/simple-icons/blob/develop/LICENSE.md); Markenrechte bleiben davon unberührt, siehe auch den [Hinweis von Simple Icons](https://github.com/simple-icons/simple-icons#disclaimer). Keine heruntergeladene Schrift und kein fremdes Techniklogo ist Bestandteil der generierten Projektbilder. Die Zeichen dienen der sachlichen Benennung verwendeter bzw. geplanter Technik, nicht einer Partnerschafts- oder Unterstützungsbehauptung.

- **Rust / PostgreSQL:** implementiertes Backend bzw. Datenbank.
- **Node.js:** ausschließlich derzeitige Release-Werkzeuge, keine Node.js-Backend-Laufzeit.
- **Next.js:** ausdrücklich geplantes Frontend, noch keine Implementierung oder veröffentlichte Frontend-Version.
- **Rust-QA:** dynamisches Badge des tatsächlich vorhandenen Workflows `rust-qa.yml`, eingeschränkt auf Pushes nach `main`.
- **Backend-Version:** dynamisches GitHub-Release-Badge, gefiltert auf `backend/v*`; Frontend-Tags können diesen Wert nicht ersetzen.

Die README verwendet relative Bildpfade, Alternativtexte und frei umbrechende Badge-Zeilen. Das Banner hat seinen eigenen dunklen Hintergrund, das Logo einen hellen Rand für helle und dunkle Ansichten. Externe Badges benötigen GitHub/Shields-Verfügbarkeit; bei einem Ausfall bleiben Bild-Alternativtexte, Links und die textliche Technikzuordnung vorhanden.

## Darstellung geprüft

Die README wurde mit GitHubs Markdown-API im GFM-Modus gerendert. Banner, Logo, Alttexte und relative Verweise sind vorhanden; alle sechs externen Badge-URLs liefern SVGs, das Release-Badge enthält die reale Version 0.1.0 und die Technik-Badges ihre Logo-Pfade. Eine lokale Vorschau dieses GitHub-HTMLs wurde mit heller und dunkler Gestaltung sowie einer 390-Pixel-Lesespalte visuell geprüft: alle acht Bilder geladen, Badges umbrechend, kein horizontaler Überlauf der Lesespalte. Die Vorschau-CSS ist nur ein Prüfmittel und wird nicht ins Repository übernommen.
