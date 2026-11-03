//! Interface (HUD fidèle aux captures + écrans FR) : cœur géant, artefacts,
//! âmes, émeraudes, objectif, barre de boss, carte, inventaire, menus.

use crate::game::{Game, Screen};
use crate::player::MAX_SOULS;
use crate::ui_atlas::{UiAtlas, IC_ARMOR, IC_ARROWS, IC_BOW, IC_CHEST, IC_COMPASS, IC_CURSOR, IC_EMERALD, IC_FIREWORK, IC_HEAD, IC_HORN, IC_PIN, IC_POTION, IC_SCROLL, IC_SKULL, IC_SOUL, IC_SWORD, IC_TOTEM};
use crate::vertex::UiVertex;
use fontdue::Font;
use glam::Vec4;

const GOLD: [f32; 4] = [0.91, 0.72, 0.29, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const PANEL: [f32; 4] = [0.055, 0.055, 0.07, 0.86];
const PANEL_BORDER: [f32; 4] = [0.79, 0.64, 0.15, 0.95];
const GREEN_TXT: [f32; 4] = [0.35, 1.0, 0.54, 1.0];
const PURPLE: [f32; 4] = [0.61, 0.36, 0.86, 1.0];

struct Ui<'a> {
    out: &'a mut Vec<UiVertex>,
    atlas: &'a mut UiAtlas,
    font: &'a Font,
    w: f32,
    h: f32,
    s: f32,
}

impl<'a> Ui<'a> {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: [f32; 4]) {
        // quad plein : échantillonne le pixel blanc solide de l'atlas UI
        let r = UiAtlas::SOLID_RECT;
        self.uv(r, x, y, w, h, c);
    }

    /// Quad texturé (rectangle UV de l'atlas UI).
    fn uv(&mut self, r: [f32; 4], x: f32, y: f32, w: f32, h: f32, c: [f32; 4]) {
        let v = |px: f32, py: f32, u: f32, vv: f32, c: [f32; 4]| UiVertex {
            pos: [px, py],
            uv: [u, vv],
            color: c,
        };
        self.out.push(v(x, y + h, r[0], r[3], c));
        self.out.push(v(x + w, y + h, r[2], r[3], c));
        self.out.push(v(x + w, y, r[2], r[1], c));
        self.out.push(v(x, y + h, r[0], r[3], c));
        self.out.push(v(x + w, y, r[2], r[1], c));
        self.out.push(v(x, y, r[0], r[1], c));
    }

    fn icon(&mut self, icon: usize, x: f32, y: f32, size: f32, c: [f32; 4]) {
        let r = self.atlas.icon_rect(icon);
        self.uv(r, x, y, size, size, c);
    }

    fn heart(&mut self, x: f32, y: f32, size: f32, c: [f32; 4]) {
        let r = UiAtlas::HEART_RECT;
        self.uv(r, x, y, size, size, c);
    }

    fn text(&mut self, x: f32, y: f32, size: f32, s: &str, c: [f32; 4], center: bool) {
        let mut tb = crate::text::TextBatch::default();
        tb.draw(self.atlas, self.font, x, y, size, s, c, true, center);
        self.out.append(&mut tb.verts);
    }

    fn vignette(&mut self, strength: f32) {
        let r = UiAtlas::VIGNETTE_RECT;
        self.uv(r, 0.0, 0.0, self.w, self.h, [1.0, 1.0, 1.0, strength]);
    }
}

/// Construit toute l'interface de la frame.
pub fn draw(g: &Game, atlas: &mut UiAtlas, font: &Font, w: f32, h: f32, out: &mut Vec<UiVertex>) {
    let mut ui = Ui { out, atlas, font, w, h, s: (h / 1080.0).clamp(0.55, 2.0) };
    match g.screen {
        Screen::Title => draw_title(&mut ui, g),
        Screen::Briefing => draw_briefing(&mut ui, g),
        Screen::Playing => {
            draw_hud(&mut ui, g);
            draw_world_anchored(&mut ui, g);
            draw_vignette_state(&mut ui, g);
        }
        Screen::Paused => {
            draw_hud(&mut ui, g);
            dim(&mut ui, 0.55);
            draw_paused(&mut ui, g);
        }
        Screen::Map => {
            draw_map(&mut ui, g);
        }
        Screen::Inventory => {
            draw_inventory(&mut ui, g);
        }
        Screen::Controls => draw_controls(&mut ui),
        Screen::GameOver => {
            draw_hud(&mut ui, g);
            dim(&mut ui, 0.6);
            draw_game_over(&mut ui, g);
        }
        Screen::Victory => draw_victory(&mut ui, g),
    }
}

fn dim(ui: &mut Ui, a: f32) {
    ui.rect(0.0, 0.0, ui.w, ui.h, [0.0, 0.0, 0.0, a]);
}

// ==================== HUD EN JEU ====================

fn draw_hud(ui: &mut Ui, g: &Game) {
    let s = ui.s;
    let p = &g.player;
    let bar_h = 118.0 * s;
    let bar_w = 780.0 * s;
    let bar_x = (ui.w - bar_w) * 0.5;
    let bar_y = ui.h - bar_h - 14.0 * s;

    // fond de barre
    ui.rect(bar_x, bar_y, bar_w, bar_h, [0.04, 0.04, 0.05, 0.82]);
    ui.rect(bar_x, bar_y, bar_w, 3.0 * s, [0.35, 0.3, 0.12, 0.9]);

    // --- âmes (gauche) ---
    let soul_x = bar_x + 18.0 * s;
    ui.icon(IC_SOUL, soul_x, bar_y + 30.0 * s, 40.0 * s, WHITE);
    ui.text(soul_x + 46.0 * s, bar_y + 34.0 * s, 22.0 * s, &format!("{}/{}", p.souls, MAX_SOULS), GREEN_TXT, false);

    // --- 3 artefacts ---
    let icons = [IC_FIREWORK, IC_HORN, IC_TOTEM];
    let cds = [12.0f32, 18.0, 45.0];
    for i in 0..3 {
        let x = bar_x + 110.0 * s + i as f32 * 78.0 * s;
        slot(ui, x, bar_y + 20.0 * s, 64.0 * s, icons[i], &format!("{}", i + 1));
        let cd = p.artifact_cd[i];
        if cd > 0.0 {
            let frac = (cd / cds[i]).clamp(0.0, 1.0);
            ui.rect(x, bar_y + 20.0 * s, 64.0 * s, 64.0 * s * frac, [0.0, 0.0, 0.0, 0.62]);
        }
    }

    // --- cœur géant + vies + niveau/XP ---
    let heart_size = 96.0 * s;
    let heart_x = (ui.w - heart_size) * 0.5;
    let heart_y = bar_y + 8.0 * s;
    let pulse = if p.hp < 30.0 { (g.time * 6.0).sin() * 0.5 + 0.5 } else { 0.0 };
    let hs = heart_size * (1.0 + pulse * 0.08);
    let low = p.hp < 30.0;
    let hc = if low {
        [1.0, 0.25 + pulse * 0.3, 0.25, 1.0]
    } else {
        [1.0, 0.42, 0.42, 1.0]
    };
    ui.heart(heart_x - (hs - heart_size) * 0.5, heart_y - (hs - heart_size) * 0.5, hs, hc);
    // vies (têtes de héros)
    for i in 0..p.lives {
        ui.icon(IC_HEAD, heart_x - 46.0 * s + i as f32 * 26.0 * s, bar_y + 74.0 * s, 22.0 * s, WHITE);
    }
    ui.text(heart_x - 40.0 * s, bar_y + 72.0 * s, 20.0 * s, &format!("×{}", p.lives), WHITE, false);
    // niveau + barre XP
    ui.text(heart_x + heart_size + 14.0 * s, bar_y + 44.0 * s, 20.0 * s, &format!("Niv. {}", p.level), [0.85, 0.7, 1.0, 1.0], false);
    let xp_w = 120.0 * s;
    let need = (p.level * 30) as f32;
    ui.rect(heart_x + heart_size + 14.0 * s, bar_y + 72.0 * s, xp_w, 10.0 * s, [0.1, 0.08, 0.14, 0.9]);
    ui.rect(heart_x + heart_size + 14.0 * s, bar_y + 72.0 * s, xp_w * (p.xp as f32 / need).clamp(0.0, 1.0), 10.0 * s, PURPLE);

    // --- potion ---
    let pot_x = heart_x + heart_size + 170.0 * s;
    slot(ui, pot_x, bar_y + 20.0 * s, 64.0 * s, IC_POTION, "E");
    if p.potion_cd > 0.0 {
        let frac = (p.potion_cd / 30.0).clamp(0.0, 1.0);
        ui.rect(pot_x, bar_y + 20.0 * s, 64.0 * s, 64.0 * s * frac, [0.0, 0.0, 0.0, 0.62]);
    }

    // --- flèches ---
    let arr_x = pot_x + 92.0 * s;
    slot(ui, arr_x, bar_y + 20.0 * s, 64.0 * s, IC_ARROWS, "Clic D");
    ui.text(arr_x + 64.0 * s - 8.0 * s, bar_y - 2.0 * s, 20.0 * s, &format!("{}", p.arrows), WHITE, false);

    // --- émeraudes (droite) ---
    let em_w = 150.0 * s;
    ui.icon(IC_EMERALD, bar_x + bar_w - em_w, bar_y + 32.0 * s, 36.0 * s, WHITE);
    ui.text(bar_x + bar_w - em_w + 42.0 * s, bar_y + 36.0 * s, 24.0 * s, &format!("{}", p.emeralds), GREEN_TXT, false);

    // --- objectif (haut droite, aligné au bord) ---
    let obj = g.quest.label();
    let obj_w = crate::text::TextBatch::width(ui.atlas, ui.font, 27.0 * s, &obj);
    ui.text(ui.w - 30.0 * s - obj_w, 26.0 * s, 27.0 * s, &obj, WHITE, false);
    let zone = g.level.zone_name(g.player.pos);
    let zone_w = crate::text::TextBatch::width(ui.atlas, ui.font, 19.0 * s, zone);
    ui.text(ui.w - 30.0 * s - zone_w, 62.0 * s, 19.0 * s, zone, GOLD, false);

    // --- barre de boss ---
    if g.boss_alive {
        let bw = 620.0 * s;
        let bx = (ui.w - bw) * 0.5;
        ui.text(ui.w * 0.5, 30.0 * s, 30.0 * s, "LE NÉCROMANCIEN", [0.9, 0.75, 1.0, 1.0], true);
        ui.rect(bx - 3.0 * s, 74.0 * s, bw + 6.0 * s, 22.0 * s + 6.0 * s, [0.06, 0.05, 0.08, 0.9]);
        if let Some(boss) = g.mobs.iter().find(|m| m.kind == crate::mobs::MobKind::Necromancer) {
            let frac = (boss.hp / boss.max_hp).clamp(0.0, 1.0);
            ui.rect(bx, 77.0 * s, bw * frac, 22.0 * s, [0.78, 0.12, 0.12, 1.0]);
        }
    }

    // --- textes flottants (dégâts) ---
    let vp = g.cam.view_proj();
    for t in &g.float_texts {
        if let Some((x, y)) = project(&vp, t.pos, ui.w, ui.h) {
            let a = (t.life * 2.0).clamp(0.0, 1.0);
            ui.text(x, y, 24.0 * s, &t.text, [t.color[0], t.color[1], t.color[2], a], true);
        }
    }
}

fn slot(ui: &mut Ui, x: f32, y: f32, size: f32, icon: usize, key: &str) {
    ui.rect(x - 3.0, y - 3.0, size + 6.0, size + 6.0, PANEL_BORDER);
    ui.rect(x, y, size, size, PANEL);
    ui.icon(icon, x + size * 0.12, y + size * 0.12, size * 0.76, WHITE);
    ui.text(x + 4.0, y + size - 20.0 * ui.s, 15.0 * ui.s, key, GOLD, false);
}

fn project(vp: &glam::Mat4, pos: glam::Vec3, w: f32, h: f32) -> Option<(f32, f32)> {
    let c = *vp * Vec4::new(pos.x, pos.y, pos.z, 1.0);
    if c.w <= 0.05 {
        return None;
    }
    let ndc = c / c.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }
    Some(((ndc.x * 0.5 + 0.5) * w, (1.0 - (ndc.y * 0.5 + 0.5)) * h))
}

fn draw_world_anchored(ui: &mut Ui, g: &Game) {
    let s = ui.s;
    // prompt d'interaction au-dessus du héros
    if let Some(pr) = &g.prompt {
        let vp = g.cam.view_proj();
        if let Some((x, y)) = project(&vp, g.player.pos + glam::Vec3::new(0.0, 2.6, 0.0), ui.w, ui.h) {
            ui.text(x, y - 30.0 * s, 22.0 * s, pr, GOLD, true);
        }
    }
}

fn draw_vignette_state(ui: &mut Ui, g: &Game) {
    ui.vignette(0.55);
    if g.damage_flash > 0.0 {
        ui.rect(0.0, 0.0, ui.w, ui.h, [0.75, 0.05, 0.05, g.damage_flash * 0.34]);
    }
    if g.player.hp < 30.0 && !g.player.dead {
        let pulse = ((g.time * 5.0).sin() * 0.5 + 0.5) * 0.16;
        ui.rect(0.0, 0.0, ui.w, ui.h, [0.6, 0.03, 0.03, pulse]);
    }
}

// ==================== ÉCRANS ====================

fn draw_title(ui: &mut Ui, g: &Game) {
    ui.vignette(0.75);
    // logo façon key art
    let cy = ui.h * 0.24;
    ui.text(ui.w * 0.5, cy, 66.0 * ui.s, "MINECRAFT", [0.82, 0.84, 0.88, 1.0], true);
    ui.text(ui.w * 0.5, cy + 80.0 * ui.s, 84.0 * ui.s, "DUNGEONS", [1.0, 0.72, 0.2, 1.0], true);
    ui.text(ui.w * 0.5, cy + 172.0 * ui.s, 24.0 * ui.s, "— Recréation non officielle à but pédagogique —", [0.7, 0.7, 0.7, 0.85], true);

    // menu
    let items = ["JOUER — CREEPER WOODS", "COMMANDES", "QUITTER"];
    let mx = ui.w * 0.5;
    let mut my = ui.h * 0.55;
    for (i, it) in items.iter().enumerate() {
        let sel = g.title_cursor == i;
        let c = if sel { GOLD } else { WHITE };
        let size = if sel { 34.0 * ui.s } else { 30.0 * ui.s };
        ui.text(mx, my, size, it, c, true);
        if sel {
            let tw = 34.0 * ui.s * it.len() as f32 * 0.62;
            ui.icon(IC_CURSOR, mx - tw * 0.5 - 40.0 * ui.s, my + 2.0 * ui.s, 26.0 * ui.s, WHITE);
        }
        my += 58.0 * ui.s;
    }
    ui.text(ui.w * 0.5, ui.h - 40.0 * ui.s, 18.0 * ui.s, "Z/S pour naviguer — Entrée pour choisir", [0.75, 0.75, 0.75, 0.9], true);
}

fn draw_briefing(ui: &mut Ui, g: &Game) {
    ui.vignette(0.85);
    dim(ui, 0.35);
    ui.text(ui.w * 0.5, ui.h * 0.3, 52.0 * ui.s, "CREEPER WOODS", GOLD, true);
    let lines = [
        "La caravane ne répond plus…",
        "Traversez les bois, libérez les villageois",
        "et retrouvez la caravane avant la nuit.",
        "Méfiez-vous des creepers.",
    ];
    let mut y = ui.h * 0.42;
    for l in lines {
        ui.text(ui.w * 0.5, y, 26.0 * ui.s, l, WHITE, true);
        y += 44.0 * ui.s;
    }
    ui.text(ui.w * 0.5, ui.h * 0.72, 22.0 * ui.s, "Appuyez sur Entrée", [0.8, 0.8, 0.8, 1.0], true);
    let _ = g;
}

fn draw_paused(ui: &mut Ui, g: &Game) {
    ui.text(ui.w * 0.5, ui.h * 0.3, 54.0 * ui.s, "PAUSE", WHITE, true);
    let items = [
        "Échap — Reprendre",
        "M — Carte",
        "I — Inventaire",
        "C — Commandes",
    ];
    let mut y = ui.h * 0.44;
    for it in items {
        ui.text(ui.w * 0.5, y, 27.0 * ui.s, it, GOLD, true);
        y += 48.0 * ui.s;
    }
    let _ = g;
}

fn draw_game_over(ui: &mut Ui, g: &Game) {
    ui.text(ui.w * 0.5, ui.h * 0.36, 56.0 * ui.s, "VOUS ÊTES TOMBÉ…", [1.0, 0.3, 0.3, 1.0], true);
    ui.text(ui.w * 0.5, ui.h * 0.48, 26.0 * ui.s, "Les bois ont eu raison de vous.", WHITE, true);
    ui.text(ui.w * 0.5, ui.h * 0.56, 24.0 * ui.s, "Entrée — Retenter la mission", GOLD, true);
    let _ = g;
}

fn draw_victory(ui: &mut Ui, g: &Game) {
    ui.vignette(0.7);
    ui.text(ui.w * 0.5, ui.h * 0.24, 56.0 * ui.s, "MISSION ACCOMPLIE !", GOLD, true);
    let mins = (g.victory_time / 60.0).floor() as i32;
    let secs = (g.victory_time % 60.0) as i32;
    let lines = [
        format!("Temps : {mins:02}:{secs:02}"),
        format!("Émeraudes : {}", g.player.emeralds),
        format!("Niveau atteint : {}", g.player.level),
        format!("Vies restantes : {}", g.player.lives),
    ];
    let mut y = ui.h * 0.4;
    for l in lines {
        ui.text(ui.w * 0.5, y, 28.0 * ui.s, &l, WHITE, true);
        y += 46.0 * ui.s;
    }
    ui.text(ui.w * 0.5, ui.h * 0.68, 22.0 * ui.s, "Entrée — Retour au titre", GOLD, true);
}

fn draw_controls(ui: &mut Ui) {
    dim(ui, 0.72);
    ui.text(ui.w * 0.5, 60.0 * ui.s, 44.0 * ui.s, "COMMANDES", GOLD, true);
    let rows = [
        ("Se déplacer", "Z Q S D (touches physiques)"),
        ("Mêlée / Interagir", "Clic gauche / F"),
        ("Arc (auto-visée)", "Clic droit"),
        ("Roulade (esquive)", "Espace"),
        ("Potion (+75 PV)", "E"),
        ("Artefacts", "1 / 2 / 3"),
        ("Carte", "M"),
        ("Inventaire", "I"),
        ("Pause", "Échap"),
        ("Capture d'écran", "F2"),
    ];
    let mut y = ui.h * 0.22;
    for (k, v) in rows {
        ui.text(ui.w * 0.32, y, 24.0 * ui.s, k, WHITE, false);
        ui.text(ui.w * 0.52, y, 24.0 * ui.s, v, GOLD, false);
        y += 42.0 * ui.s;
    }
    ui.text(ui.w * 0.5, ui.h - 50.0 * ui.s, 20.0 * ui.s, "Échap — Retour", [0.8, 0.8, 0.8, 1.0], true);
}

// ==================== CARTE ====================

fn draw_map(ui: &mut Ui, g: &Game) {
    ui.rect(0.0, 0.0, ui.w, ui.h, [0.07, 0.09, 0.11, 0.96]);
    let (minx, maxx, minz, maxz) = g.level.bounds;
    let pad = 90.0 * ui.s;
    let mw = ui.w - pad * 2.0;
    let mh = ui.h - pad * 2.0;
    let scale = (mw / (maxx - minx)).min(mh / (maxz - minz));
    let ox = pad + (mw - (maxx - minx) * scale) * 0.5;
    let oy = pad + (mh - (maxz - minz) * scale) * 0.5;
    let tx = |x: f32| ox + (x - minx) * scale;
    let tz = |z: f32| oy + (z - minz) * scale;

    ui.text(ui.w * 0.5, 20.0 * ui.s, 40.0 * ui.s, "CREEPER WOODS — CARTE", GOLD, true);

    // eau + lave
    for r in &g.level.water_rects {
        ui.rect(tx(r[0]), tz(r[1]), r[2] * scale, r[3] * scale, [0.25, 0.44, 0.68, 0.9]);
    }
    for r in &g.level.lava_rects {
        ui.rect(tx(r[0]), tz(r[1]), r[2] * scale, r[3] * scale, [0.85, 0.35, 0.1, 0.95]);
    }
    // chemin
    for w in g.level.waypoints.windows(2) {
        let x0 = tx(w[0].x);
        let z0 = tz(w[0].z);
        let x1 = tx(w[1].x);
        let z1 = tz(w[1].z);
        let steps = ((x1 - x0).hypot(z1 - z0) / 6.0).ceil().max(1.0) as i32;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            ui.rect(x0 + (x1 - x0) * t, z0 + (z1 - z0) * t, 7.0 * ui.s, 7.0 * ui.s, [0.76, 0.6, 0.38, 0.95]);
        }
    }
    // épingles : cages restantes + coffres + portail
    for it in &g.interactables {
        if it.done {
            continue;
        }
        let (icon, c) = match it.kind {
            crate::game::InteractKind::Cage(_) => (IC_PIN, [1.0, 0.8, 0.3, 1.0]),
            crate::game::InteractKind::Chest(_) | crate::game::InteractKind::CryptChest => (IC_CHEST, [1.0, 0.9, 0.5, 1.0]),
            crate::game::InteractKind::Portal => (IC_SKULL, [0.8, 0.5, 1.0, 1.0]),
            crate::game::InteractKind::Rune => (IC_SCROLL, [0.7, 0.5, 1.0, 1.0]),
            _ => (IC_PIN, [0.8, 0.8, 0.8, 1.0]),
        };
        ui.icon(icon, tx(it.pos.x) - 10.0 * ui.s, tz(it.pos.z) - 10.0 * ui.s, 20.0 * ui.s, c);
    }
    // joueur (flèche = tête du héros)
    ui.icon(IC_HEAD, tx(g.player.pos.x) - 12.0 * ui.s, tz(g.player.pos.z) - 12.0 * ui.s, 24.0 * ui.s, WHITE);
    // boussole N
    ui.icon(IC_COMPASS, ui.w - 120.0 * ui.s, 30.0 * ui.s, 44.0 * ui.s, WHITE);
    ui.text(ui.w - 98.0 * ui.s, 78.0 * ui.s, 20.0 * ui.s, "N", WHITE, false);
    // objectif courant
    ui.text(30.0 * ui.s, ui.h - 46.0 * ui.s, 24.0 * ui.s, &format!("Objectif : {}", g.quest.label()), GOLD, false);
    // zones nommées
    for z in &g.level.zones {
        ui.text(tx(z.center.x), tz(z.center.z), 14.0 * ui.s, z.name, [0.85, 0.85, 0.8, 0.85], true);
    }
    ui.text(ui.w * 0.5, ui.h - 26.0 * ui.s, 18.0 * ui.s, "M / Échap — Fermer", [0.75, 0.75, 0.75, 0.9], true);
}

// ==================== INVENTAIRE ====================

fn draw_inventory(ui: &mut Ui, g: &Game) {
    dim(ui, 0.6);
    let pw = 760.0 * ui.s;
    let ph = 520.0 * ui.s;
    let px = (ui.w - pw) * 0.5;
    let py = (ui.h - ph) * 0.5;
    ui.rect(px, py, pw, ph, PANEL);
    ui.rect(px, py, pw, 4.0 * ui.s, PANEL_BORDER);
    ui.rect(px, py + ph - 4.0 * ui.s, pw, 4.0 * ui.s, PANEL_BORDER);
    ui.text(px + pw * 0.5, py + 24.0 * ui.s, 38.0 * ui.s, "INVENTAIRE", GOLD, true);

    // équipement
    let eq = [
        (IC_SWORD, "Mêlée : Épée en fer"),
        (IC_BOW, "Distance : Arc court"),
        (IC_ARMOR, "Armure : Cotte de mailles"),
    ];
    let mut y = py + 96.0 * ui.s;
    for (icon, label) in eq {
        ui.icon(icon, px + 60.0 * ui.s, y, 44.0 * ui.s, WHITE);
        ui.text(px + 120.0 * ui.s, y + 8.0 * ui.s, 24.0 * ui.s, label, WHITE, false);
        y += 62.0 * ui.s;
    }
    // artefacts
    ui.text(px + 60.0 * ui.s, y + 6.0 * ui.s, 24.0 * ui.s, "Artefacts :", GOLD, false);
    y += 44.0 * ui.s;
    let arts = [(IC_FIREWORK, "Flèche d'artifice (1)"), (IC_HORN, "Cor de vent — 60 âmes (2)"), (IC_TOTEM, "Totem de régénération (3)")];
    for (icon, label) in arts {
        ui.icon(icon, px + 60.0 * ui.s, y, 40.0 * ui.s, WHITE);
        ui.text(px + 116.0 * ui.s, y + 6.0 * ui.s, 21.0 * ui.s, label, WHITE, false);
        y += 54.0 * ui.s;
    }
    // stats (droite)
    let p = &g.player;
    let stats = [
        format!("PV : {}/100", p.hp as i32),
        format!("Âmes : {}/{}", p.souls, MAX_SOULS),
        format!("Émeraudes : {}", p.emeralds),
        format!("Flèches : {}", p.arrows),
        format!("Niveau : {} (+{} XP)", p.level, p.xp),
        format!("Points d'enchantement : {}", p.enchant_points),
    ];
    let mut sy = py + 96.0 * ui.s;
    for st in stats {
        ui.text(px + pw - 300.0 * ui.s, sy, 22.0 * ui.s, &st, GREEN_TXT, false);
        sy += 44.0 * ui.s;
    }
    ui.text(px + pw * 0.5, py + ph - 40.0 * ui.s, 18.0 * ui.s, "I / Échap — Fermer", [0.75, 0.75, 0.75, 0.9], true);
}
