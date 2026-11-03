//! Héros : déplacement, roulade, combo de mêlée (mémorisé), potion, artefacts,
//! construction du modèle cuboid animé.

use crate::input::Input;
use crate::level::Level;
use crate::vertex::{push_box, DynVertex};
use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::level::{SLOT_GOLD, SLOT_PLANK, SLOT_STONE, SLOT_TRUNK};
use crate::mobs::SLOT_FACE;
use winit::keyboard::KeyCode;

pub struct Player {
    pub pos: Vec3,
    pub yaw: f32,
    pub hp: f32,
    pub lives: u32,
    pub souls: u32,
    pub emeralds: u32,
    pub arrows: u32,
    pub xp: u32,
    pub level: u32,
    pub enchant_points: u32,
    pub combo_stage: u32,
    pub combo_timer: f32,
    pub attack_anim: f32,
    pub attack_cd: f32,
    pub roll_timer: f32,
    pub roll_cd: f32,
    pub iframes: f32,
    pub potion_cd: f32,
    pub artifact_cd: [f32; 3],
    pub walk_phase: f32,
    pub hurt_flash: f32,
    pub totem_pos: Option<Vec3>,
    pub totem_timer: f32,
    pub dead: bool,
    pub dead_timer: f32,
}

pub const MAX_HP: f32 = 100.0;
pub const MAX_SOULS: u32 = 300;
pub const POTION_HEAL: f32 = 75.0;
pub const HORN_SOUL_COST: u32 = 60;

impl Player {
    pub fn new(spawn: Vec3) -> Player {
        Player {
            pos: spawn,
            yaw: 0.0,
            hp: MAX_HP,
            lives: 3,
            souls: 0,
            emeralds: 0,
            arrows: 20,
            xp: 0,
            level: 1,
            enchant_points: 0,
            combo_stage: 0,
            combo_timer: 0.0,
            attack_anim: 0.0,
            attack_cd: 0.0,
            roll_timer: 0.0,
            roll_cd: 0.0,
            iframes: 0.0,
            potion_cd: 0.0,
            artifact_cd: [0.0; 3],
            walk_phase: 0.0,
            hurt_flash: 0.0,
            totem_pos: None,
            totem_timer: 0.0,
            dead: false,
            dead_timer: 0.0,
        }
    }

    /// Tick de mouvement/réactions commun (les attaques sont résolues côté jeu).
    pub fn tick(&mut self, input: &Input, level: &Level, dt: f32) {
        let axis = crate::input::move_axis(input);
        let rolling = self.roll_timer > 0.0;

        if !self.dead && !rolling {
            if axis != Vec2::ZERO {
                let dir = Vec3::new(axis.x, 0.0, axis.y).normalize_or_zero();
                self.yaw = dir.z.atan2(dir.x);
                let vel = dir * 4.6;
                level.slide(&mut self.pos, vel * dt, 0.38);
                self.walk_phase += dt * 10.0;
            } else {
                self.walk_phase += dt * 2.0;
            }
        } else if rolling {
            let dir = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
            level.slide(&mut self.pos, dir * 9.0 * dt, 0.38);
            self.walk_phase += dt * 16.0;
        }

        // Roulade (Espace)
        if input.just_pressed(KeyCode::Space) && !self.dead && self.roll_timer <= 0.0 && self.roll_cd <= 0.0 {
            self.roll_timer = 0.45;
            self.roll_cd = 1.15;
            self.iframes = self.iframes.max(0.5);
        }

        for t in [
            &mut self.attack_cd,
            &mut self.roll_cd,
            &mut self.potion_cd,
            &mut self.iframes,
            &mut self.combo_timer,
            &mut self.attack_anim,
        ] {
            *t = (*t - dt).max(0.0);
        }
        for c in &mut self.artifact_cd {
            *c = (*c - dt).max(0.0);
        }
        self.roll_timer = (self.roll_timer - dt).max(0.0);
        self.hurt_flash = (self.hurt_flash - dt * 3.0).max(0.0);
        if self.combo_timer <= 0.0 && self.combo_stage > 0 && self.attack_cd <= 0.0 {
            // le combo se mémorise entre les pauses mais expire au combat
            self.combo_stage = 0;
        }
        // Totem de régénération
        if let Some(tp) = self.totem_pos {
            self.totem_timer -= dt;
            if self.pos.distance(tp) < 4.5 && self.hp < MAX_HP {
                self.hp = (self.hp + 5.0 * dt).min(MAX_HP);
            }
            if self.totem_timer <= 0.0 {
                self.totem_pos = None;
            }
        }
    }

    pub fn wants_potion(&self, input: &Input) -> bool {
        input.just_pressed(KeyCode::KeyE) && !self.dead && self.potion_cd <= 0.0 && self.hp < MAX_HP
    }

    pub fn drink_potion(&mut self) {
        self.hp = (self.hp + POTION_HEAL).min(MAX_HP);
        self.potion_cd = 30.0;
    }

    pub fn artifact_ready(&self, idx: usize) -> bool {
        self.artifact_cd[idx] <= 0.0 && !self.dead
    }

    pub fn use_artifact(&mut self, idx: usize, cd: f32) {
        self.artifact_cd[idx] = cd;
    }

    pub fn add_xp(&mut self, amount: u32) {
        self.xp += amount;
        // 1 point d'enchantement par niveau (courbe douce)
        let need = self.level * 30;
        if self.xp >= need {
            self.xp -= need;
            self.level += 1;
            self.enchant_points += 1;
        }
    }

    pub fn add_souls(&mut self, amount: u32) {
        self.souls = (self.souls + amount).min(MAX_SOULS);
    }

    /// Dégâts subis (i-frames et roulade gérés ici). Retourne true si touché.
    pub fn take_damage(&mut self, dmg: f32) -> bool {
        if self.dead || self.iframes > 0.0 || self.roll_timer > 0.0 {
            return false;
        }
        self.hp -= dmg;
        self.iframes = 0.7;
        self.hurt_flash = 1.0;
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.dead = true;
            self.dead_timer = 1.6;
        }
        true
    }
}

/// Modèle du héros (tunique teal, épée, animation de marche/attaque/roulade).
pub fn build_mesh(p: &Player, out: &mut Vec<DynVertex>) {
    let root_pitch = if p.roll_timer > 0.0 {
        let t = 1.0 - p.roll_timer / 0.45;
        Mat4::from_translation(p.pos + Vec3::new(0.0, 0.55, 0.0))
            * Mat4::from_rotation_y(-p.yaw)
            * Mat4::from_rotation_x(t * std::f32::consts::TAU)
            * Mat4::from_translation(Vec3::new(0.0, -0.55, 0.0))
            * Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))
    } else {
        Mat4::from_translation(p.pos) * Mat4::from_rotation_y(-p.yaw)
    };
    let root = root_pitch;
    let flash = p.hurt_flash;
    let tint = |c: [f32; 3]| -> Vec4 {
        Vec4::new((c[0] + flash * 1.6).min(2.2), c[1], (c[2] + flash * 1.6).min(2.2), 1.0)
    };
    let swing = if p.roll_timer > 0.0 { 0.0 } else { p.walk_phase.sin() * 0.65 };

    // jambes (pantalon brun, bottes grises)
    for (i, sx) in [-0.13f32, 0.13].iter().enumerate() {
        let sw = if i == 0 { swing } else { -swing };
        let leg = root * Mat4::from_translation(Vec3::new(*sx, 0.72, 0.0)) * Mat4::from_rotation_x(sw);
        push_box(out, leg * Mat4::from_translation(Vec3::new(0.0, -0.28, 0.0)), Vec3::new(0.12, 0.28, 0.12), [SLOT_PLANK; 6], tint([0.42, 0.29, 0.18]), [0.0, 1.0, 0.0, 0.0]);
        push_box(out, leg * Mat4::from_translation(Vec3::new(0.0, -0.62, 0.02)), Vec3::new(0.13, 0.1, 0.15), [SLOT_STONE; 6], tint([0.35, 0.35, 0.38]), [0.0, 1.0, 0.0, 0.0]);
    }
    // torse (tunique teal) + ceinture
    push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.08, 0.0)), Vec3::new(0.26, 0.37, 0.15), [SLOT_PLANK; 6], tint([0.23, 0.64, 0.64]), [0.0, 1.0, 0.0, 0.0]);
    push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 0.76, 0.0)), Vec3::new(0.27, 0.06, 0.16), [SLOT_GOLD; 6], tint([0.55, 0.42, 0.2]), [0.0, 1.0, 0.0, 0.0]);
    // bras
    let arm_l = root * Mat4::from_translation(Vec3::new(-0.36, 1.38, 0.0)) * Mat4::from_rotation_x(-swing * 0.7);
    push_box(out, arm_l * Mat4::from_translation(Vec3::new(0.0, -0.3, 0.0)), Vec3::new(0.11, 0.3, 0.11), [SLOT_PLANK; 6], tint([0.2, 0.55, 0.55]), [0.0, 1.0, 0.0, 0.0]);
    push_box(out, arm_l * Mat4::from_translation(Vec3::new(0.0, -0.66, 0.0)), Vec3::splat(0.09), [SLOT_FACE; 6], tint([0.78, 0.6, 0.44]), [0.0, 1.0, 0.0, 0.0]);
    let arm_r = root * Mat4::from_translation(Vec3::new(0.36, 1.38, 0.0)) * Mat4::from_rotation_x(swing * 0.7 - p.attack_anim * 2.1);
    push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.3, 0.0)), Vec3::new(0.11, 0.3, 0.11), [SLOT_PLANK; 6], tint([0.2, 0.55, 0.55]), [0.0, 1.0, 0.0, 0.0]);
    push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.66, 0.0)), Vec3::splat(0.09), [SLOT_FACE; 6], tint([0.78, 0.6, 0.44]), [0.0, 1.0, 0.0, 0.0]);
    // épée en main droite (lame + garde)
    push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.78, 0.06)), Vec3::new(0.04, 0.22, 0.04), [SLOT_TRUNK; 6], tint([0.4, 0.28, 0.16]), [0.0, 1.0, 0.0, 0.0]);
    push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -1.04, 0.06)), Vec3::new(0.12, 0.03, 0.05), [SLOT_GOLD; 6], tint([0.55, 0.42, 0.2]), [0.0, 1.0, 0.0, 0.0]);
    push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -1.28, 0.06)), Vec3::new(0.045, 0.24, 0.02), [SLOT_GOLD; 6], tint([0.85, 0.88, 0.95]), [0.0, 1.0, 0.0, 0.0]);
    // tête (peau + cheveux)
    let head = root * Mat4::from_translation(Vec3::new(0.0, 1.66, 0.0));
    push_box(out, head, Vec3::splat(0.25), [SLOT_FACE; 6], tint([0.78, 0.6, 0.44]), [0.0, 1.0, 0.0, 0.0]);
    push_box(out, head * Mat4::from_translation(Vec3::new(0.0, 0.12, 0.0)), Vec3::new(0.26, 0.13, 0.26), [SLOT_TRUNK; 6], tint([0.29, 0.2, 0.12]), [0.0, 1.0, 0.0, 0.0]);
}
