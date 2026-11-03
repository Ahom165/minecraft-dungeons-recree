# Minecraft Dungeons — Recréation (Creeper Woods)

Recréation fidèle du niveau **Creeper Woods** de *Minecraft Dungeons*, en **Rust + wgpu**,
avec interface **100 % française**, HUD fidèle aux captures du jeu original, boss
**Nécromancien (600 PV)**, 3 artefacts, carte, âmes (max 300), combo de mêlée mémorisé,
ennemis enchantés, coffres, secret *Creepy Crypt*, pont de lave et temple de sortie.

> Recréation non officielle, à but pédagogique. *Minecraft Dungeons* est une marque de Mojang/Microsoft.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange) ![wgpu](https://img.shields.io/badge/wgpu-0.19-blue)

## Lancer

### Windows
Télécharger `minecraft-dungeons-recree.exe` (artefact CI ou release), double-cliquer.
Aucune dépendance : police et textures sont générées au lancement.

### Linux (depuis les sources)
```bash
cargo build --release
./target/release/minecraft-dungeons-recree
```

## Contrôles (scancodes physiques — identiques en AZERTY et QWERTY)

| Action | Touche |
|---|---|
| Se déplacer | `Z Q S D` (touches physiques W A S D) |
| Mêlée (combo de 3) | Clic gauche |
| Interagir | `F` |
| Arc (auto-visée) | Clic droit |
| Roulade (i-frames) | `Espace` |
| Potion (+75 PV, recharge 30 s) | `E` |
| Artefact 1 — Flèche d'artifice | `1` |
| Artefact 2 — Cor de vent (60 âmes) | `2` |
| Artefact 3 — Totem de régénération | `3` |
| Carte | `M` |
| Inventaire | `I` |
| Pause | `Échap` |
| Capture d'écran | `F2` |

## La mission

1. **Rejoindre le village** (lac aux têtes de pierre → bois profonds → auberge).
2. **Libérer les villageois (0/5)** — cages dorées, touche `F`.
3. **Trouver la caravane** (cimetière → tombe à rune → secret **Creepy Crypt**).
4. **Affronter le Nécromancien** — 600 PV, invocations en cercle (max 7 zombies),
   fuite téléportée, orbes violettes répulsives.
5. **S'échapper par le temple** (pont de lave).

Ennemis : zombies (20 PV), squelettes (15 PV, distance), creepers (12 PV, explosion),
**enchantés** (aura violette, PV ×2, XP ×2). 100 PV, 3 vies, potion ~75 PV,
émeraudes et XP (1 point d'enchantement par niveau).

## Textures personnalisées

Toutes les textures sont **procédurales** (générées au lancement). Pour les remplacer
par les textures officielles extraites du jeu, déposer des PNG 128×128 dans
`assets/textures/` avec le nom de la case :

`grass_top`, `grass_side`, `dirt`, `path_dirt`, `stone`, `mossy_stone`, `cobble`,
`gravel`, `trunk_side`, `trunk_top`, `leaves_oak`, `leaves_dark`, `bush_red`, `plank`,
`roof_dark`, `roof_red`, `water`, `lava`, `mushroom_stem`, `mushroom_cap`, `glow_moss`,
`rune_stone`, `tomb_stone`, `temple_brick`, `gold_block`, `bookshelf`, `door`, `banner`,
`obsidian`, `redstone`, `web`, `mossy_cobble`, `shadow`, `glow_disc`, `star`, `ring`,
`leaf_sprite`, `hay`, `soul_face`, `exit_portal`.

Extraction des originaux : *Minecraft Dungeons Extractor* (outil communautaire),
puis exporter les atlas de tuiles et recadrer chaque tuile en 128×128.

## Compilation

```bash
cargo build --release        # build
cargo clippy -- -D warnings  # zéro warning exigé (vérifié en CI)
cargo run -- --qa captures   # mode QA : génère 8 captures d'écran puis quitte
```

Variables d'environnement utiles :
- `MCD_BACKEND` : force un backend (`vulkan`, `gl`) — sinon tous les backends primaires.
- `MCD_DUMP_TEX=1` : sauvegarde l'atlas monde tel qu'uploadé sur GPU (`/tmp/world_tex_gpu.png`).
- `MCD_DUMP_BUF=1` : sauvegarde le vertex buffer statique (`/tmp/static_gpu.bin`).
- `MCD_SIZE=1280x720` : taille de fenêtre du mode QA.

## CI / CD

`.github/workflows/build.yml` :
- **Linux** : `cargo clippy -- -D warnings` + build release + artefact.
- **Windows** : build release + artefact `minecraft-dungeons-recree-windows` (le .exe).
- **Release** : sur tag `v*`, le .exe est attaché à une GitHub Release.

## Structure du code

```
src/
├── main.rs        Point d'entrée, boucle winit, mode QA (captures)
├── app/…          (voir modules ci-dessous)
├── gfx.rs         Renderer wgpu (monde éclairé + brouillard, particules, UI, capture)
├── atlas.rs       Atlas de textures procédural (40 tuiles) + mips CPU + overrides PNG
├── ui_atlas.rs    Atlas UI : icônes pixel-art, cœur, vignette, glyphe cache
├── text.rs        Rendu de texte (fontdue) avec contour façon Minecraft
├── vertex.rs      Sommets + constructeur de cuboïdes animés (style Minecraft)
├── level.rs       Génération de Creeper Woods (10 zones, colliders, lumières)
├── mobs.rs        IA (zombie, squelette, creeper, nécromancien, villageois) + modèles
├── player.rs      Héros : déplacement, combo, roulade, potion, artefacts
├── game.rs        Boucle de jeu : écrans, mission, combat, objectifs, ramassages
├── hud.rs         HUD fidèle (cœur géant, artefacts, âmes, émeraudes, carte, menus)
├── particles.rs   Feuilles, braises, âmes, explosions, ondes
├── camera.rs      Caméra 3/4 façon Minecraft Dungeons
├── input.rs       Entrées scancodes (AZERTY-safe)
└── rng.rs         PRNG déterministe + bruit (aucune dépendance)
```

## Performance

- Monde statique fusionné en **1 draw call** (~120 k sommets).
- Entités dynamiques recomposées sur CPU (cuboids animés) : 1 draw call.
- Particules : 2 draw calls (alpha trié + additif).
- UI : 1 draw call (texte + icônes + vignette).
- Éclairage : soleil + 16 point lights les plus proches (façon MCD), tone mapping Reinhard.
