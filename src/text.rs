//! Rendu de texte (batch de quads UI) avec contour, alignement centre possible.

use crate::ui_atlas::{UiAtlas, UI_W};
use crate::vertex::UiVertex;
use fontdue::Font;

const RASTER_SIZES: [u16; 7] = [13, 16, 20, 26, 34, 44, 56];

fn nearest_raster(size: f32) -> u16 {
    let mut best = RASTER_SIZES[0];
    let mut best_d = f32::MAX;
    for s in RASTER_SIZES {
        let d = (s as f32 - size).abs();
        if d < best_d {
            best_d = d;
            best = s;
        }
    }
    best
}

#[derive(Default)]
pub struct TextBatch {
    pub verts: Vec<UiVertex>,
}

impl TextBatch {
    /// Largeur mesurée du texte (pour l'alignement à droite).
    pub fn width(atlas: &mut UiAtlas, font: &Font, size: f32, text: &str) -> f32 {
        let raster = nearest_raster(size);
        let scale = size / raster as f32;
        let mut w = 0.0;
        for ch in text.chars() {
            if ch == ' ' {
                w += size * 0.32;
                continue;
            }
            let (_, adv, _, _) = atlas.glyph(font, ch, raster);
            w += adv * scale;
        }
        w
    }

    /// Dessine `text` ; `x,y` = coin haut-gauche (ou centre si `center_x`).
    /// `outline` ajoute un contour noir épais (style Minecraft Dungeons).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        atlas: &mut UiAtlas,
        font: &Font,
        x: f32,
        y: f32,
        size: f32,
        text: &str,
        color: [f32; 4],
        outline: bool,
        center_x: bool,
    ) {
        let raster = nearest_raster(size);
        let scale = size / raster as f32;

        let width_total = if center_x {
            let mut w = 0.0;
            for ch in text.chars() {
                let (r, adv, _, _) = atlas.glyph(font, ch, raster);
                w += if ch == ' ' { size * 0.32 } else { adv * scale };
                let _ = r;
            }
            w
        } else {
            0.0
        };
        let mut pen_x = if center_x { x - width_total * 0.5 } else { x };

        for ch in text.chars() {
            if ch == ' ' {
                pen_x += size * 0.32;
                continue;
            }
            let (r, adv, left, top) = atlas.glyph(font, ch, raster);
            let gw = (r[2] - r[0]) * UI_W as f32 * scale;
            let gh = (r[3] - r[1]) * UI_W as f32 * scale;
            let gx = pen_x + left * scale;
            let gy = y + top * scale;
            if outline {
                for (ox, oy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0), (-2.0, -2.0), (2.0, 2.0), (-2.0, 2.0), (2.0, -2.0)] {
                    Self::push_quad(
                        &mut self.verts,
                        gx + ox,
                        gy + oy,
                        gw,
                        gh,
                        r,
                        [0.02, 0.02, 0.03, color[3]],
                    );
                }
            }
            Self::push_quad(&mut self.verts, gx, gy, gw, gh, r, color);
            pen_x += adv * scale;
        }
    }

    fn push_quad(verts: &mut Vec<UiVertex>, x: f32, y: f32, w: f32, h: f32, r: [f32; 4], c: [f32; 4]) {
        let v = |px: f32, py: f32, u: f32, vv: f32| UiVertex {
            pos: [px, py],
            uv: [u, vv],
            color: c,
        };
        verts.push(v(x, y + h, r[0], r[3]));
        verts.push(v(x + w, y + h, r[2], r[3]));
        verts.push(v(x + w, y, r[2], r[1]));
        verts.push(v(x, y + h, r[0], r[3]));
        verts.push(v(x + w, y, r[2], r[1]));
        verts.push(v(x, y, r[0], r[1]));
    }
}
