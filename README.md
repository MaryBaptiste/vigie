# Vigie

Station sol au look TUI rétro, en trois écrans :

- **[F1] Orbite** (écran d'accueil) : suivi d'objets en orbite (ISS, Hubble, Tiangong, Landsat 9) sur un globe ou un planisphère, avec leur télémétrie. Les positions sont calculées côté Rust avec SGP4, à partir des éléments GP publiés par [CelesTrak](https://celestrak.org).
- **[F2] Ciel** : planétarium. La voûte céleste visible depuis n'importe quel point de la Terre (étoiles, constellations, Soleil, Lune, planètes et comètes brillantes) en projection stéréographique, une Terre 3D cliquable pour choisir le lieu d'observation, et le contrôle du temps simulé.
- **[F3] Objets** : les planètes, la Lune et les comètes, avec leur statut de visibilité, leurs heures de lever, de passage et de coucher, et une courbe de hauteur sur la nuit.

Application desktop **Tauri 2** (Rust) + **Vue 3 / TypeScript / SCSS** + **Three.js**. Tous les calculs (SGP4, astronomie) sont faits côté Rust.

## Prérequis

- Node.js ≥ 20 et npm
- Rust stable (≥ 1.88, édition 2024) avec `clippy`
- Prérequis système Tauri : <https://tauri.app/start/prerequisites/> (WebView2 sous Windows, `webkit2gtk-4.1` sous Linux / Raspberry Pi OS)
- Python ≥ 3.10, seulement pour régénérer les données embarquées du planétarium

Pas d'OpenSSL : `reqwest` utilise `rustls`. La police VT323 est embarquée dans le bundle, donc l'interface fonctionne hors ligne. Il faut toutefois le réseau pour récupérer les TLE. Le planétarium fonctionne entièrement hors ligne : seules les comètes ont besoin du réseau, et elles passent sur le cache disque, puis en `OFFLINE`, s'il manque.

## Commandes

```sh
npm install
npm run tauri dev       # lancement en développement
npm run tauri build     # binaire + installeur dans src-tauri/target/release/bundle

# Vérifications
npx vue-tsc --noEmit
cd src-tauri
cargo clippy --all-targets -- -D warnings
cargo test                  # tests unitaires et validation astronomique (hors ligne)
cargo test -- --ignored     # numéros NORAD sur CelesTrak et format de CometEls.txt sur le site du MPC (réseau)

# Régénération des données embarquées du planétarium
python -I tools/fetch_sources.py      # télécharge les sources brutes dans tools/raw/
python -I tools/build_catalogs.py     # écrit src-tauri/data/*.json et src-tauri/src/astronomy/vsop87_data.rs
```

## Architecture

### Backend (`src-tauri/src/`)

Les couches sont strictes, et chacune ne connaît que celle d'en dessous :

```
services/   #[tauri::command] : aucune logique, délègue à une facade
   │
facades/    logique métier, injectées via app.manage()
   │        ── Orbite
   │          SpaceObjectFacade    catalogue des objets suivis
   │          TelemetryFacade      cache TLE (12 h), propagation SGP4, calculs dérivés, trajectoire
   │          SolarFacade          point subsolaire et terminateur jour/nuit
   │          orbit_math           fonctions pures : GMST, TEME → ECEF → géodésique WGS84, apogée/périgée
   │          solar_math           fonctions pures : position du Soleil (Astronomical Almanac), terminateur
   │        ── Planétarium
   │          ObserverFacade       position de l'observateur (tauri-plugin-store, Paris par défaut)
   │          SkyFacade            état du ciel : étoiles, constellations, objets, état du ciel
   │          CelestialBodyFacade  fiches des objets, nuit de référence, événements
   │          CometCatalog         éléments MPC : cache disque, rafraîchis en tâche de fond tous les 7 jours
   │          ephemeris            enchaîne les calculs pour un lieu et un instant (Situation)
   ├── mappers/    fonctions pures, modèle interne → DTO (SpaceObject-, Telemetry-, Trajectory-, Solar-,
   │               Sky-, CelestialBody-, Observer-, CometCatalogMapper)
   ├── clients/    HTTP CelesTrak (OMM JSON → sgp4::Elements) et Minor Planet Center (CometEls.txt)
   ├── models/     modèles internes (SpaceObject, OrbitalState, Trajectory, Observer, Star, CelestialBody, Comet…)
   ├── astronomy/  calculs astronomiques purs et testés, sans état ni I/O
   └── catalogs/   chargement des données embarquées du planétarium (include_str!)
dto/        structures sérialisées vers le front (camelCase) ; statuts de visibilité en français
error.rs    AppError (thiserror), sérialisé en { kind, message }
```

Commandes exposées pour l'orbite :
- `get_space_objects`
- `get_telemetry(noradId)` : interrogée chaque seconde par le front.
- `get_trajectory(noradId)` : positions sur −½ à +1 période autour de l'instant courant, un point toutes les 30 s. Interrogée toutes les 30 s.
- `get_solar_state` : point subsolaire et terminateur, tous les 1° de longitude, avec la phase lever ou coucher. Interrogée chaque minute.

Commandes exposées pour le planétarium :

| Commande | Rôle |
|---|---|
| `get_observer()` / `set_observer(lat, lon, altitudeM, label)` | position persistée de l'observateur |
| `get_sky(datetimeUtc?)` | étoiles au-dessus de l'horizon, segments de constellation (ceux qui traversent l'horizon sont gardés), libellés au barycentre visible, Soleil, Lune, planètes et comètes de magnitude ≤ 12, état du ciel, temps sidéral local |
| `get_visible_bodies(datetimeUtc?)` | Lune, 7 planètes et comètes de magnitude ≤ 12, avec tous les champs de `CelestialBodyDto` |
| `get_comet_catalog_status()` | date des éléments MPC, source (`NETWORK`, `CACHE`, `OFFLINE` ou `LOADING`), et indicateur « vieux de plus de 14 jours » |

Comportement réseau de `TelemetryFacade` :
- Les éléments de chaque objet sont mis en cache 12 h.
- Si CelesTrak est injoignable, la facade réutilise les derniers éléments connus. Le front affiche alors `DATA STALE` dès que le TLE a plus de 72 h.
- Après un échec, aucune nouvelle requête n'est envoyée pendant 30 s, pour ne pas solliciter CelesTrak à chaque polling.

### Planétarium : calculs

Le module `astronomy/` contient :

| Module | Rôle |
|---|---|
| `time` | jour julien, ΔT (Espenak & Meeus 2006), TT ↔ UT |
| `sidereal` | temps sidéral moyen et apparent (Meeus 12), temps sidéral local |
| `nutation` | nutation et obliquité (Meeus 22, termes principaux) |
| `precession` | matrice de précession de Lieske (Meeus 21), toutes époques |
| `epoch` | tout ce qui ne dépend que de l'instant : matrices J2000 → vrai de la date, aberration annuelle, Terre |
| `coordinates` | conversion équatoriales → horizontales, réfraction (Sæmundsson), parallaxe topocentrique (Meeus 40) |
| `vsop87` | VSOP87D tronqué (séries générées dans `vsop87_data.rs`) |
| `sun`, `planets`, `moon` | positions apparentes, temps de lumière, magnitudes (Meeus 41), phases (Meeus 48), Lune selon Meeus 47 |
| `comet` | orbites képlériennes en variables universelles (toute excentricité), magnitude H + 5 log Δ + K log r |
| `stars` | lieu apparent d'une étoile (mouvement propre, précession, nutation, aberration) |
| `rise_set` | lever, passage et coucher à partir de courbes échantillonnées, nuit de référence |
| `constellation` | constellation hôte, par l'algorithme de Roman (1987) |

La **nuit de référence** est la nuit en cours si le Soleil est couché, sinon la prochaine. Elle va du coucher au lever du Soleil. Les levers, passages et couchers sont cherchés dans une fenêtre de 24 h centrée sur cette nuit, par pas de 10 min, avec interpolation. Dans les régions polaires, où il n'y a ni coucher ni lever, la fenêtre est centrée sur le minimum de hauteur du Soleil.

Le **statut de visibilité** se détermine dans cet ordre :
1. `SOUS L'HORIZON` si la hauteur apparente est négative.
2. `NOYÉ DANS LE JOUR` si le Soleil est levé ou si l'élongation est inférieure à 10°. La Lune fait exception : elle reste `VISIBLE` en plein jour, sauf si elle est trop près du Soleil.
3. `TROP FAIBLE` si la magnitude dépasse 6.
4. `VISIBLE` sinon.

### Front (`src/`)

```
App.vue          barre de navigation TUI : [F1] ORBITE / [F2] CIEL / [F3] OBJETS
router/          vue-router (historique par hash) : / (orbite, accueil), /ciel, /objets
views/           MainView     globe (2/3) | objets + télémétrie (1/3)
                 SkyView      voûte | observateur + temps
                 BodiesView   tableau | détail + comètes
components/      communs      TuiPanel (cadre), DataRow (ligne alignée), ViewTabs (onglets),
                              TuiButton, TuiInput, TuiTable
                 orbite       EarthGlobe (Three.js), WorldMap (SVG), ObjectList, TelemetryPanel
                 planétarium  SkyDome, EarthPicker, ObserverPanel, TimeControls, SkyStatusBar,
                              BodyDetail, AltitudeChart, MoonPhaseAscii, VisibilityLabel, CometStatusPanel
composables/     communs      usePolling (générique), useHotkeys (raccourcis actifs tant que le composant est monté)
                 orbite       useSpaceObjects (liste + sélection), useTelemetry (1 s), useTrajectory (30 s),
                              useSolarState (60 s)
                 planétarium  useTimeControl (temps simulé partagé), useObserver,
                              useSky (instantanés + interpolation), useBodies
assets/geo/      côtes Natural Earth 1:110m (domaine public), compactées en [lon, lat]
services/        wrappers typés : seul invokeCommand.ts appelle invoke()
types/           miroirs TypeScript des DTO Rust
utils/           formatage à largeur fixe, lat/lon ↔ scène 3D, projection équirectangulaire
                 (découpage à l'antiméridien), découpage passé/futur de la trajectoire,
                 projection stéréographique, interpolation du ciel, palette, ASCII
styles/          _variables.scss (palettes), _mixins.scss, main.scss (reset, scanlines)
```

Règles du front :
- TypeScript est en mode strict, avec `noUncheckedIndexedAccess` et sans `any`.
- Les composants n'appellent jamais `invoke()` directement.
- Les couleurs viennent uniquement de `_variables.scss`. Canvas et Three.js lisent les custom properties.

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

### Voûte céleste : pourquoi Canvas 2D

La voûte est dessinée en **Canvas 2D** plutôt qu'avec Three.js, pour trois raisons :
- Le volume est modeste : environ 2 500 étoiles visibles, 700 segments et une centaine de libellés par image. Regroupées en une vingtaine de `Path2D` (par opacité et par teinte), elles se dessinent en quelques millisecondes, ce qui tient 60 fps, y compris sur un Raspberry Pi 4.
- Le texte est le cœur du style TUI : noms de constellations, d'étoiles et d'objets, points cardinaux. Canvas 2D l'affiche nativement dans la police VT323, net et avec le glow. En WebGL, il aurait fallu des atlas de glyphes ou des sprites.
- La projection stéréographique (zénith au centre, est à gauche) et le découpage des segments à l'horizon sont des calculs 2D simples. Une scène 3D n'apporte rien ici.

Three.js reste utilisé pour la Terre, où la 3D et le raycast sont utiles.

### Fluidité du ciel

Le backend calcule des **instantanés**. `useSky` en demande toujours un d'avance sur le temps simulé, à intervalle réel de 400 ms à 5 s selon l'accélération : à ×3600, cela fait un instantané toutes les 24 min simulées. À chaque image, `SkyDome` interpole les directions (vecteurs ENU normalisés) entre les deux instantanés qui encadrent l'instant, appariés par identifiant : numéro HR, identifiant de segment, abréviation, identifiant d'objet. Si le backend tarde, le rendu extrapole jusqu'à une demi-période.

### Thème

Trois palettes sont disponibles : `orange` (par défaut), `amber` et `green`. Pour en changer :
- à la compilation, modifier `$phosphor-theme` dans `src/styles/_variables.scss` ;
- à l'exécution, ajouter `data-phosphor="green"` sur `<html>`. Les couleurs du globe sont lues au montage.

Chaque palette définit aussi trois fonds de voûte pour le planétarium : `sky-night`, `sky-twilight` et `sky-day`.

### Clavier

Les raccourcis ne sont actifs que sur l'écran qui les déclare.

| Écran | Touche | Action |
|---|---|---|
| tous | `F1` / `F2` / `F3` | écran Orbite / Ciel / Objets |
| Orbite | `↑` / `↓` | déplace le curseur dans la liste des objets |
| Orbite | `Entrée` ou clic | sélectionne l'objet sous le curseur |
| Orbite | `V` | bascule entre le globe et le planisphère |
| Ciel | `C` / `N` / `G` | constellations / noms / grille alt-az (30° et 60°) |
| Ciel | `R`, `Échap` | recentre la vue et désépingle l'objet |
| Ciel | `Espace` | pause / reprise |
| Ciel | `←` / `→` | −1 h / +1 h |
| Objets | `↑` / `↓`, `Entrée` ou `V` | sélection dans le tableau des objets, puis ouverture dans le ciel |

À la souris, sur la voûte : le survol affiche une infobulle, le clic épingle l'objet, la molette zoome et le glisser déplace la vue une fois zoomée. Sur la Terre du planétarium : le glisser fait tourner le globe, la molette zoome et un clic choisit le lieu d'observation.

## Données du planétarium : sources, licences et attributions

| Donnée | Source | Licence | Stockage |
|---|---|---|---|
| Étoiles (V ≤ 6,0 ; 5 080 étoiles) | *Yale Bright Star Catalogue*, 5e éd. révisée (Hoffleit & Warren 1991), via le Harvard TDC / NASA ADC (CDS V/50) | données scientifiques librement réutilisables avec citation | `src-tauri/data/stars.json` |
| Noms propres des étoiles | *IAU Catalog of Star Names* (WGSN, E. Mamajek), version du 2022-04-04 ; graphies françaises usuelles (Véga, Bételgeuse…) dans `tools/star_names_fr.csv` | CC BY 4.0 (IAU) | dans `stars.json` |
| Tracés des constellations | [d3-celestial](https://github.com/ofrohn/d3-celestial), `constellations.lines.json` © 2015 Olaf Frohn | BSD 3-Clause | `src-tauri/data/constellations.json` |
| Noms des constellations (latin, français, IAU) | table constituée pour le projet (`tools/constellation_names.csv`), d'après les noms officiels de l'IAU et l'usage français | — | dans `constellations.json` |
| Limites des constellations | Roman, N. G. 1987, *PASP* 99, 695 (CDS VI/42), copie distribuée par [astropy](https://github.com/astropy/astropy) | données publiques ; fichier astropy sous BSD 3-Clause | `src-tauri/data/boundaries.json` |
| Planètes et Soleil | VSOP87D, Bretagnon & Francou 1988, Bureau des longitudes / IMCCE ([ftp.imcce.fr](https://ftp.imcce.fr/pub/ephem/planets/vsop87/)), tronqué par `tools/build_catalogs.py` | données publiques, citation demandée | `src-tauri/src/astronomy/vsop87_data.rs` |
| Lune | tables 47.A et 47.B de Meeus (ELP-2000/82 tronquée), reprises de [astronomia](https://github.com/commenthol/astronomia) © Sonia Keys, commenthol | MIT | `src-tauri/src/astronomy/moon_tables.rs` |
| Comètes | Minor Planet Center, [`CometEls.txt`](https://www.minorplanetcenter.net/iau/MPCORB/CometEls.txt) | données publiques du MPC (IAU), citation demandée | téléchargées, cache dans `<app cache>/comets/`, rafraîchies tous les 7 jours |
| Côtes de la Terre | Natural Earth 1:110m | domaine public | `src/assets/geo/coastlines.json` |
| Police | VT323 (Peter Hull), via `@fontsource/vt323` | SIL Open Font License 1.1 | bundle |

Pour les planètes, le crate `vsop87` (MIT/Apache) a été écarté : son dépôt est archivé depuis 2024 et n'est plus maintenu. Les séries officielles de l'IMCCE sont donc tronquées par le script `tools/build_catalogs.py`. Un terme est gardé si A·0,2^α ≥ 10⁻⁷, ce qui donne 3 965 termes au total. L'écart avec les séries complètes reste inférieur à 0,25″ entre 1800 et 2200, période hors de laquelle `get_sky` refuse les dates.

## Précision du planétarium

Les tests Rust (`cargo test`) comparent les calculs à trois références :
- les exemples résolus de Meeus, *Astronomical Algorithms*, 2e éd. : 7.a, 12.a/b, 13.b, 21.b, 22.a, 23.a, 25.b, 32.a, 33.a, 40.a, 41.a, 47.a, 48.a, 15.a ;
- des éphémérides **JPL Horizons**, notées en dur dans les tests ;
- **Skyfield** avec DE421, également noté en dur.

| Grandeur | Tolérance testée | Écart typique |
|---|---|---|
| Temps sidéral | < 0,05 s (Meeus 12.a/b) | < 0,01 s |
| Soleil (RA/Dec apparentes) | < 0,002° (Horizons, 2010 et 2026) | ~1″ |
| Planètes (RA/Dec apparentes) | < 0,005° (Horizons, 7 planètes × 2 dates) | quelques ″ |
| Lune (RA/Dec géocentriques) | < 0,01° (Horizons) | ~10″ |
| Lune topocentrique depuis Paris à 1,2° de hauteur | < 0,02° (Horizons) | valide la parallaxe (~1°) |
| Alt/az d'étoiles (Véga et Polaire depuis Paris, Acrux depuis Sydney) | < 0,01° (Skyfield) | < 1″ |
| Comète 2P/Encke | < 0,01° (Horizons) | — |
| Lever, passage et coucher | < 2 min (Meeus 15.a ; lever et coucher du Soleil à Paris, Skyfield) | < 1 min |

Les hauteurs affichées sont **apparentes** : la réfraction standard est incluse. Les levers et couchers utilisent les hauteurs de référence classiques : −0,5667° pour les étoiles et planètes, −0,8333° pour le Soleil, et le bord supérieur topocentrique pour la Lune. La parallaxe annuelle des étoiles est négligée, de même que la correction FK5 de VSOP87 (0,09″) et ΔUT1 (moins d'1 s). Les magnitudes planétaires suivent Meeus 41, à environ 0,1 mag près.

Contrôles manuels, aussi couverts par les tests de `SkyFacade` :
- depuis Paris, l'étoile Polaire est à environ 49° de hauteur, plein nord ;
- depuis Sydney, en avril à 22 h, la Croix du Sud est haute et la Polaire est sous l'horizon.

## Limites connues du planétarium

- L'heure « LOCAL » est celle de la machine. Faute de base de fuseaux hors ligne, la barre d'état affiche aussi le temps solaire moyen du lieu observé.
- La teinte B−V des étoiles reste dans la palette : légèrement vers `sun` pour les étoiles chaudes, vers `danger` pour les étoiles froides.
