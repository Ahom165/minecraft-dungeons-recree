//! Mobs : statistiques, IA et construction des modèles cuboides animés
//! (style Minecraft : zombie, squelette, creeper, nécromancien, villageois).

use crate::level::{Level, SLOT_GOLD, SLOT_PLANK, SLOT_PORTAL, SLOT_STONE, SLOT_TRUNK};
use crate::vertex::{push_box, DynVertex};
use glam::{Mat4, Vec3, Vec4};

pub const SLOT_FACE: usize = 38;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MobKind {
    Zombie,
    Skeleton,
    Creeper,
    Necromancer,
    Villager,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MobState {
    Idle,
    Wander,
    Chase,
    Windup,
    Cooldown,
    Summon,
    Cast,
}

pub struct Mob {
    pub kind: MobKind,
    pub pos: Vec3,
    pub yaw: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub state: MobState,
    pub timer: f32,
    pub wander_target: Vec3,
    pub home: Vec3,
    pub enchanted: bool,
    pub walk_phase: f32,
    pub attack_anim: f32,
    pub fuse: f32,
    pub summoned: bool,
    pub dead: bool,
    pub fade: f32,
    pub hurt_flash: f32,
}

impl Mob {
    pub fn new(kind: MobKind, pos: Vec3, enchanted: bool) -> Mob {
        let (hp, _max_hp) = match kind {
            MobKind::Zombie => (20.0, 20.0),
            MobKind::Skeleton => (15.0, 15.0),
            MobKind::Creeper => (12.0, 12.0),
            MobKind::Necromancer => (600.0, 600.0),
            MobKind::Villager => (10.0, 10.0),
        };
        let hp = if enchanted { hp * 2.0 } else { hp };
        Mob {
            kind,
            pos,
            yaw: 0.0,
            hp,
            max_hp: hp,
            state: MobState::Idle,
            timer: 0.0,
            wander_target: pos,
            home: pos,
            enchanted,
            walk_phase: 0.0,
            attack_anim: 0.0,
            fuse: 0.0,
            summoned: false,
            dead: false,
            fade: 1.0,
            hurt_flash: 0.0,
        }
    }

    pub fn radius(&self) -> f32 {
        match self.kind {
            MobKind::Necromancer => 0.6,
            _ => 0.42,
        }
    }

    pub fn speed(&self) -> f32 {
        match self.kind {
            MobKind::Zombie => 2.7,
            MobKind::Skeleton => 2.3,
            MobKind::Creeper => 2.4,
            MobKind::Necromancer => 2.9,
            MobKind::Villager => 0.8,
        }
    }
}

/// Événements produits par l'IA des mobs, consommés par le jeu.
pub enum MobEvent {
    ShootArrow { pos: Vec3, dir: Vec3 },
    Explode { pos: Vec3 },
    HitPlayer { dmg: f32 },
    CastOrbs { pos: Vec3, base_yaw: f32 },
    SummonZombie { pos: Vec3 },
}

pub struct AiCtx<'a> {
    pub player_pos: Vec3,
    pub player_alive: bool,
    pub level: &'a Level,
    pub dt: f32,
    pub rng: &'a mut crate::rng::Rng,
    pub events: &'a mut Vec<MobEvent>,
}

pub fn tick(mob: &mut Mob, ctx: &mut AiCtx) {
    if mob.dead {
        mob.fade -= ctx.dt * 1.4;
        return;
    }
    mob.hurt_flash = (mob.hurt_flash - ctx.dt * 4.0).max(0.0);
    mob.attack_anim = (mob.attack_anim - ctx.dt * 3.0).max(0.0);
    let to_player = ctx.player_pos - mob.pos;
    let dist = to_player.length();
    let dir_to_player = to_player / dist.max(0.001);

    let mut desired = Vec3::ZERO;
    match mob.kind {
        MobKind::Villager => {
            // Petit errage nerveux
            mob.timer -= ctx.dt;
            if mob.timer <= 0.0 {
                mob.timer = ctx.rng.range(2.0, 5.0);
                let a = ctx.rng.range(0.0, std::f32::consts::TAU);
                mob.wander_target = mob.home + Vec3::new(a.cos() * 1.5, 0.0, a.sin() * 1.5);
            }
            let d = mob.wander_target - mob.pos;
            if d.length() > 0.3 {
                desired = d.normalize_or_zero() * mob.speed() * 0.5;
            }
        }
        MobKind::Zombie => {
            if dist < 16.0 && ctx.player_alive {
                mob.state = MobState::Chase;
            } else if mob.state == MobState::Chase {
                mob.state = MobState::Wander;
            }
            match mob.state {
                MobState::Chase => {
                    mob.yaw = dir_to_player.y.atan2(dir_to_player.x);
                    if dist > 1.6 {
                        desired = dir_to_player * mob.speed();
                    } else {
                        mob.state = MobState::Windup;
                        mob.timer = 0.45;
                    }
                }
                MobState::Windup => {
                    mob.timer -= ctx.dt;
                    mob.attack_anim = 1.0;
                    if mob.timer <= 0.0 {
                        if dist < 2.2 && ctx.player_alive {
                            ctx.events.push(MobEvent::HitPlayer { dmg: 8.0 });
                        }
                        mob.state = MobState::Cooldown;
                        mob.timer = 1.0;
                    }
                }
                _ => {
                    mob.timer -= ctx.dt;
                    if mob.timer <= 0.0 {
                        mob.state = MobState::Wander;
                        mob.timer = ctx.rng.range(2.0, 5.0);
                        let a = ctx.rng.range(0.0, std::f32::consts::TAU);
                        mob.wander_target = mob.home + Vec3::new(a.cos() * 4.0, 0.0, a.sin() * 4.0);
                    }
                    let d = mob.wander_target - mob.pos;
                    if d.length() > 0.4 {
                        desired = d.normalize_or_zero() * mob.speed() * 0.4;
                        mob.yaw = desired.z.atan2(desired.x);
                    }
                }
            }
        }
        MobKind::Skeleton => {
            if dist < 18.0 && ctx.player_alive {
                mob.yaw = dir_to_player.y.atan2(dir_to_player.x);
                if dist < 7.0 {
                    desired = -dir_to_player * mob.speed();
                } else if dist > 11.0 {
                    desired = dir_to_player * mob.speed();
                } else {
                    // strafe
                    let side = Vec3::new(-dir_to_player.z, 0.0, dir_to_player.x);
                    desired = side * mob.speed() * 0.6;
                }
                mob.timer -= ctx.dt;
                if mob.timer <= 0.0 {
                    mob.timer = 2.4;
                    mob.attack_anim = 1.0;
                    ctx.events.push(MobEvent::ShootArrow {
                        pos: mob.pos + Vec3::new(0.0, 1.4, 0.0),
                        dir: dir_to_player,
                    });
                }
            } else {
                mob.timer = 1.2;
            }
        }
        MobKind::Creeper => {
            if dist < 2.5 && ctx.player_alive {
                mob.fuse += ctx.dt;
                mob.yaw = dir_to_player.y.atan2(dir_to_player.x);
                if mob.fuse >= 1.2 {
                    mob.dead = true;
                    mob.fade = 0.0;
                    ctx.events.push(MobEvent::Explode { pos: mob.pos });
                }
            } else {
                mob.fuse = (mob.fuse - ctx.dt * 2.0).max(0.0);
                if dist < 14.0 && ctx.player_alive {
                    mob.yaw = dir_to_player.y.atan2(dir_to_player.x);
                    desired = dir_to_player * mob.speed();
                }
            }
        }
        MobKind::Necromancer => match mob.state {
            MobState::Summon => {
                mob.timer -= ctx.dt;
                mob.attack_anim = 0.6 + 0.4 * (mob.timer * 6.0).sin().abs();
                if !mob.summoned && mob.timer <= 2.2 {
                    mob.summoned = true;
                    let circle = 7;
                    for i in 0..circle {
                        let a = i as f32 / circle as f32 * std::f32::consts::TAU;
                        ctx.events.push(MobEvent::SummonZombie {
                            pos: mob.pos + Vec3::new(a.cos() * 3.2, 0.0, a.sin() * 3.2),
                        });
                    }
                }
                if mob.timer <= 0.0 {
                    mob.state = MobState::Cast;
                    mob.timer = 1.5;
                }
            }
            MobState::Cast => {
                mob.timer -= ctx.dt;
                mob.yaw = dir_to_player.y.atan2(dir_to_player.x);
                // garde ses distances
                if dist < 9.0 {
                    desired = -dir_to_player * mob.speed();
                }
                if mob.timer <= 0.0 {
                    mob.attack_anim = 1.0;
                    ctx.events.push(MobEvent::CastOrbs { pos: mob.pos, base_yaw: mob.yaw });
                    mob.timer = 3.5;
                }
                if dist < 3.5 {
                    // fuite téléportée
                    mob.state = MobState::Summon;
                    mob.timer = 3.0;
                    let a = ctx.rng.range(0.0, std::f32::consts::TAU);
                    let mut np = mob.pos + Vec3::new(a.cos() * 12.0, 0.0, a.sin() * 12.0);
                    ctx.level.slide(&mut np, Vec3::ZERO, mob.radius());
                    mob.pos = np;
                }
            }
            _ => {
                mob.state = MobState::Summon;
                mob.timer = 3.0;
            }
        },
    }

    // Déplacement + collision + animation de marche
    if desired != Vec3::ZERO {
        let mut vel = desired;
        let mut probe = mob.pos;
        ctx.level.slide(&mut probe, vel * ctx.dt, mob.radius());
        if (probe - mob.pos).length() < vel.length() * ctx.dt * 0.3 {
            // bloqué : glisse perpendiculairement
            let side = Vec3::new(-vel.z, 0.0, vel.x).normalize_or_zero();
            vel = side * mob.speed() * 0.8;
        }
        let r = mob.radius();
        ctx.level.slide(&mut mob.pos, vel * ctx.dt, r);
        mob.walk_phase += ctx.dt * 9.0;
    } else {
        mob.walk_phase += ctx.dt * 1.5;
    }
}

// =====================================================================
// Construction des modèles (cuboids animés)
// =====================================================================

struct LimbStyle {
    half: Vec3,
    slot: usize,
}

fn biped_limbs(kind: MobKind) -> (LimbStyle, LimbStyle, LimbStyle) {
    // (bras, jambes, torse)
    match kind {
        MobKind::Skeleton => (
            LimbStyle { half: Vec3::new(0.08, 0.36, 0.08), slot: SLOT_STONE },
            LimbStyle { half: Vec3::new(0.08, 0.36, 0.08), slot: SLOT_STONE },
            LimbStyle { half: Vec3::new(0.22, 0.37, 0.13), slot: SLOT_STONE },
        ),
        MobKind::Villager => (
            LimbStyle { half: Vec3::new(0.10, 0.36, 0.10), slot: SLOT_TRUNK },
            LimbStyle { half: Vec3::new(0.12, 0.36, 0.12), slot: SLOT_PLANK },
            LimbStyle { half: Vec3::new(0.28, 0.40, 0.16), slot: SLOT_PLANK },
        ),
        _ => (
            LimbStyle { half: Vec3::new(0.11, 0.36, 0.11), slot: SLOT_TRUNK },
            LimbStyle { half: Vec3::new(0.12, 0.36, 0.12), slot: SLOT_PLANK },
            LimbStyle { half: Vec3::new(0.26, 0.37, 0.15), slot: SLOT_PLANK },
        ),
    }
}

/// Construit le mesh animé d'un mob dans `out` (monde dynamique).
pub fn build_mesh(mob: &Mob, out: &mut Vec<DynVertex>) {
    if mob.dead && mob.fade <= 0.0 {
        return;
    }
    let alpha = mob.fade.min(1.0);
    let hurt = mob.hurt_flash;
    let base_tint = |c: [f32; 3]| -> Vec4 {
        Vec4::new(
            (c[0] + hurt * 1.5).min(2.0),
            (c[1] + hurt * 0.3).min(2.0),
            (c[2] + hurt * 1.5).min(2.0),
            alpha,
        )
    };
    let root = Mat4::from_translation(mob.pos) * Mat4::from_rotation_y(-mob.yaw);
    let swing = mob.walk_phase.sin() * 0.65;

    match mob.kind {
        MobKind::Creeper => {
            let swell = 1.0 + mob.fuse * 0.22 + (mob.fuse * 14.0).sin() * 0.03 * mob.fuse;
            let green = if mob.fuse > 0.0 {
                let f = mob.fuse / 1.2;
                [0.31 + f * 0.9, 0.67, 0.31, ]
            } else {
                [0.31, 0.67, 0.31]
            };
            let tint = base_tint(green);
            let tint_dark = base_tint([green[0] * 0.55, green[1] * 0.55, green[2] * 0.55]);
            // 4 pattes
            for (lx, lz) in [(-0.16f32, -0.22f32), (0.16, -0.22), (-0.16, 0.22), (0.16, 0.22)] {
                push_box(out, root * Mat4::from_translation(Vec3::new(lx, 0.16, lz)), Vec3::new(0.12, 0.16, 0.12), [SLOT_TRUNK; 6], tint_dark, [0.0, 1.0, 0.0, 0.0]);
            }
            // corps
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 0.72, 0.0)) * Mat4::from_scale(Vec3::splat(swell)), Vec3::new(0.26, 0.42, 0.22), [SLOT_TRUNK; 6], tint, [0.0, 1.0, 0.0, 0.0]);
            // tête + face iconique (yeux + bouche noirs en relief)
            let head_m = root * Mat4::from_translation(Vec3::new(0.0, 1.42, 0.0)) * Mat4::from_scale(Vec3::splat(swell));
            push_box(out, head_m, Vec3::splat(0.25), [SLOT_TRUNK; 6], tint, [0.0, 1.0, 0.0, 0.0]);
            let face_tint = base_tint([0.02, 0.02, 0.02]);
            for ex in [-0.13f32, 0.13] {
                push_box(out, head_m * Mat4::from_translation(Vec3::new(ex, 0.07, 0.26)), Vec3::new(0.055, 0.07, 0.012), [SLOT_TRUNK; 6], face_tint, [0.0, 1.0, 0.0, 0.0]);
            }
            push_box(out, head_m * Mat4::from_translation(Vec3::new(0.0, -0.03, 0.26)), Vec3::new(0.06, 0.05, 0.012), [SLOT_TRUNK; 6], face_tint, [0.0, 1.0, 0.0, 0.0]);
            push_box(out, head_m * Mat4::from_translation(Vec3::new(-0.06, -0.1, 0.26)), Vec3::new(0.035, 0.09, 0.012), [SLOT_TRUNK; 6], face_tint, [0.0, 1.0, 0.0, 0.0]);
            push_box(out, head_m * Mat4::from_translation(Vec3::new(0.06, -0.1, 0.26)), Vec3::new(0.035, 0.09, 0.012), [SLOT_TRUNK; 6], face_tint, [0.0, 1.0, 0.0, 0.0]);
        }
        MobKind::Necromancer => {
            let robe = base_tint([0.23, 0.16, 0.35]);
            let robe_dark = base_tint([0.14, 0.09, 0.22]);
            let skin = base_tint([0.78, 0.62, 0.47]);
            // jupe (robe évasée)
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 0.42, 0.0)), Vec3::new(0.34, 0.42, 0.24), [SLOT_PLANK; 6], robe_dark, [0.0, 1.0, 0.0, 0.0]);
            // torse
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.08, 0.0)), Vec3::new(0.27, 0.34, 0.18), [SLOT_PLANK; 6], robe, [0.0, 1.0, 0.0, 0.0]);
            // bras levé (incantation) + bras
            let arm_l = root * Mat4::from_translation(Vec3::new(-0.36, 1.28, 0.0)) * Mat4::from_rotation_x(-2.2 + swing * 0.1);
            push_box(out, arm_l * Mat4::from_translation(Vec3::new(0.0, -0.25, 0.0)), Vec3::new(0.08, 0.3, 0.08), [SLOT_PLANK; 6], robe_dark, [0.0, 1.0, 0.0, 0.0]);
            let arm_r = root * Mat4::from_translation(Vec3::new(0.36, 1.28, 0.0)) * Mat4::from_rotation_x(-1.2 - mob.attack_anim * 1.2);
            push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.25, 0.0)), Vec3::new(0.08, 0.3, 0.08), [SLOT_PLANK; 6], robe_dark, [0.0, 1.0, 0.0, 0.0]);
            // bâton doré + orbe violette émissive
            push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.45, 0.12)), Vec3::new(0.035, 0.5, 0.035), [SLOT_GOLD; 6], base_tint([0.9, 0.7, 0.25]), [0.0, 1.0, 0.0, 0.0]);
            push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -1.05, 0.12)), Vec3::splat(0.09), [SLOT_PORTAL; 6], Vec4::new(1.7, 1.2, 2.2, alpha), [1.0, 1.0, 0.0, 0.0]);
            // tête pâle + capuche
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.62, 0.0)), Vec3::splat(0.24), [SLOT_FACE; 6], skin, [0.0, 1.0, 0.0, 0.0]);
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.66, -0.02)), Vec3::new(0.3, 0.28, 0.26), [SLOT_PLANK; 6], robe_dark, [0.0, 0.92, 0.0, 0.0]);
        }
        MobKind::Villager => {
            let tint_body = base_tint([0.48, 0.36, 0.23]);
            let tint_skin = base_tint([0.78, 0.6, 0.44]);
            // robe couvrant les jambes
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 0.55, 0.0)), Vec3::new(0.28, 0.55, 0.18), [SLOT_PLANK; 6], tint_body, [0.0, 1.0, 0.0, 0.0]);
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.12, 0.0)), Vec3::new(0.24, 0.22, 0.15), [SLOT_PLANK; 6], tint_body, [0.0, 1.0, 0.0, 0.0]);
            // tête + gros nez
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0)), Vec3::splat(0.24), [SLOT_FACE; 6], tint_skin, [0.0, 1.0, 0.0, 0.0]);
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.46, 0.26)), Vec3::new(0.06, 0.12, 0.07), [SLOT_FACE; 6], tint_skin, [0.0, 1.0, 0.0, 0.0]);
        }
        _ => {
            // Zombie / Squelette : bipède standard
            let (arms, legs, body) = biped_limbs(mob.kind);
            let swing = mob.walk_phase.sin() * 0.65;
            let swing_opp = -swing;
            let (c_limb, c_legs, c_body, c_head): ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) = match mob.kind {
                MobKind::Zombie => ([0.24, 0.48, 0.24], [0.16, 0.28, 0.42], [0.16, 0.54, 0.54], [0.24, 0.48, 0.24]),
                _ => ([0.81, 0.81, 0.79], [0.72, 0.72, 0.7], [0.78, 0.78, 0.76], [0.85, 0.85, 0.83]),
            };
            let leg_l = root * Mat4::from_translation(Vec3::new(-0.13, 0.72, 0.0)) * Mat4::from_rotation_x(swing);
            push_box(out, leg_l * Mat4::from_translation(Vec3::new(0.0, -0.36, 0.0)), legs.half, [legs.slot; 6], base_tint(c_legs), [0.0, 1.0, 0.0, 0.0]);
            let leg_r = root * Mat4::from_translation(Vec3::new(0.13, 0.72, 0.0)) * Mat4::from_rotation_x(swing_opp);
            push_box(out, leg_r * Mat4::from_translation(Vec3::new(0.0, -0.36, 0.0)), legs.half, [legs.slot; 6], base_tint(c_legs), [0.0, 1.0, 0.0, 0.0]);
            push_box(out, root * Mat4::from_translation(Vec3::new(0.0, 1.08, 0.0)), body.half, [body.slot; 6], base_tint(c_body), [0.0, 1.0, 0.0, 0.0]);
            // bras (zombie : tendus devant)
            let zraise = if mob.kind == MobKind::Zombie { -1.4 } else { 0.0 };
            let arm_l = root * Mat4::from_translation(Vec3::new(-0.36, 1.38, 0.0)) * Mat4::from_rotation_x(swing_opp * 0.7 + zraise);
            push_box(out, arm_l * Mat4::from_translation(Vec3::new(0.0, -0.32, 0.0)), arms.half, [arms.slot; 6], base_tint(c_limb), [0.0, 1.0, 0.0, 0.0]);
            let arm_r = root * Mat4::from_translation(Vec3::new(0.36, 1.38, 0.0)) * Mat4::from_rotation_x(swing * 0.7 + zraise - mob.attack_anim * 1.6);
            push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.32, 0.0)), arms.half, [arms.slot; 6], base_tint(c_limb), [0.0, 1.0, 0.0, 0.0]);
            // arc du squelette
            if mob.kind == MobKind::Skeleton {
                push_box(out, arm_r * Mat4::from_translation(Vec3::new(0.0, -0.55, 0.1)), Vec3::new(0.03, 0.3, 0.03), [SLOT_PLANK; 6], base_tint([0.5, 0.36, 0.2]), [0.0, 1.0, 0.0, 0.0]);
            }
            // tête
            let head_m = root * Mat4::from_translation(Vec3::new(0.0, 1.66, 0.0));
            push_box(out, head_m, Vec3::splat(0.25), [SLOT_FACE; 6], base_tint(c_head), [0.0, 1.0, 0.0, 0.0]);
        }
    }

    // Aura enchantée : cubes violets pulsants flottants
    if mob.enchanted && !mob.dead {
        let t = mob.walk_phase * 0.7;
        for i in 0..3 {
            let a = t + i as f32 * std::f32::consts::TAU / 3.0;
            let p = mob.pos + Vec3::new(a.cos() * 0.55, 0.9 + (t * 2.0 + i as f32).sin() * 0.25, a.sin() * 0.55);
            push_box(out, Mat4::from_translation(p) * Mat4::from_rotation_y(a), Vec3::splat(0.07), [39; 6], Vec4::new(1.9, 1.2, 2.6, 0.9), [1.0, 1.0, 0.0, 0.0]);
        }
    }
}
