//! Atlas d'interface : icônes pixel-art 16×16 (agrandies ×2), cœur 64px, vignette,
//! et packer de glyphes (fontdue) pour le texte FR.

use std::collections::HashMap;
use fontdue::Font;

pub const UI_W: usize = 1024;
pub const UI_H: usize = 2048;

/// Indices des icônes dans l'atlas.
pub const IC_HEART: usize = 0;
pub const IC_CURSOR: usize = 1;
pub const IC_SOUL: usize = 2;
pub const IC_EMERALD: usize = 3;
pub const IC_ARROWS: usize = 4;
pub const IC_POTION: usize = 5;
pub const IC_FIREWORK: usize = 6;
pub const IC_HORN: usize = 7;
pub const IC_TOTEM: usize = 8;
pub const IC_SWORD: usize = 9;
pub const IC_BOW: usize = 10;
pub const IC_ARMOR: usize = 11;
pub const IC_HEAD: usize = 12;
pub const IC_COMPASS: usize = 13;
pub const IC_PIN: usize = 14;
pub const IC_CHEST: usize = 15;
pub const IC_SCROLL: usize = 16;
pub const IC_SKULL: usize = 17;
pub const N_ICONS: usize = 18;

const HEART: [&str; 16] = [
    "................",
    "...XX....XX.....",
    "..xRRX..xRRX....",
    ".xRRRRXXRRRRx...",
    ".xRRRRRRRRRRx...",
    ".xRRRRRRRRRRx...",
    ".xRRRRRRRRRRx...",
    "..xRRRRRRRRx....",
    "...xRRRRRRx.....",
    "....xRRRRx......",
    ".....xRRx.......",
    "......xx........",
    "................",
    "................",
    "................",
    "................",
];
const CURSOR: [&str; 16] = [
    "X...............",
    "XX..............",
    "XXX.............",
    "XXXX............",
    "XXXXX...........",
    "XXXXXX..........",
    "XXXXXXX.........",
    "XXXXXXXX........",
    "XXXXXXX.........",
    "XX..XXX.........",
    "X....XX.........",
    ".....XX.........",
    "......X.........",
    "................",
    "................",
    "................",
];
const SOUL: [&str; 16] = [
    "................",
    ".......g........",
    "......gGg.......",
    "......GwG.......",
    ".....gWWWg......",
    ".....GWWWG......",
    "....gWWWWWg.....",
    "....GWWWWWG.....",
    "....GWWWWWG.....",
    ".....GWWWG......",
    ".....gWwWg......",
    "......GwG.......",
    ".......g........",
    "................",
    "................",
    "................",
];
const EMERALD: [&str; 16] = [
    "................",
    "................",
    ".....dDDd.......",
    "....dEGGEd......",
    "...dEGWWGEd.....",
    "..dEGWWWWGEd....",
    "..dEGWWWWGEd....",
    "..dEGGGGGGEd....",
    "...dEGGGGEd.....",
    "....dEGGEd......",
    ".....dDDd.......",
    "................",
    "................",
    "................",
    "................",
    "................",
];
const ARROWS: [&str; 16] = [
    "..............w.",
    ".....w.......ww.",
    "....www.....ww..",
    "....www....ww...",
    "....fww...ww....",
    ".....fww.ww.....",
    "......fwww......",
    ".....wwfww......",
    "....ww.fww......",
    "...ww...fww.....",
    "..ww....fww.....",
    ".ww......fww....",
    "ww........fww...",
    "............ww..",
    "................",
    "................",
];
const POTION: [&str; 16] = [
    "................",
    "......sss.......",
    "......s.s.......",
    "......sss.......",
    ".....gg.gg......",
    ".....GgggG......",
    "....gRRRRRg.....",
    "....gRRRRRg.....",
    "....gRRRRRg.....",
    "....gRRRRRg.....",
    ".....gRRRg......",
    "......ggg.......",
    "................",
    "................",
    "................",
    "................",
];
const FIREWORK: [&str; 16] = [
    ".......F........",
    "......FFF.......",
    ".....FFFFF......",
    ".....FyFyF......",
    ".....FFFFF......",
    "......RyR.......",
    "......RRR.......",
    "......RRR.......",
    "......RRR.......",
    "......RRR.......",
    "......wRw.......",
    ".....w.R.w......",
    "................",
    "................",
    "................",
    "................",
];
const HORN: [&str; 16] = [
    "................",
    "..........gg....",
    ".........gGGg...",
    "........gGGGGg..",
    "........gGGGGg..",
    ".......gGGGGg...",
    "......gGGGGg....",
    ".....gGGGGg.....",
    "....gGGGGg......",
    "...gGGGGg.......",
    "..gGGGGg........",
    ".gggggg.........",
    ".ggggg..........",
    "................",
    "................",
    "................",
];
const TOTEM: [&str; 16] = [
    "................",
    ".....gggggg.....",
    "....gGGGGGGg....",
    "....GGeeGGGG....",
    "....GGeeGGGG....",
    "....GGGGGGGG....",
    "....GgGGGGgG....",
    "....GGrrrrGG....",
    ".....GGGGGG.....",
    "....ggGGGGgg....",
    "....gGGGGGGg....",
    "....gGGGGGGg....",
    ".....gGGGGg.....",
    "......gggg......",
    "................",
    "................",
];
const SWORD: [&str; 16] = [
    "..............W.",
    ".............WW.",
    "............WW..",
    "...........WW...",
    "..........WW....",
    ".........WW.....",
    "........WW......",
    ".......WW.......",
    "....b.WW........",
    "....bWW.........",
    "...bbb..........",
    "..bb.bb.........",
    ".bb...b.........",
    "bb..............",
    "................",
    "................",
];
const BOW: [&str; 16] = [
    ".....bbb........",
    "...bb...b.......",
    "..b......b......",
    ".b...w....b.....",
    ".b...w....b.....",
    "b....w.....b....",
    "b....w.....b....",
    "b....w.....b....",
    "b....w.....b....",
    "b....w.....b....",
    ".b...w....b.....",
    ".b...w....b.....",
    "..b......b......",
    "...bb...b.......",
    ".....bbb........",
    "................",
];
const ARMOR: [&str; 16] = [
    "................",
    "..bb........bb..",
    ".bBBb......bBBb.",
    ".bBBBbbbbbbBBBb.",
    ".bBBBBBBBBBBBBb.",
    ".bBBBBBBBBBBBBb.",
    ".bBBBBBBBBBBBBb.",
    "..bBBBBBBBBBBb..",
    "..bBBBBBBBBBBb..",
    "..bBBBBBBBBBBb..",
    "..bBBBBBBBBBBb..",
    "..bBBBBBBBBBBb..",
    "...bBBBBBBBBb...",
    "....bbbbbbbb....",
    "................",
    "................",
];
const HEAD: [&str; 16] = [
    "................",
    "...hhhhhhhh.....",
    "..hhhhhhhhhh....",
    "..hffffffffh....",
    "..hffffffffh....",
    "..hfEffffEfh....",
    "..hfEffffEfh....",
    "..hffffffffh....",
    "..hffmmmmffh....",
    "..hffffffffh....",
    "...hhhhhhhh.....",
    "................",
    "................",
    "................",
    "................",
    "................",
];
const COMPASS: [&str; 16] = [
    "................",
    "....gggggggg....",
    "...gGGGGGGGGg...",
    "..gGGGrrGGGGGg..",
    "..gGGrrrrGGGGg..",
    "..gGrrrrrrGGGg..",
    "..gGrrGGrrGGGg..",
    "..gGrGGGGrGGGg..",
    "..gGGGGGGGGGGg..",
    "..gGGGGGGGGGGg..",
    "...gGGGGGGGGg...",
    "....gggggggg....",
    "................",
    "................",
    "................",
    "................",
];
const PIN: [&str; 16] = [
    "................",
    ".....gggg.......",
    "....gGGGGg......",
    "...gGGrrGGg.....",
    "...gGrrrrGg.....",
    "...gGrrrrGg.....",
    "...gGGrrGGg.....",
    "....gGGGGg......",
    ".....gGGg.......",
    ".....gGGg.......",
    "......gg........",
    "................",
    "................",
    "................",
    "................",
    "................",
];
const CHEST: [&str; 16] = [
    "................",
    "...gggggggggg...",
    "..gGGGGGGGGGGg..",
    "..gGGGGGGGGGGg..",
    "..gggggggggggg..",
    "..gGGGyyyyGGGg..",
    "..gGGGyGGyGGGg..",
    "..gGGGyGGyGGGg..",
    "..gGGGyyyyGGGg..",
    "..gGGGGGGGGGGg..",
    "..gGGGGGGGGGGg..",
    "...gggggggggg...",
    "................",
    "................",
    "................",
    "................",
];
const SCROLL: [&str; 16] = [
    "................",
    "...wwwwwwww.....",
    "..wWWWWWWWWw....",
    "..wWWWWWWWWWw...",
    "..wWttttWWWWw...",
    "..wWWWWWWWWWw...",
    "..wWttttttWWWw..",
    "..wWWWWWWWWWw...",
    "..wWttttWWWWw...",
    "..wWWWWWWWWWw...",
    "..wWWWWWWWWw....",
    "...wwwwwwww.....",
    "................",
    "................",
    "................",
    "................",
];
const SKULL: [&str; 16] = [
    "................",
    "....wwwwwww.....",
    "...wWWWWWWWw....",
    "..wWWWWWWWWWw...",
    "..wWWWWWWWWWw...",
    "..wEWWWWWWEWw...",
    "..wEEWWWWWEEW...",
    "..wWWWWWWWWWw...",
    "...wWWWWWWWw....",
    "....wEWEWEw.....",
    "....wwwwwww.....",
    "................",
    "................",
    "................",
    "................",
    "................",
];

type IconArt = &'static [&'static str; 16];
type Palette = [(char, [u8; 4]); 8];
const ICON_DEFS: [(IconArt, Palette); N_ICONS] = [
    (HEART_REF, PALETTE_HEART),
    (CURSOR_REF, PALETTE_CURSOR),
    (SOUL_REF, PALETTE_SOUL),
    (EMERALD_REF, PALETTE_EMERALD),
    (ARROWS_REF, PALETTE_ARROWS),
    (POTION_REF, PALETTE_POTION),
    (FIREWORK_REF, PALETTE_FIREWORK),
    (HORN_REF, PALETTE_HORN),
    (TOTEM_REF, PALETTE_TOTEM),
    (SWORD_REF, PALETTE_SWORD),
    (BOW_REF, PALETTE_BOW),
    (ARMOR_REF, PALETTE_ARMOR),
    (HEAD_REF, PALETTE_HEAD),
    (COMPASS_REF, PALETTE_COMPASS),
    (PIN_REF, PALETTE_PIN),
    (CHEST_REF, PALETTE_CHEST),
    (SCROLL_REF, PALETTE_SCROLL),
    (SKULL_REF, PALETTE_SKULL),
];

const HEART_REF: &[&str; 16] = &HEART;
const CURSOR_REF: &[&str; 16] = &CURSOR;
const SOUL_REF: &[&str; 16] = &SOUL;
const EMERALD_REF: &[&str; 16] = &EMERALD;
const ARROWS_REF: &[&str; 16] = &ARROWS;
const POTION_REF: &[&str; 16] = &POTION;
const FIREWORK_REF: &[&str; 16] = &FIREWORK;
const HORN_REF: &[&str; 16] = &HORN;
const TOTEM_REF: &[&str; 16] = &TOTEM;
const SWORD_REF: &[&str; 16] = &SWORD;
const BOW_REF: &[&str; 16] = &BOW;
const ARMOR_REF: &[&str; 16] = &ARMOR;
const HEAD_REF: &[&str; 16] = &HEAD;
const COMPASS_REF: &[&str; 16] = &COMPASS;
const PIN_REF: &[&str; 16] = &PIN;
const CHEST_REF: &[&str; 16] = &CHEST;
const SCROLL_REF: &[&str; 16] = &SCROLL;
const SKULL_REF: &[&str; 16] = &SKULL;

type Pal = [(char, [u8; 4]); 8];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
const PALETTE_HEART: Pal = [
    ('x', [90, 16, 16, 255]),
    ('X', [58, 8, 8, 255]),
    ('R', [212, 42, 42, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
];
const PALETTE_CURSOR: Pal = [
    ('X', [255, 240, 200, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
    ('x', TRANSPARENT),
    ('R', TRANSPARENT),
];
const PALETTE_SOUL: Pal = [
    ('g', [46, 160, 92, 255]),
    ('G', [89, 255, 138, 255]),
    ('w', [210, 255, 226, 255]),
    ('W', [120, 255, 170, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
];
const PALETTE_EMERALD: Pal = [
    ('d', [16, 84, 44, 255]),
    ('D', [40, 140, 80, 255]),
    ('E', [52, 209, 106, 255]),
    ('G', [110, 235, 150, 255]),
    ('W', [210, 255, 226, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
];
const PALETTE_ARROWS: Pal = [
    ('w', [120, 200, 255, 255]),
    ('f', [240, 220, 160, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
    ('x', TRANSPARENT),
];
const PALETTE_POTION: Pal = [
    ('s', [170, 178, 190, 255]),
    ('g', [200, 90, 60, 255]),
    ('G', [235, 130, 100, 255]),
    ('R', [200, 40, 40, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
];
const PALETTE_FIREWORK: Pal = [
    ('F', [240, 200, 90, 255]),
    ('y', [255, 240, 180, 255]),
    ('R', [200, 60, 40, 255]),
    ('w', [255, 255, 255, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
];
const PALETTE_HORN: Pal = [
    ('g', [110, 76, 40, 255]),
    ('G', [190, 140, 78, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
    ('x', TRANSPARENT),
];
const PALETTE_TOTEM: Pal = [
    ('g', [110, 80, 30, 255]),
    ('G', [214, 168, 60, 255]),
    ('e', [60, 44, 20, 255]),
    ('r', [200, 60, 40, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
];
const PALETTE_SWORD: Pal = [
    ('W', [225, 232, 240, 255]),
    ('b', [110, 76, 40, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
    ('x', TRANSPARENT),
];
const PALETTE_BOW: Pal = [
    ('b', [140, 100, 52, 255]),
    ('w', [230, 226, 210, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
    ('x', TRANSPARENT),
];
const PALETTE_ARMOR: Pal = [
    ('b', [70, 74, 84, 255]),
    ('B', [140, 148, 164, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
    ('x', TRANSPARENT),
];
const PALETTE_HEAD: Pal = [
    ('h', [74, 52, 30, 255]),
    ('f', [198, 142, 99, 255]),
    ('E', [50, 110, 190, 255]),
    ('m', [150, 96, 62, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
];
const PALETTE_COMPASS: Pal = [
    ('g', [90, 90, 96, 255]),
    ('G', [190, 192, 200, 255]),
    ('r', [200, 50, 44, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
];
const PALETTE_PIN: Pal = [
    ('g', [40, 110, 180, 255]),
    ('G', [70, 160, 235, 255]),
    ('r', [255, 250, 230, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
];
const PALETTE_CHEST: Pal = [
    ('g', [86, 62, 34, 255]),
    ('G', [150, 108, 60, 255]),
    ('y', [222, 178, 60, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
];
const PALETTE_SCROLL: Pal = [
    ('w', [160, 130, 80, 255]),
    ('W', [232, 220, 190, 255]),
    ('t', [120, 100, 66, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
];
const PALETTE_SKULL: Pal = [
    ('w', [150, 150, 142, 255]),
    ('W', [220, 220, 210, 255]),
    ('E', [40, 40, 44, 255]),
    ('.', TRANSPARENT),
    (' ', TRANSPARENT),
    ('_', TRANSPARENT),
    ('-', TRANSPARENT),
    ('0', TRANSPARENT),
];

pub type GlyphMap = HashMap<(char, u16), ([f32; 4], f32, f32, f32)>;

pub struct UiAtlas {
    pub pixels: Vec<u8>,
    /// (uv_rect, advance, left, top) par glyphe (char, taille_px)
    /// (uv_rect, advance, left, top) par glyphe (char, taille_px)
    glyphs: GlyphMap,
    shelf_x: usize,
    shelf_y: usize,
    shelf_h: usize,
    pub dirty_glyphs: Vec<(char, u16)>,
}

impl UiAtlas {
    pub fn new(font: &Font) -> Self {
        let mut pixels = vec![0u8; UI_W * UI_H * 4];
        // Icônes 32×32 (16×16 agrandi ×2), rangées depuis (0,0)
        for (idx, (art, pal)) in ICON_DEFS.iter().enumerate() {
            let ox = (idx % 32) * 32;
            let oy = (idx / 32) * 32;
            for (ry, row) in art.iter().enumerate() {
                for (rx, ch) in row.chars().enumerate() {
                    let mut color = TRANSPARENT;
                    for (pc, pv) in pal.iter() {
                        if *pc == ch {
                            color = *pv;
                            break;
                        }
                    }
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let x = ox + rx * 2 + dx;
                            let y = oy + ry * 2 + dy;
                            let i = (y * UI_W + x) * 4;
                            pixels[i..i + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
        // Grand cœur 64×64 à (0, 128) : dessiné en agrandissant l'icône cœur ×4 avec contour sombre
        for (idx, (art, pal)) in ICON_DEFS.iter().enumerate() {
            if idx != IC_HEART {
                continue;
            }
            for (ry, row) in art.iter().enumerate() {
                for (rx, ch) in row.chars().enumerate() {
                    let mut color = TRANSPARENT;
                    for (pc, pv) in pal.iter() {
                        if *pc == ch {
                            color = *pv;
                            break;
                        }
                    }
                    if color[3] == 0 {
                        continue;
                    }
                    for dy in 0..4 {
                        for dx in 0..4 {
                            let x = rx * 4 + dx;
                            let y = 128 + ry * 4 + dy;
                            if x < 64 && y < 192 {
                                let i = (y * UI_W + x) * 4;
                                pixels[i..i + 4].copy_from_slice(&color);
                            }
                        }
                    }
                }
            }
        }
        // Pixel blanc solide (1022,0) : sert de fond aux rectangles pleins
        let solid = 1022 * 4;
        pixels[solid..solid + 4].copy_from_slice(&[255, 255, 255, 255]);
        // Vignette 512×512 à (0, 192)
        for y in 0..512 {
            for x in 0..512 {
                let dx = (x as f32 - 255.5) / 255.5;
                let dy = (y as f32 - 255.5) / 255.5;
                let d = (dx * dx + dy * dy).sqrt();
                let a = ((d - 0.55) / 0.45).clamp(0.0, 1.0).powf(1.6) * 215.0;
                let i = ((192 + y) * UI_W + x) * 4;
                pixels[i..i + 4].copy_from_slice(&[8, 4, 6, a as u8]);
            }
        }
        let mut atlas = UiAtlas {
            pixels,
            glyphs: HashMap::new(),
            shelf_x: 0,
            shelf_y: 704,
            shelf_h: 0,
            dirty_glyphs: Vec::new(),
        };
        // Pré-charge le jeu de caractères français à toutes les tailles utilisées
        let chars: Vec<char> = (" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
            .chars()
            .chain("éèêëàâçîïôûùüÿœÉÈÊÀÂÇÎÏÔÛÙŒ".chars())
            .chain("…’‘“”–—×»«".chars()))
            .collect();
        for size in [13u16, 16, 20, 26, 34, 44, 56] {
            for &ch in &chars {
                atlas.glyph(font, ch, size);
            }
        }
        atlas
    }

    /// Rect UV du glyphe : [u0, v0, u1, v1] + métriques.
    pub fn glyph(&mut self, font: &Font, ch: char, size: u16) -> ([f32; 4], f32, f32, f32) {
        let key = (ch, size);
        if let Some(&(r, adv, l, t)) = self.glyphs.get(&key) {
            return (r, adv, l, t);
        }
        let (m, cov) = font.rasterize(ch, size as f32);
        let gw = m.width.max(1);
        let gh = m.height.max(1);
        // Packer (shelf) — nouveau rayon si ça ne tient pas
        if self.shelf_x + gw + 1 > UI_W {
            self.shelf_y += self.shelf_h + 1;
            self.shelf_x = 0;
            self.shelf_h = 0;
        }
        if self.shelf_y + gh + 1 > UI_H {
            // Manque de place : réutilise (0,704) — cache partiel acceptable
            self.shelf_y = 704;
            self.shelf_x = 0;
            self.shelf_h = 0;
        }
        let (x0, y0) = (self.shelf_x, self.shelf_y);
        for gy in 0..gh {
            for gx in 0..gw {
                let a = cov.get(gy * m.width.max(1) + gx).copied().unwrap_or(0);
                let i = ((y0 + gy) * UI_W + x0 + gx) * 4;
                self.pixels[i..i + 4].copy_from_slice(&[255, 255, 255, a]);
            }
        }
        self.shelf_x += gw + 1;
        self.shelf_h = self.shelf_h.max(gh);
        let rect = [
            x0 as f32 / UI_W as f32,
            y0 as f32 / UI_H as f32,
            (x0 + gw) as f32 / UI_W as f32,
            (y0 + gh) as f32 / UI_H as f32,
        ];
        let adv = m.advance_width;
        let left = m.xmin as f32;
        let top = -m.ymin as f32;
        self.glyphs.insert(key, (rect, adv, left, top));
        self.dirty_glyphs.push((ch, size));
        (rect, adv, left, top)
    }

    pub fn icon_rect(&self, icon: usize) -> [f32; 4] {
        let s = 32.0;
        let ox = (icon % 32) as f32;
        let oy = (icon / 32) as f32;
        [
            ox * s / UI_W as f32,
            oy * s / UI_H as f32,
            (ox * s + s) / UI_W as f32,
            (oy * s + s) / UI_H as f32,
        ]
    }

    pub const VIGNETTE_RECT: [f32; 4] = [
        0.0,
        192.0 / UI_H as f32,
        512.0 / UI_W as f32,
        704.0 / UI_H as f32,
    ];
    pub const HEART_RECT: [f32; 4] = [
        0.0,
        128.0 / UI_H as f32,
        64.0 / UI_W as f32,
        192.0 / UI_H as f32,
    ];
    /// Pixel opaque unique (rectangles pleins).
    pub const SOLID_RECT: [f32; 4] = [
        1022.0 / UI_W as f32,
        0.5 / UI_H as f32,
        1023.0 / UI_W as f32,
        1.5 / UI_H as f32,
    ];
}
