//! Atlas de textures du monde : 1024×1024, grille 8×8 de tuiles 128 px.
//! Textures 100 % procédurales, remplacées si `assets/textures/<nom>.png` existe.

use crate::rng::Rng;
use image::DynamicImage;

pub const TILE: usize = 128;

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
    "soul_face",    // 38 ( visage pâle du nécromancien)
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

pub fn paint_slot(name: &str, px: &mut [u8], rng: &mut Rng) {
    let put = |x: usize, y: usize, c: [u8; 4], px: &mut [u8]| {
        if x < TILE && y < TILE {
            let i = (y * TILE + x) * 4;
            px[i..i + 4].copy_from_slice(&c);
        }
    };
    let fill = |c: [u8; 3], px: &mut [u8]| {
        for y in 0..TILE {
            for x in 0..TILE {
                put(x, y, [c[0], c[1], c[2], 255], px);
            }
        }
    };
    let grain = |rng: &mut Rng, amount: u8, px: &mut [u8]| {
        for y in 0..TILE {
            for x in 0..TILE {
                let i = (y * TILE + x) * 4;
                let n = (rng.i64(-(amount as i64), amount as i64)) as i32;
                for ch in 0..3 {
                    let v = (px[i + ch] as i32 + n).clamp(0, 255) as u8;
                    px[i + ch] = v;
                }
            }
        }
    };
    let speckles = |rng: &mut Rng, color: [u8; 3], count: usize, size: usize, px: &mut [u8]| {
        for _ in 0..count {
            let cx = rng.i64(0, (TILE - size) as i64) as usize;
            let cy = rng.i64(0, (TILE - size) as i64) as usize;
            for dy in 0..size {
                for dx in 0..size {
                    put(cx + dx, cy + dy, [color[0], color[1], color[2], 255], px);
                }
            }
        }
    };
    let blotches = |rng: &mut Rng, color: [u8; 3], count: usize, radius: usize, px: &mut [u8]| {
        for _ in 0..count {
            let cx = rng.i64(0, TILE as i64) as usize;
            let cy = rng.i64(0, TILE as i64) as usize;
            let r = radius as f32;
            for dy in 0..radius * 2 {
                for dx in 0..radius * 2 {
                    let ddx = dx as f32 - r;
                    let ddy = dy as f32 - r;
                    if ddx * ddx + ddy * ddy <= r * r {
                        let x = (cx + dx).saturating_sub(radius);
                        let y = (cy + dy).saturating_sub(radius);
                        put(x, y, [color[0], color[1], color[2], 255], px);
                    }
                }
            }
        }
    };

    match name {
        "grass_top" => {
            for yy in 0..TILE {
                for xx in 0..TILE {
                    let t = xx as f32 / 127.0;
                    let r = (255.0 * (1.0 - t)) as u8;
                    let b = (255.0 * t) as u8;
                    put(xx, yy, [r, 0, b, 255], px);
                }
            }
            grain(rng, 0, px);
        }
        "grass_side" => {
            fill([122, 91, 55], px);
            grain(rng, 8, px);
            // frange d'herbe en haut
            for x in 0..TILE {
                let h = 8 + rng.i64(0, 10) as usize;
                for y in 0..h {
                    let c = if rng.f32() > 0.5 { [76, 122, 47] } else { [62, 102, 40] };
                    put(x, y, [c[0], c[1], c[2], 255], px);
                }
            }
        }
        "dirt" | "path_dirt" => {
            let base = if name == "dirt" { [110, 82, 50] } else { [143, 106, 61] };
            fill(base, px);
            blotches(rng, [95, 70, 42], 20, 6, px);
            speckles(rng, [70, 52, 32], 40, 2, px);
            grain(rng, 6, px);
        }
        "stone" | "cobble" | "mossy_cobble" => {
            let base = match name {
                "stone" => [122, 122, 118],
                "cobble" => [110, 110, 106],
                _ => [96, 106, 88],
            };
            fill(base, px);
            // pavés irréguliers
            for row in 0..4 {
                for col in 0..4 {
                    let x0 = col * 32 + row % 2 * 8;
                    let y0 = row * 32;
                    let c = [base[0] + 18, base[1] + 18, base[2] + 16];
                    for y in 0..28 {
                        for x in 0..28 {
                            put((x0 + x) % TILE, (y0 + y) % TILE, [c[0], c[1], c[2], 255], px);
                        }
                    }
                }
            }
            if name == "mossy_cobble" {
                blotches(rng, [77, 122, 55], 14, 8, px);
            }
            grain(rng, 7, px);
        }
        "mossy_stone" => {
            fill([104, 116, 96], px);
            blotches(rng, [77, 122, 55], 22, 9, px);
            blotches(rng, [62, 90, 48], 14, 6, px);
            grain(rng, 6, px);
        }
        "gravel" => {
            fill([104, 100, 94], px);
            speckles(rng, [130, 126, 120], 160, 3, px);
            speckles(rng, [72, 68, 64], 160, 3, px);
            grain(rng, 5, px);
        }
        "trunk_side" => {
            fill([90, 68, 38], px);
            for x in 0..TILE {
                if x % 11 < 2 {
                    for y in 0..TILE {
                        put(x, y, [63, 47, 26, 255], px);
                    }
                }
            }
            grain(rng, 6, px);
        }
        "trunk_top" => {
            fill([140, 108, 64], px);
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = x as f32 - 64.0;
                    let dy = y as f32 - 64.0;
                    let d = (dx * dx + dy * dy).sqrt();
                    if (d as i32 / 9) % 2 == 0 {
                        put(x, y, [104, 78, 44, 255], px);
                    }
                }
            }
        }
        "leaves_oak" | "leaves_dark" => {
            let base = if name == "leaves_oak" { [47, 91, 34] } else { [34, 64, 26] };
            fill(base, px);
            let light = if name == "leaves_oak" { [78, 138, 46] } else { [52, 96, 38] };
            blotches(rng, light, 30, 7, px);
            blotches(rng, [20, 40, 16], 26, 8, px);
            grain(rng, 9, px);
        }
        "bush_red" => {
            fill([160, 48, 40], px);
            blotches(rng, [126, 30, 26], 20, 8, px);
            blotches(rng, [196, 70, 54], 16, 5, px);
            grain(rng, 8, px);
        }
        "plank" => {
            fill([158, 120, 72], px);
            for row in 0..4 {
                let y0 = row * 32;
                for y in 0..32 {
                    for x in 0..TILE {
                        let c = if y == 0 || y == 31 { [116, 86, 50] } else { [158, 120, 72] };
                        put(x, y0 + y, [c[0], c[1], c[2], 255], px);
                    }
                }
                // joints décalés
                let j = (row * 37 + 20) % TILE;
                for y in 1..31 {
                    put(j, y0 + y, [116, 86, 50, 255], px);
                }
            }
            grain(rng, 5, px);
        }
        "roof_dark" | "roof_red" => {
            let base = if name == "roof_dark" { [58, 52, 60] } else { [138, 58, 44] };
            fill(base, px);
            for row in 0..8 {
                let y0 = row * 16;
                for y in 0..16 {
                    for x in 0..TILE {
                        let c = if y < 3 { [base[0] + 24, base[1] + 20, base[2] + 22] } else { base };
                        put(x, y0 + y, [c[0], c[1], c[2], 255], px);
                    }
                }
                for x in (row % 2 * 16..TILE).step_by(32) {
                    for y in 0..16 {
                        put(x, y0 + y, [base[0] - 20, base[1] - 18, base[2] - 18, 255], px);
                    }
                }
            }
        }
        "water" => {
            fill([63, 111, 174], px);
            blotches(rng, [111, 159, 217], 18, 10, px);
            blotches(rng, [44, 82, 138], 16, 9, px);
            grain(rng, 4, px);
        }
        "lava" => {
            fill([212, 59, 15], px);
            blotches(rng, [255, 122, 31], 22, 9, px);
            blotches(rng, [255, 196, 64], 10, 5, px);
            blotches(rng, [130, 28, 8], 12, 7, px);
            grain(rng, 6, px);
        }
        "mushroom_stem" => {
            fill([222, 214, 190], px);
            blotches(rng, [190, 180, 156], 10, 6, px);
            grain(rng, 4, px);
        }
        "mushroom_cap" => {
            fill([176, 58, 42], px);
            speckles(rng, [236, 230, 214], 14, 7, px);
            blotches(rng, [140, 38, 28], 8, 7, px);
            grain(rng, 5, px);
        }
        "glow_moss" => {
            fill([72, 100, 52], px);
            blotches(rng, [142, 214, 90], 24, 6, px);
            speckles(rng, [214, 255, 150], 60, 2, px);
        }
        "rune_stone" => {
            fill([96, 100, 92], px);
            grain(rng, 6, px);
            // rune gravée lumineuse
            let rune = [
                "....XXXX....",
                "...X....X...",
                "...X....X...",
                "....XXXX....",
                "...X....X...",
                "..X......X..",
            ];
            let ox = 16;
            let oy = 36;
            for (ry, row) in rune.iter().enumerate() {
                for (rx, ch) in row.chars().enumerate() {
                    if ch == 'X' {
                        for dy in 0..8 {
                            for dx in 0..8 {
                                put(ox + rx * 8 + dx, oy + ry * 8 + dy, [120, 220, 255, 255], px);
                            }
                        }
                    }
                }
            }
        }
        "tomb_stone" => {
            fill([107, 111, 102], px);
            blotches(rng, [86, 90, 84], 10, 8, px);
            for y in 40..52 {
                for x in 30..98 {
                    put(x, y, [88, 92, 86, 255], px);
                }
            }
            grain(rng, 5, px);
        }
        "temple_brick" => {
            fill([60, 58, 69], px);
            for row in 0..8 {
                let y0 = row * 16;
                for x in 0..TILE {
                    put(x, y0, [40, 38, 47, 255], px);
                }
                let off = row % 2 * 16;
                for x in (off..TILE).step_by(32) {
                    for y in 0..16 {
                        put(x, y0 + y, [40, 38, 47, 255], px);
                    }
                }
            }
            blotches(rng, [82, 80, 96], 10, 5, px);
            grain(rng, 5, px);
        }
        "gold_block" => {
            fill([222, 178, 60], px);
            for i in 0..TILE {
                put(i, 0, [250, 214, 120, 255], px);
                put(i, 1, [250, 214, 120, 255], px);
                put(0, i, [178, 136, 40, 255], px);
                put(1, i, [178, 136, 40, 255], px);
            }
            grain(rng, 5, px);
        }
        "bookshelf" => {
            fill([122, 91, 55], px);
            for shelf in 0..3 {
                let y0 = 6 + shelf * 42;
                for x in 4..124 {
                    for y in y0..y0 + 30 {
                        put(x, y, [158, 120, 72, 255], px);
                    }
                }
                let mut x = 8;
                while x < 118 {
                    let w = rng.i64(5, 11) as usize;
                    let c: [u8; 3] = match rng.i64(0, 4) {
                        0 => [148, 62, 48],
                        1 => [58, 96, 138],
                        2 => [86, 128, 56],
                        _ => [152, 122, 60],
                    };
                    for dx in 0..w {
                        for y in 0..26 {
                            put(x + dx, y0 + 2 + y, [c[0], c[1], c[2], 255], px);
                        }
                    }
                    x += w + 2;
                }
            }
        }
        "door" => {
            fill([122, 91, 55], px);
            for y in 0..TILE {
                for x in 0..TILE {
                    let edge = !(6..=121).contains(&x) || y < 6;
                    let c = if edge { [86, 62, 36] } else { [140, 106, 64] };
                    put(x, y, [c[0], c[1], c[2], 255], px);
                }
            }
            put(106, 64, [222, 178, 60, 255], px);
        }
        "banner" => {
            fill([138, 58, 44], px);
            for y in 0..TILE {
                for x in 0..TILE {
                    if (x / 16) % 2 == 0 {
                        put(x, y, [176, 74, 56, 255], px);
                    }
                }
            }
        }
        "obsidian" => {
            fill([34, 28, 46], px);
            blotches(rng, [58, 46, 84], 14, 6, px);
            speckles(rng, [92, 70, 140], 26, 2, px);
        }
        "redstone" => {
            fill([96, 40, 32], px);
            speckles(rng, [214, 60, 40], 90, 3, px);
            speckles(rng, [255, 120, 90], 30, 2, px);
        }
        "web" => {
            fill([0, 0, 0], px);
            for i in 0..4 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let (dx, dy) = (a.cos(), a.sin());
                for t in 0..TILE / 2 {
                    let x = 64 + (dx * t as f32) as usize;
                    let y = 64 + (dy * t as f32) as usize;
                    put(x, y, [225, 225, 225, 255], px);
                    put(TILE - x.min(TILE - 1), y, [225, 225, 225, 255], px);
                }
            }
            for r in [24usize, 44, 60] {
                for a in 0..360 {
                    let rad = a as f32 * std::f32::consts::PI / 180.0;
                    let x = (64.0 + rad.cos() * r as f32) as usize;
                    let y = (64.0 + rad.sin() * r as f32) as usize;
                    if x < TILE && y < TILE {
                        put(x, y, [225, 225, 225, 255], px);
                    }
                }
            }
        }
        "hay" => {
            fill([196, 160, 62], px);
            for y in (0..TILE).step_by(10) {
                for x in 0..TILE {
                    put(x, y, [160, 128, 46, 255], px);
                }
            }
            grain(rng, 6, px);
        }
        "soul_face" => {
            fill([198, 160, 120], px);
            grain(rng, 5, px);
            // yeux sombres enfoncés
            for (ex, ey) in [(36, 56), (80, 56)] {
                for dy in 0..14 {
                    for dx in 0..12 {
                        put(ex + dx, ey + dy, [40, 30, 34, 255], px);
                    }
                }
            }
            for y in 92..100 {
                for x in 44..84 {
                    put(x, y, [70, 48, 44, 255], px);
                }
            }
        }
        "exit_portal" => {
            fill([20, 10, 34], px);
            blotches(rng, [96, 40, 190], 18, 10, px);
            blotches(rng, [160, 90, 255], 12, 6, px);
            speckles(rng, [220, 180, 255], 40, 2, px);
        }
        // ---- Cases spéciales à dégradé (particules/ombres) ----
        "shadow" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = (x as f32 - 63.5) / 63.5;
                    let dy = (y as f32 - 63.5) / 63.5;
                    let d = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
                    let a = ((1.0 - d) * 255.0 * 1.4).clamp(0.0, 255.0) as u8;
                    put(x, y, [10, 12, 10, a], px);
                }
            }
        }
        "glow_disc" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = (x as f32 - 63.5) / 63.5;
                    let dy = (y as f32 - 63.5) / 63.5;
                    let d = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
                    let a = ((1.0 - d) * (1.0 - d) * 255.0) as u8;
                    put(x, y, [255, 255, 255, a], px);
                }
            }
        }
        "star" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = (x as f32 - 63.5).abs();
                    let dy = (y as f32 - 63.5).abs();
                    let v = (1.0 - (dx + dy) / 63.5).clamp(0.0, 1.0);
                    let a = (v.powf(3.0) * 255.0) as u8;
                    put(x, y, [255, 255, 255, a], px);
                }
            }
        }
        "ring" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = x as f32 - 63.5;
                    let dy = y as f32 - 63.5;
                    let d = (dx * dx + dy * dy).sqrt();
                    let v = (1.0 - (d - 52.0).abs() / 8.0).clamp(0.0, 1.0);
                    put(x, y, [255, 255, 255, (v * 255.0) as u8], px);
                }
            }
        }
        "leaf_sprite" => {
            for y in 0..TILE {
                for x in 0..TILE {
                    let dx = (x as f32 - 63.5) / 63.5;
                    let dy = (y as f32 - 63.5) / 63.5;
                    let v = 1.0 - (dx * dx + dy * dy).sqrt();
                    let g = (120.0 + v * 110.0).clamp(0.0, 255.0) as u8;
                    let a = if v > 0.25 { 255 } else { 0 };
                    put(x, y, [g / 2, g, g / 3, a], px);
                }
            }
        }
        _ => {
            fill([200, 60, 200], px); // case inconnue : rose vif, facile à repérer
        }
    }
}

pub fn build(atlas_dir: &str) -> WorldAtlas {
    let mut atlas = vec![0u8; ATLAS_SIZE * ATLAS_SIZE * 4];
    let mut used_names = Vec::new();
    for (i, name) in SLOT_NAMES.iter().enumerate() {
        let mut buf = vec![0u8; TILE * TILE * 4];
        let mut rng = Rng::new(0x9E37_79B9 + i as u64);
        paint_slot(name, &mut buf, &mut rng);
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
