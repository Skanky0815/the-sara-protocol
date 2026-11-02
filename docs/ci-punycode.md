# CI-Wartung: punycode im Cleanup (#54)

## Befund

Der [Lauf 41193423442](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41193423442) („Prepare component release“, 31.10.2026) enthält im Job `prepare` um 10:39:23 UTC exakt die gemeldete Warnung `(node:2369) [DEP0040]`. Betroffen ist der Post-Schritt von `actions/setup-node@v4`, aufgelöst zu `49933ea5288caeca8642d1e84afbd3f7d6820020`. Der Lauf war erfolgreich. Auch der [Backend-delivery-Lauf 41194814514](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41194814514) zeigt die Warnung im setup-node-Cleanup.

Der offizielle [v4-Lockfile](https://github.com/actions/setup-node/blob/49933ea5288caeca8642d1e84afbd3f7d6820020/package-lock.json) enthält `@actions/cache 4.0.3`, `@azure/ms-rest-js 2.7.0` sowie `@azure/storage-blob 12.16.0` → `@azure/core-http 3.0.3`. Diese laden `node-fetch 2.7.0` → `whatwg-url 5.0.0` → `tr46 0.0.3`. Das gebündelte `dist/cache-save/index.js` referenziert daraus das eingebaute `require("punycode")` (ncc-Modul 4876, Zeile 88599). Schon das Laden des Post-Bundles erzeugt die Warnung, auch ohne aktiviertes Paketmanager-Caching.

## Entscheidung und fokussierter Nachweis

Die gepflegte offizielle [Version v7.1.0](https://github.com/actions/setup-node/releases/tag/v7.1.0) ersetzt v4 an allen drei setup-node-Einsatzstellen. Der SHA-Pin lautet `949feb2413d6458794dcd2491c4babbbce0c15c1`. Die Action läuft mit Node.js 24; die gewählte Node-Version bleibt ebenfalls 24. Die GitHub-gehosteten Runner erfüllen die erforderliche Runner-Version. Bestehende andere Pins und Berechtigungen bleiben erhalten.

Ein eigener Bundle-Patch wäre wartungsintensiv und unnötig. Globale Warnungsunterdrückung oder ein Downgrade beheben die Ursache nicht. Ein breites Upgrade anderer Actions gehört nicht zu dieser Änderung; Warnungen von Artefakt-Actions im Hauptschritt sind separat zu behandeln.

Lokaler Vergleich mit Node.js 24.22.1 und `node --trace-deprecation dist/cache-save/index.js`, ohne Token oder GitHub-Umgebungsdaten: v4 erzeugt DEP0040 mit Stack zu Modul 4876, v7.1.0 beendet denselben Aufruf erfolgreich ohne Deprecation-Warnung. Beide melden lediglich, dass kein Cache konfiguriert ist. Für den echten Cleanup-Nachweis dient der Workflow „Workflow validation“ auf dem PR-Commit, der dieselbe gepinnte Action verwendet. Release-Workflows werden dafür nicht manuell ausgelöst, da sie Veröffentlichungen verändern können.
