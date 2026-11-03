# SPEC VISUELLE — Recréation Minecraft Dungeons (Creeper Woods)

Source : 64 captures de référence dans `download/references/screens/` + planches-contact.
Cette spec est la LOI pour le code. Chaque valeur ci-dessous a été observée sur les captures.

## 1. Caméra (ref_02_f034f9a3, planche_01)
- Perspective ~45° FOV, placée derrière/au-dessus du héros, pitch ≈ 55°.
- Offset caméra : (0, 13.5, 9.5) du joueur, lerp doux (0.08), look-at joueur + (0, 1, 0).
- Le héros fait ~120-140 px de haut en 1080p. Léger zoom arrière quand beaucoup d'ennemis.

## 2. HUD (positions en fraction d'écran, 1920×1080 de référence)
- **Bas-centre, barre noire arrondie** (y≈0.78-0.88, de x≈0.22 à x≈0.78) :
  - Compteur d'âmes : icône violette + nombre vert (ex « 60 ») — À GAUCHE des artefacts.
  - 3 slots d'artefacts : carrés sombres #1d1d1f bordure or #c9a227, ~64 px, icône dedans,
    hint de touche (1/2/3) dans le coin bas-gauche, disque de recharge qui balaye.
  - CŒUR GÉANT rouge #d42a2a (contour #7a1414) ~110 px au centre : la vie.
    Sous lui : « 3 ♥ » = 3 vies, et « LV n » + barre XP violette #9b5cff.
  - Slot potion (icône fiole rouge/bleue, hint E, recharge ~30 s).
  - Slot flèches (icône flèches bleues) + badge nombre (ex « 20 ») au-dessus.
  - À droite : icône émeraude verte #34d16a + compteur (ex « 15993 »).
- **Haut-DROITE** : objectif en BLANC gras + sous-ligne OR #e8b84b (ex « LIBÉRER LES
  VILLAGEOIS — 2/5 »), petit icône or au-dessus. Saute/scala quand progression.
- **Boss** : nom en haut-centre (grand, blanc, contourné) + barre rouge #c81f1f longue
  avec losanges d'extrémité.
- Dégâts flottants : blanc (normal), jaune #ffd24a (crit/charge), montent et fondent.
- Vignette rouge + flash quand blessé ; battement de cœur sonore (optionnel).
- Prompt d'interaction au-dessus du joueur : touche entre crochets + action (ex « [F] Ouvrir »).
- Carte (M) : overlay plein écran façon parchemin bleu-sombre, tracé du chemin clair,
  icône joueur flèche, boussole « N » en haut à droite, objectifs listés.
- Inventaire (I) : panneau sombre central : modèle gauche, slots équipement centre
  (mêlée / distance / armure + 3 artefacts), détails objet à droite, émeraudes en haut.

## 3. Palette monde (Creeper Woods)
- Herbe : base #4c7a2f, clair #6f9c3c, ombre #243a14. Chemin terre : #8f6a3d / #7a5b34.
- Troncs : #5a4426 / #3f2f1a. Feuillages : chêne #2f5b22→#4e8a2e, buissons rouges #a03028.
- Champignons géants : chapeau rouge #b03a2a à pois blancs, pied crème — LUMINEUX (halo chaud).
- Eau : #3f6fae, reflets clairs #6f9fd9, translucide. Lave : cœur #ff7a1f, bord #d43b0f, ÉMISSIVE.
- Pierre/cimetière : #7d8a77, mousse #5d7a55, tombes #6b6f66. Temple : briques sombres #3c3a45.
- Brouillard : vert sombre désat #16211a, dense (exp2 k≈0.028), ciel = couleur du brouillard.
- Lumière « torche héros » chaude (1.0, 0.72, 0.35) r≈9 autour du joueur + chaque champignon
  = point light chaude r≈6. Sun faible froid (0.35, 0.42, 0.5) pour modeler les cubes.

## 4. Cubes & personnages (style Minecraft, cuboids)
- Héros : tunique teal #3aa3a3, pantalon brun #6b4a2f, bottes grises, tête chair #c68e63,
  cheveux bruns, ÉPÉE en main droite (lame claire + garde brune).
- Zombie : peau verte #3e7a3e, tunique cyan #2a8a8a, pantalon bleu #35456b.
- Squelette : os #cfcfcf, arcs, yeux sombres. Creeper : vert #4faa4f taché, face noire iconique.
- Nécromancien : robe violette #3a2a5a, visage pâle, bâti OR avec orbe violette ÉMISSIVE, plus grand.
- Villageois : robe brune #7a5b3a, gros nez. Enchanté : teinte violette + aura de particules.
- Animations procédurales : balancement bras/jambes en marche (sin), attaque = rotation bras,
  creeper enfle avant d'exploser, mort = chute + fondu. Ombres = disque sombre alpha 0.45.

## 5. Ambiance / particules
- Braises/oranges et particules de feuilles qui tombent partout (billboards additifs).
- Âmes : wisps VERTS #59ff8a qui volent en spirale vers le joueur (compteur d'âmes).
- Coulée de lumière dorée autour des torches/champignons (halo additif au sol).
- Pluie légère possible ; god-rays simulés par halos.

## 6. Écrans (FR)
- **Titre** : fond = camp de nuit rendu par le moteur (feu de camp, héros, brume) ;
  logo « MINECRAFT DUNGEONS » façon key art (argent + or biseauté) ; menu à gauche :
  JOUER / COMMANDES / QUITTER (curseur ➤).
- **Briefing** : texte d'intro de mission FR sur fond sombre, « Appuyez sur Entrée ».
- **Pause** : REPRENDRE / COMMANDES / QUITTER LA MISSION. **Game over** : « VOUS ÊTES TOMBÉ… »
  vies restantes, respawn au dernier point de contrôle. **Victoire** : stats (temps, émeraudes,
  âmes, villageois libérés, niveau).
- Mention discrète : « Recréation non officielle à but pédagogique ».

## 7. Mécaniques chiffrées (issues de la recherche validée)
- 100 PV, 3 vies/mission, âmes max 300, combo mêlée 3 coups mémorisé entre pauses,
  roulade avec i-frames, potion ~75 PV (E, recharge), flèches limitées (20 au départ),
  émeraudes + XP (1 pt/niveau). Enchanté : PV ×2, aura violette, +XP.
- Nécromancien : 600 PV, invocations en cercle ≤7 zombies, fuite puis ré-invocation,
  orbes laser violettes répulsives. Bestiaire : zombie 20 PV, squelette 15 PV (distance),
  creeper 12 PV (explose ~30). Villageois ×5 en cages ; marchand capturé ; 2 coffres
  (auberge, caravane) ; parchemin secret → « Creepy Crypt » (salle bonus).
- Objectifs FR : Rejoindre le village → Libérer les villageois (0/5) → Trouver la caravane →
  Affronter le Nécromancien → S'échapper par le temple.

## 8. Contrôles (scancodes physiques — AZERTY-safe)
ZQSD via scancodes KeyW/A/S/D ; Clic G mêlée (+F) ; Clic D arc ; Espace roulade ;
E potion ; 1/2/3 artefacts ; M carte ; I inventaire ; Échap pause ; F2 capture.
