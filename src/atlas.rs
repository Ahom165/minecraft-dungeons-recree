//! Atlas de textures du monde : 1024×1024, grille 8×8 de tuiles 128 px.
//! Textures procédurales en pixel-art 16×16 (style Minecraft), upscale
//! nearest ×8 pour un rendu net ; remplacées si `assets/textures/<nom>.png` existe.

use image::DynamicImage;

pub const TILE: usize = 128;

/// Grille pixel-art (16×16) et facteur d'agrandissement (×8).
const P: usize = 16;
const PX: usize = TILE / P;

/// Noms des cases (index = case dans la grille). Sert aussi au chargement utilisateur.
pub const SLOT_NAMES: [&str; 40] = [
    "grass_top",    // 0
    "grass_side",   // 1
    "dirt",         // 2
    "path_dirt",    // 3
    "stone",        // 4
    "mossy_stone",  // 5
    "cobble",       // 6
    "gravel",       // 7
    "trunk_side",   // 8
    "trunk_top",    // 9
    "leaves_oak",   // 10
    "leaves_dark",  // 11
    "bush_red",     // 12
    "plank",        // 13
    "roof_dark",    // 14
    "roof_red",     // 15
    "water",        // 16
    "lava",         // 17
    "mushroom_stem",// 18
    "mushroom_cap", // 19
    "glow_moss",    // 20
    "rune_stone",   // 21
    "tomb_stone",   // 22
    "temple_brick", // 23
    "gold_block",   // 24
    "bookshelf",    // 25
    "door",         // 26
    "banner",       // 27
    "obsidian",     // 28
    "redstone",     // 29
    "web",          // 30
    "mossy_cobble", // 31
    "shadow",       // 32 (disque sombre, ombres portées)
    "glow_disc",    // 33 (halo doux, particules additives)
    "star",         // 34 (étincelle 4 branches)
    "ring",         // 35 (cercle, ondes de choc)
    "leaf_sprite",  // 36 (feuille qui tombe)
    "hay",          // 37 (caravane)
    "soul_face",    // 38 (visage pâle du nécromancien)
    "exit_portal",  // 39 (portail violet)
];

pub struct WorldAtlas {
    /// Atlas 1024×1024 RGBA + mips (1024, 512, 256, 128).
    pub pixels: Vec<u8>,
    pub used_names: Vec<String>,
}

pub const ATLAS_SIZE: usize = 1024;
pub const MIP_COUNT: usize = 3; // 1024 -> 128 (les cases font encore 16 px)

/// Taille totale (octets) de l'atlas avec tous ses mips.
pub const TOTAL_STRIDE: usize = {
    let mut off = 0;
    let mut m = 0;
    while m <= MIP_COUNT {
        let s = ATLAS_SIZE >> m;
        off += s * s * 4;
        m += 1;
    }
    off
};

/// Décalage (octets) du mip `mip` dans le buffer.
pub const fn mip_offset(mip: usize) -> usize {
    let mut off = 0;
    let mut m = 0;
    while m < mip {
        let s = ATLAS_SIZE >> m;
        off += s * s * 4;
        m += 1;
    }
    off
}

// ============================================================
//  Outils pixel-art 16×16
// ============================================================

/// Hash déterministe → f32 dans [0,1).
fn h01(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(0x27D4_EB2D)
        ^ (y as u32).wrapping_mul(0x1656_67B1)
        ^ seed;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

/// Bruit aggloméré : gros amas (2×2) + grain fin — imite le dithering Minecraft.
fn clump(x: i32, y: i32, seed: u32) -> f32 {
    0.62 * h01(x >> 1, y >> 1, seed) + 0.38 * h01(x, y, seed ^ 0x9E37_79B9)
}

/// Choisit une couleur d'une palette de 4 nuances selon t∈[0,1].
fn pick(pal: &[[u8; 3]; 4], t: f32) -> [u8; 3] {
    let i = if t < 0.22 {
        0
    } else if t < 0.5 {
        1
    } else if t < 0.78 {
        2
    } else {
        3
    };
    pal[i]
}

/// Peinture 16×16 puis upscale nearest ×8 vers 128×128.
struct P16 {
    px: Vec<u8>,
}

impl P16 {
    fn new() -> P16 {
        P16 { px: vec![0; P * P * 4] }
    }
    fn set(&mut self, x: i32, y: i32, c: [u8; 3], a: u8) {
        if (0..P as i32).contains(&x) && (0..P as i32).contains(&y) {
            let i = ((y as usize) * P + x as usize) * 4;
            self.px[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], a]);
        }
    }
    /// Bruit aggloméré + palette sur toute la tuile.
    fn terrain(&mut self, pal: &[[u8; 3]; 4], seed: u32) {
        for y in 0..P as i32 {
            for x in 0..P as i32 {
                let c = pick(pal, clump(x, y, seed));
                self.set(x, y, c, 255);
            }
        }
    }
    fn upscale(&self, out: &mut [u8]) {
        for y in 0..TILE {
            let sy = y / PX;
            for x in 0..TILE {
                let sx = x / PX;
                let s = (sy * P + sx) * 4;
                let d = (y * TILE + x) * 4;
                out[d..d + 4].copy_from_slice(&self.px[s..s + 4]);
            }
        }
    }
}

// Palettes (nuances du plus sombre au plus clair).
const PAL_GRASS: [[u8; 3]; 4] = [[69, 122, 46], [84, 145, 58], [99, 168, 68], [114, 189, 79]];
const PAL_DIRT: [[u8; 3]; 4] = [[94, 65, 40], [110, 78, 51], [121, 85, 58], [138, 102, 71]];
const PAL_STONE: [[u8; 3]; 4] = [[104, 104, 104], [116, 116, 116], [128, 128, 128], [140, 140, 140]];
const PAL_WATER: [[u8; 3]; 4] = [[28, 58, 120], [34, 70, 138], [40, 82, 156], [32, 66, 128]];
const PAL_LAVA: [[u8; 3]; 4] = [[122, 28, 6], [178, 58, 10], [224, 104, 20], [246, 150, 32]];

pub fn paint_slot(name: &str, px: &mut [u8]) {
    // ---- Cases spéciales à dégradé lisse (particules/ombres) : peintes en 128 px ----
    match name {
        "shadow" => {
            paint_radial(px, |d| {
                let a = ((1.0 - d) * 255.0 * 1.4).clamp(0.0, 255.0) as u8;
                [10, 12, 10, a]
            });
            return;
        }
        "glow_disc" => {
            paint_radial(px, |d| {
                let a = ((1.0 - d) * (1.0 - d) * 255.0) as u8;
                [255, 255, 255, a]
            });
            return;
        }
        "star" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = (x as f32 - 63.5).abs();
                    let dy = (y as f32 - 63.5).abs();
                    let v = (1.0 - (dx + dy) / 63.5).clamp(0.0, 1.0);
                    let a = (v.powf(3.0) * 255.0) as u8;
                    let i = (y * TILE + x) * 4;
                    px[i..i + 4].copy_from_slice(&[255, 255, 255, a]);
                }
            }
            return;
        }
        "ring" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = x as f32 - 63.5;
                    let dy = y as f32 - 63.5;
                    let d = (dx * dx + dy * dy).sqrt();
                    let v = (1.0 - (d - 52.0).abs() / 8.0).clamp(0.0, 1.0);
                    let i = (y * TILE + x) * 4;
                    px[i..i + 4].copy_from_slice(&[255, 255, 255, (v * 255.0) as u8]);
                }
            }
            return;
        }
        "leaf_sprite" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = (x as f32 - 63.5) / 63.5;
                    let dy = (y as f32 - 63.5) / 63.5;
                    let v = 1.0 - (dx * dx + dy * dy).sqrt();
                    let g = (120.0 + v * 110.0).clamp(0.0, 255.0) as u8;
                    let a = if v > 0.25 { 255 } else { 0 };
                    let i = (y * TILE + x) * 4;
                    px[i..i + 4].copy_from_slice(&[g / 2, g, g / 3, a]);
                }
            }
            return;
        }
        _ => {}
    }

    // ---- Tuiles de blocs : pixel-art 16×16 ----
    let mut t = P16::new();
    let mut seed: u32 = 0;
    for (i, n) in SLOT_NAMES.iter().enumerate() {
        if *n == name {
            seed = 0x51ED_2700 + i as u32;
        }
    }
    match name {
        "grass_top" => {
            t.terrain(&PAL_GRASS, seed);
            // brins clairs épars
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    if h01(x, y, seed ^ 7) > 0.94 {
                        t.set(x, y, [142, 212, 101], 255);
                    }
                }
            }
        }
        "grass_side" => {
            // terre + frange d'herbe irrégulière sur 3-4 px (style Minecraft)
            t.terrain(&PAL_DIRT, seed);
            for x in 0..P as i32 {
                let dip = (h01(x, 11, seed) * 2.0).floor() as i32; // 0 ou 1
                let top = 3 + dip;
                for y in 0..top {
                    let c = pick(&PAL_GRASS, clump(x, y, seed));
                    t.set(x, y, c, 255);
                }
                if h01(x, 12, seed) > 0.6 {
                    let c = pick(&PAL_GRASS, h01(x, 13, seed));
                    t.set(x, top, c, 255);
                }
            }
        }
        "dirt" => {
            t.terrain(&PAL_DIRT, seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    if h01(x >> 1, y >> 1, seed ^ 21) > 0.88 {
                        t.set(x, y, [74, 52, 32], 255);
                    }
                }
            }
        }
        "path_dirt" => {
            t.terrain(&[[138, 107, 68], [151, 118, 78], [165, 131, 90], [178, 146, 106]], seed);
        }
        "stone" => {
            t.terrain(&PAL_STONE, seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    if h01(x, y, seed ^ 31) > 0.93 {
                        t.set(x, y, [90, 90, 90], 255);
                    }
                }
            }
        }
        "cobble" | "mossy_cobble" => {
            // pavés 4×4 de 4 px, mortier sombre, nuance par pavé
            let mortar: [u8; 3] = [69, 69, 69];
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let id = (x / 4) * 8 + y / 4;
                    let shade = 0.82 + h01(id, 3, seed) * 0.18;
                    let mut c = pick(&PAL_STONE, h01(id, 7, seed));
                    if x % 4 == 0 || y % 4 == 0 {
                        c = mortar;
                    } else {
                        let j = (h01(x, y, seed) - 0.5) * 12.0;
                        c = [
                            (c[0] as f32 * shade + j).clamp(0.0, 255.0) as u8,
                            (c[1] as f32 * shade + j).clamp(0.0, 255.0) as u8,
                            (c[2] as f32 * shade + j).clamp(0.0, 255.0) as u8,
                        ];
                    }
                    t.set(x, y, c, 255);
                }
            }
            if name == "mossy_cobble" {
                for y in 0..P as i32 {
                    for x in 0..P as i32 {
                        if clump(x, y, seed ^ 41) > 0.68 {
                            t.set(x, y, [91, 122, 58], 255);
                        }
                    }
                }
            }
        }
        "gravel" => {
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let r = h01(x, y, seed);
                    let c: [u8; 3] = if r < 0.2 {
                        [95, 95, 95]
                    } else if r < 0.5 {
                        [117, 117, 117]
                    } else if r < 0.8 {
                        [135, 135, 135]
                    } else if r < 0.92 {
                        [154, 154, 154]
                    } else {
                        [138, 122, 98]
                    };
                    t.set(x, y, c, 255);
                }
            }
        }
        "trunk_side" => {
            // écorce : stries verticales + sillons sombres
            for x in 0..P as i32 {
                let col = h01(x, 0, seed);
                let groove = col > 0.72;
                for y in 0..P as i32 {
                    let j = (h01(x, y, seed ^ 5) - 0.5) * 14.0;
                    let base: [[u8; 3]; 3] = [[90, 68, 40], [107, 82, 50], [122, 94, 58]];
                    let mut c = base[(col * 2.99) as usize];
                    if groove {
                        c = [71, 54, 31];
                    }
                    t.set(x, y, [
                        (c[0] as f32 + j).clamp(0.0, 255.0) as u8,
                        (c[1] as f32 + j).clamp(0.0, 255.0) as u8,
                        (c[2] as f32 + j).clamp(0.0, 255.0) as u8,
                    ], 255);
                }
            }
        }
        "trunk_top" => {
            // cernes concentriques carrés + bord d'écorce
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let dx = (x - 8).abs().max((y - 8).abs());
                    let c: [u8; 3] = if dx >= 7 {
                        [107, 82, 50]
                    } else if dx == 0 {
                        [168, 135, 79]
                    } else if dx % 2 == 0 {
                        [156, 123, 69]
                    } else {
                        [138, 108, 62]
                    };
                    t.set(x, y, c, 255);
                }
            }
        }
        "leaves_oak" | "leaves_dark" => {
            let pal: [[u8; 3]; 4] = if name == "leaves_oak" {
                [[30, 61, 20], [42, 82, 28], [58, 110, 38], [76, 134, 50]]
            } else {
                [[22, 48, 15], [31, 66, 22], [42, 86, 30], [53, 104, 38]]
            };
            let hole = if name == "leaves_oak" { 0.93 } else { 0.9 };
            t.terrain(&pal, seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let r = h01(x, y, seed ^ 9);
                    if r > hole {
                        t.set(x, y, [0, 0, 0], 0); // trou de feuillage
                    } else if r > hole - 0.05 {
                        let bright: [u8; 3] = if name == "leaves_oak" { [95, 160, 62] } else { [70, 130, 48] };
                        t.set(x, y, bright, 255);
                    }
                }
            }
        }
        "bush_red" => {
            t.terrain(&[[110, 30, 24], [142, 42, 34], [168, 58, 44], [196, 78, 58]], seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    if h01(x, y, seed ^ 13) > 0.94 {
                        t.set(x, y, [216, 98, 74], 255);
                    }
                }
            }
        }
        "plank" => {
            // 4 planches horizontales, joints décalés, veine discrète
            let joints = [3i32, 11, 6, 14];
            for y in 0..P as i32 {
                let row = y / 4;
                for x in 0..P as i32 {
                    let seam = y % 4 == 3 || x == joints[row as usize];
                    let j = (h01(x, y, seed) - 0.5) * 10.0;
                    let base: [u8; 3] = if seam {
                        [126, 96, 52]
                    } else {
                        match y % 4 {
                            0 => [184, 148, 95],
                            1 => [178, 142, 90],
                            2 => [172, 136, 86],
                            _ => [166, 130, 82],
                        }
                    };
                    t.set(x, y, [
                        (base[0] as f32 + j).clamp(0.0, 255.0) as u8,
                        (base[1] as f32 + j).clamp(0.0, 255.0) as u8,
                        (base[2] as f32 + j).clamp(0.0, 255.0) as u8,
                    ], 255);
                }
            }
        }
        "roof_dark" | "roof_red" => {
            // bardeaux : rangées de 4 px, joints décalés, liseré clair en haut
            let (pal, hl, jcol): ([[u8; 3]; 3], [u8; 3], [u8; 3]) = if name == "roof_dark" {
                ([[51, 44, 60], [60, 52, 68], [69, 60, 80]], [82, 70, 94], [36, 31, 44])
            } else {
                ([[122, 48, 36], [142, 58, 44], [160, 68, 50]], [176, 80, 64], [94, 36, 26])
            };
            for y in 0..P as i32 {
                let row = y / 4;
                for x in 0..P as i32 {
                    let joint = x % 8 == (row * 4) % 8;
                    let c = if y % 4 == 0 {
                        hl
                    } else if joint {
                        jcol
                    } else {
                        pal[(y % 4 - 1).clamp(0, 2) as usize]
                    };
                    t.set(x, y, c, 255);
                }
            }
        }
        "water" => {
            // vagues horizontales par rangée + scintillement
            for y in 0..P as i32 {
                let band = h01(0, y, seed);
                for x in 0..P as i32 {
                    let mut c = pick(&PAL_WATER, band * 0.7 + h01(x, y, seed) * 0.3);
                    if h01(x, y, seed ^ 17) > 0.98 {
                        c = [70, 110, 180];
                    }
                    t.set(x, y, c, 255);
                }
            }
        }
        "lava" => {
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let mut c = pick(&PAL_LAVA, clump(x, y, seed));
                    if h01(x, y, seed ^ 19) > 0.975 {
                        c = [252, 196, 60];
                    }
                    t.set(x, y, c, 255);
                }
            }
        }
        "mushroom_stem" => {
            t.terrain(&[[200, 191, 164], [216, 208, 184], [228, 220, 196], [236, 230, 210]], seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    if h01(x, y, seed ^ 23) > 0.92 {
                        t.set(x, y, [168, 156, 128], 255);
                    }
                }
            }
        }
        "mushroom_cap" => {
            t.terrain(&[[142, 42, 32], [168, 52, 36], [188, 64, 48], [204, 80, 60]], seed);
            for (sx, sy) in [(3, 4), (10, 3), (6, 9), (12, 11)] {
                for dy in 0..2 {
                    for dx in 0..2 {
                        t.set(sx + dx, sy + dy, [232, 224, 208], 255);
                    }
                }
            }
        }
        "glow_moss" => {
            t.terrain(&[[58, 82, 40], [68, 98, 48], [80, 114, 56], [92, 130, 64]], seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let r = clump(x, y, seed ^ 27);
                    if r > 0.72 {
                        t.set(x, y, [155, 216, 90], 255);
                    }
                    if h01(x, y, seed ^ 28) > 0.95 {
                        t.set(x, y, [200, 240, 120], 255);
                    }
                }
            }
        }
        "rune_stone" => {
            t.terrain(&PAL_STONE, seed);
            // rune gravée lumineuse (cadre + croix diagonale)
            let bright: [u8; 3] = [88, 200, 232];
            let dim: [u8; 3] = [48, 144, 176];
            for k in 2..14 {
                t.set(k, 2, dim, 255);
                t.set(k, 13, dim, 255);
                t.set(2, k, dim, 255);
                t.set(13, k, dim, 255);
            }
            for k in 0..12 {
                t.set(4 + k / 2 + (k % 2), 4 + k, dim, 255);
            }
            for k in 5..11 {
                t.set(k, k, bright, 255);
                t.set(16 - k, k, bright, 255);
            }
        }
        "tomb_stone" => {
            t.terrain(&[[118, 122, 112], [130, 134, 124], [142, 146, 136], [152, 156, 146]], seed);
            for x in 0..P as i32 {
                t.set(x, 0, [106, 110, 100], 255);
                t.set(x, 15, [96, 100, 92], 255);
            }
            for y in 5..9 {
                for x in 6..10 {
                    t.set(x, y, [104, 108, 100], 255);
                }
            }
        }
        "temple_brick" => {
            // pierres taillées 2×2 (8×8), mortier sombre, rangées décalées
            let mortar: [u8; 3] = [78, 78, 78];
            for y in 0..P as i32 {
                let row = y / 8;
                for x in 0..P as i32 {
                    let id = row * 2 + (x / 8);
                    let joint = y % 8 == 0 || x % 8 == (row % 2) * 8;
                    let j = (h01(x, y, seed) - 0.5) * 10.0;
                    let mut c = pick(&PAL_STONE, h01(id, 5, seed));
                    if joint {
                        c = mortar;
                    } else {
                        c = [
                            (c[0] as f32 + j).clamp(0.0, 255.0) as u8,
                            (c[1] as f32 + j).clamp(0.0, 255.0) as u8,
                            (c[2] as f32 + j).clamp(0.0, 255.0) as u8,
                        ];
                    }
                    t.set(x, y, c, 255);
                }
            }
        }
        "gold_block" => {
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let border = x == 0 || y == 0 || x == 15 || y == 15;
                    let c: [u8; 3] = if border {
                        [184, 144, 40]
                    } else if x < 3 && y < 3 {
                        [252, 232, 122]
                    } else if x > 11 && y > 11 {
                        [200, 160, 46]
                    } else {
                        [232, 192, 56]
                    };
                    t.set(x, y, c, 255);
                }
            }
        }
        "bookshelf" => {
            // planches en haut/bas/milieu, rangées de livres colorés
            let plank: [u8; 3] = [184, 148, 95];
            let seam: [u8; 3] = [126, 96, 52];
            let books: [[u8; 3]; 6] = [
                [168, 56, 44], [56, 104, 168], [78, 142, 62],
                [200, 160, 46], [124, 78, 168], [192, 104, 56],
            ];
            for y in 0..P as i32 {
                let shelf = !(2..=13).contains(&y) || y == 7 || y == 8;
                let mut x = 1;
                while x < 15 {
                    let w = 2 + (h01(x, y / 4, seed) * 1.99) as i32;
                    for bx in x..(x + w).min(15) {
                        for by in 0..P as i32 {
                            let c = if shelf {
                                if by % 4 == 3 { seam } else { plank }
                            } else if by > 2 && by < 7 {
                                books[(h01(bx, by / 4, seed) * 5.99) as usize]
                            } else if by > 9 && by < 14 {
                                books[(h01(bx + 7, by / 4, seed ^ 3) * 5.99) as usize]
                            } else if by == 2 || by == 9 {
                                seam
                            } else {
                                plank
                            };
                            t.set(bx, by, c, 255);
                        }
                    }
                    x += w;
                }
            }
        }
        "door" => {
            let frame: [u8; 3] = [94, 70, 40];
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let edge = x == 0 || x == 15 || y == 0 || y == 15;
                    let col = h01(x, 0, seed);
                    let mut c = match (col * 2.99) as usize {
                        0 => [142, 110, 62],
                        1 => [154, 120, 68],
                        _ => [166, 132, 76],
                    };
                    if edge {
                        c = frame;
                    }
                    t.set(x, y, c, 255);
                }
            }
            t.set(12, 8, [240, 210, 74], 255);
            t.set(11, 8, [240, 210, 74], 255);
            t.set(12, 9, [240, 210, 74], 255);
            t.set(11, 9, [240, 210, 74], 255);
        }
        "banner" => {
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let c: [u8; 3] = if x == 0 || x == 15 || y == 15 {
                        [110, 30, 24]
                    } else if (x / 2) % 2 == 0 {
                        [176, 62, 48]
                    } else {
                        [160, 50, 40]
                    };
                    t.set(x, y, c, 255);
                }
            }
        }
        "obsidian" => {
            t.terrain(&[[14, 10, 22], [22, 16, 32], [30, 22, 48], [42, 30, 68]], seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    if h01(x, y, seed ^ 33) > 0.94 {
                        t.set(x, y, [78, 58, 120], 255);
                    }
                }
            }
        }
        "redstone" => {
            t.terrain(&PAL_STONE, seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let r = clump(x, y, seed ^ 35);
                    if r > 0.86 {
                        t.set(x, y, [194, 56, 24], 255);
                    }
                    if r > 0.93 {
                        t.set(x, y, [255, 106, 74], 255);
                    }
                }
            }
        }
        "web" => {
            // toile : fond transparent, fils blancs en 16×16
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    t.set(x, y, [232, 232, 232], 0);
                }
            }
            let spoke = |t: &mut P16, dx: i32, dy: i32| {
                for k in 0..8 {
                    t.set(8 + dx * k / 8, 8 + dy * k / 8, [232, 232, 232], 255);
                }
            };
            spoke(&mut t, 8, 8);
            spoke(&mut t, -8, 8);
            spoke(&mut t, 8, -8);
            spoke(&mut t, -8, -8);
            for r in [3, 6] {
                for a in 0..32 {
                    let ang = a as f32 * std::f32::consts::PI / 16.0;
                    t.set(8 + (ang.cos() * r as f32) as i32, 8 + (ang.sin() * r as f32) as i32, [232, 232, 232], 255);
                }
            }
        }
        "hay" => {
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let j = (h01(x, y, seed) - 0.5) * 12.0;
                    let mut c: [u8; 3] = match y % 5 {
                        0 => [168, 136, 42],
                        1 => [200, 160, 46],
                        2 => [190, 150, 44],
                        3 => [180, 142, 40],
                        _ => [172, 136, 38],
                    };
                    if y % 5 == 4 && x % 8 == 7 {
                        c = [142, 112, 32];
                    }
                    t.set(x, y, [
                        (c[0] as f32 + j).clamp(0.0, 255.0) as u8,
                        (c[1] as f32 + j).clamp(0.0, 255.0) as u8,
                        (c[2] as f32 + j).clamp(0.0, 255.0) as u8,
                    ], 255);
                }
            }
        }
        "soul_face" => {
            t.terrain(&[[184, 144, 104], [198, 160, 120], [210, 174, 136], [222, 190, 152]], seed);
            for (ex, ey) in [(4, 5), (10, 5)] {
                for dy in 0..3 {
                    for dx in 0..3 {
                        t.set(ex + dx, ey + dy, [42, 30, 34], 255);
                    }
                }
            }
            for x in 6..10 {
                t.set(x, 11, [74, 50, 48], 255);
                t.set(x, 12, [58, 40, 40], 255);
            }
        }
        "exit_portal" => {
            t.terrain(&[[26, 12, 42], [58, 28, 94], [85, 40, 140], [122, 60, 184]], seed);
            for y in 0..P as i32 {
                for x in 0..P as i32 {
                    let r = h01(x, y, seed ^ 37);
                    if r > 0.94 {
                        t.set(x, y, [160, 92, 224], 255);
                    }
                    if r > 0.98 {
                        t.set(x, y, [216, 176, 255], 255);
                    }
                }
            }
        }
        _ => {
            // case inconnue : rose vif, facile à repérer en QA
            for y in 0..P {
                for x in 0..P {
                    t.set(x as i32, y as i32, [200, 60, 200], 255);
                }
            }
        }
    }
    t.upscale(px);
}

/// Peint un dégradé radial 128×128 (ombres, halos).
fn paint_radial(px: &mut [u8], f: impl Fn(f32) -> [u8; 4]) {
    for y in 0..TILE {
        for x in 0..TILE {
            let dx = (x as f32 - 63.5) / 63.5;
            let dy = (y as f32 - 63.5) / 63.5;
            let d = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
            let i = (y * TILE + x) * 4;
            px[i..i + 4].copy_from_slice(&f(d));
        }
    }
}

pub fn build(atlas_dir: &str) -> WorldAtlas {
    let mut atlas = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
    let mut used_names = Vec::new();
    for (i, name) in SLOT_NAMES.iter().enumerate() {
        let mut buf = vec![0u8; TILE * TILE * 4];
        paint_slot(name, &mut buf);
        // Override utilisateur : assets/textures/<name>.png
        let user_path = format!("{atlas_dir}/{name}.png");
        if let Ok(img) = image::open(&user_path) {
            let rgba: DynamicImage =
                img.resize_exact(TILE as u32, TILE as u32, image::imageops::FilterType::Nearest);
            let raw = rgba.to_rgba8().into_raw();
            buf.copy_from_slice(&raw);
            used_names.push(name.to_string());
        }
        // Copie dans la grille 8×8
        let col = i % 8;
        let row = i / 8;
        for y in 0..TILE {
            let src = y * TILE * 4;
            let dst = ((row * TILE + y) * ATLAS_SIZE + col * TILE) * 4;
            atlas[dst..dst + TILE * 4].copy_from_slice(&buf[src..src + TILE * 4]);
        }
    }
    // Mips (downscale 2× de l'atlas entier)
    let mut pixels = vec![0u8; TOTAL_STRIDE];
    let mut level = atlas;
    let mut side = ATLAS_SIZE;
    for mip in 0..=MIP_COUNT {
        let off = mip_offset(mip);
        pixels[off..off + level.len()].copy_from_slice(&level);
        if mip < MIP_COUNT {
            level = downscale_half_atlas(&level, side);
            side /= 2;
        }
    }
    WorldAtlas { pixels, used_names }
}

/// Réduction 2× de l'atlas (boîte moyenne).
fn downscale_half_atlas(src: &[u8], side: usize) -> Vec<u8> {
    let out = side / 2;
    let mut dst = vec![0u8; out * out * 4];
    for y in 0..out {
        for x in 0..out {
            for c in 0..4 {
                let sum = |sx: usize, sy: usize| src[(sy * side + sx) * 4 + c] as u32;
                let v = (sum(x * 2, y * 2) + sum(x * 2 + 1, y * 2) + sum(x * 2, y * 2 + 1) + sum(x * 2 + 1, y * 2 + 1)) / 4;
                dst[(y * out + x) * 4 + c] = v as u8;
            }
        }
    }
    dst
}
