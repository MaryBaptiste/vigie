# Vigie

Suivi d'objets en orbite (ISS, Hubble, Tiangong, Landsat 9) au look TUI rétro, façon écran de station sol.
Application desktop **Tauri 2** (Rust) + **Vue 3 / TypeScript / SCSS** + **Three.js**.
Les positions sont calculées côté Rust avec SGP4, à partir des éléments GP publiés par [CelesTrak](https://celestrak.org).

## Prérequis

- Node.js ≥ 20 et npm
- Rust stable (≥ 1.88, édition 2024) avec `clippy`
- Prérequis système Tauri : <https://tauri.app/start/prerequisites/> (WebView2 sous Windows, `webkit2gtk-4.1` sous Linux / Raspberry Pi OS)

Pas d'OpenSSL : `reqwest` utilise `rustls`. La police VT323 est embarquée dans le bundle, donc l'interface fonctionne hors ligne. Il faut toutefois le réseau pour récupérer les TLE.

## Commandes

```sh
npm install
npm run tauri dev       # lancement en développement
npm run tauri build     # binaire + installeur dans src-tauri/target/release/bundle

# Vérifications
npx vue-tsc --noEmit
cd src-tauri
cargo clippy --all-targets -- -D warnings
cargo test                  # tests unitaires (hors ligne)
cargo test -- --ignored     # vérifie les numéros NORAD du catalogue sur CelesTrak (réseau)
```

## Architecture

### Backend (`src-tauri/src/`)

Les couches sont strictes, et chacune ne connaît que celle d'en dessous :

```
services/   #[tauri::command] : aucune logique, délègue à une facade
   │
facades/    logique métier, injectées via app.manage()
   │          SpaceObjectFacade  catalogue des objets suivis
   │          TelemetryFacade    cache TLE (12 h), propagation SGP4, calculs dérivés, trajectoire
   │          SolarFacade        point subsolaire et terminateur jour/nuit
   │          orbit_math         fonctions pures : GMST, TEME → ECEF → géodésique WGS84, apogée/périgée
   │          solar_math         fonctions pures : position du Soleil (Astronomical Almanac), terminateur
   ├── mappers/   fonctions pures, modèle interne → DTO (SpaceObject-, Telemetry-, TrajectoryMapper)
   ├── clients/   HTTP CelesTrak + désérialisation brute (OMM JSON → sgp4::Elements)
   └── models/    modèles internes (SpaceObject, OrbitalState, Trajectory, GeodeticPosition)
dto/        structures sérialisées vers le front (camelCase)
error.rs    AppError (thiserror), sérialisé en { kind, message }
```

Commandes exposées :
- `get_space_objects`
- `get_telemetry(noradId)` : interrogée chaque seconde par le front.
- `get_trajectory(noradId)` : positions sur −½ à +1 période autour de l'instant courant, un point toutes les 30 s. Interrogée toutes les 30 s.
- `get_solar_state` : point subsolaire et terminateur, tous les 1° de longitude, avec la phase lever ou coucher. Interrogée chaque minute.

Comportement réseau de `TelemetryFacade` :
- Les éléments de chaque objet sont mis en cache 12 h.
- Si CelesTrak est injoignable, la facade réutilise les derniers éléments connus. Le front affiche alors `DATA STALE` dès que le TLE a plus de 72 h.
- Après un échec, aucune nouvelle requête n'est envoyée pendant 30 s, pour ne pas solliciter CelesTrak à chaque polling.

### Front (`src/`)

```
views/MainView.vue     mise en page : globe (2/3) | objets + télémétrie (1/3)
components/            TuiPanel (cadre), DataRow (ligne alignée), ViewTabs (onglets),
                       EarthGlobe (Three.js), WorldMap (SVG), ObjectList, TelemetryPanel
composables/           usePolling (générique), useSpaceObjects (liste + sélection),
                       useTelemetry (1 s), useTrajectory (30 s), useSolarState (60 s)
assets/geo/            côtes Natural Earth 1:110m (domaine public), compactées en [lon, lat]
services/              wrappers typés : seul invokeCommand.ts appelle invoke()
types/                 miroirs TypeScript des DTO Rust
utils/                 formatage à largeur fixe, lat/lon → scène 3D, projection équirectangulaire
                       (découpage à l'antiméridien), découpage passé/futur de la trajectoire
styles/                _variables.scss (palettes), _mixins.scss, main.scss (reset, scanlines)
```

Règles du front :
- TypeScript est en mode strict, sans `any`.
- Les composants n'appellent jamais `invoke()` directement.
- Les couleurs viennent uniquement de `_variables.scss`.

### Globe

Le globe affiche, sur une sphère fil de fer :
- le graticule (tous les 15°) et les côtes ;
- la trajectoire de l'objet sélectionné : passé atténué, futur en pleine intensité, partie cachée derrière la Terre en transparence ;
- un marqueur à la position courante, relié au sol par sa ligne de nadir.

Tout est exprimé en repère terrestre, ce qui garde la trajectoire cohérente avec les continents. La rotation du globe est décorative (un tour par minute).

### Planisphère

C'est une projection équirectangulaire en SVG. Elle affiche :
- les côtes et une grille tous les 30° ;
- la trajectoire passée et future de l'objet ;
- la position de l'objet (réticule clignotant) ;
- le point subsolaire ;
- le terminateur : lever du soleil en trait plein dans la couleur `sun`, coucher en pointillés dans la couleur `sun-dim` ;
- la zone de nuit, en ombré.

Le globe et le planisphère se partagent la zone de gauche. On bascule de l'un à l'autre avec les onglets ou la touche `V`. Quand le globe est masqué, il est démonté et sa scène WebGL est libérée.

### Thème

Trois palettes sont disponibles : `orange` (par défaut), `amber` et `green`. Pour en changer :
- à la compilation, modifier `$phosphor-theme` dans `src/styles/_variables.scss` ;
- à l'exécution, ajouter `data-phosphor="green"` sur `<html>`. Les couleurs du globe sont lues au montage.

### Clavier

- `↑` / `↓` déplacent le curseur dans la liste des objets.
- `Entrée` sélectionne l'objet sous le curseur.
- Un clic sur un objet le sélectionne aussi.
- `V` bascule entre le globe et le planisphère.
