//! Particules : feuilles, braises, étincelles, âmes, explosions, auras.
//! Deux lots : alpha (feuilles) et additif (halos, âmes, étincelles).

use crate::rng::Rng;
use crate::vertex::{slot_rect, FxInstance};
use glam::{Vec3, Vec4};

pub const UV_DISC: [f32; 4] = slot_rect(33);
pub const UV_STAR: [f32; 4] = slot_rect(34);
pub const UV_RING: [f32; 4] = slot_rect(35);
pub const UV_LEAF: [f32; 4] = slot_rect(36);

struct Particle {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    max_life: f32,
    size: f32,
    color: Vec4,
    uv: [f32; 4],
    additive: bool,
    grav: f32,
    drag: f32,
    homing: bool,
}

pub struct Particles {
    list: Vec<Particle>,
}

impl Particles {
    pub fn new() -> Particles {
        Particles { list: Vec::with_capacity(2048) }
    }
}

impl Particle {
    fn base(pos: Vec3, vel: Vec3, life: f32, size: f32, color: Vec4, uv: [f32; 4], additive: bool) -> Particle {
        Particle {
            pos,
            vel,
            life,
            max_life: life,
            size,
            color,
            uv,
            additive,
            grav: 0.0,
            drag: 1.0,
            homing: false,
        }
    }
}

impl Particles {

    fn push(&mut self, p: Particle) {
        if self.list.len() < 4000 {
            self.list.push(p);
        }
    }

    pub fn leaf(&mut self, rng: &mut Rng, center: Vec3, radius: f32) {
        let p = Particle::base(
            center + Vec3::new(rng.range(-radius, radius), rng.range(4.0, 7.0), rng.range(-radius, radius)),
            Vec3::new(rng.range(-0.4, 0.4), rng.range(-1.1, -0.6), rng.range(-0.4, 0.4)),
            rng.range(4.0, 7.0),
            rng.range(0.12, 0.2),
            Vec4::new(0.55, 0.85, 0.35, 0.9),
            UV_LEAF,
            false,
        );
        self.push(Particle { grav: -0.15, drag: 0.99, ..p });
    }

    pub fn ember(&mut self, rng: &mut Rng, pos: Vec3) {
        let p = Particle::base(
            pos,
            Vec3::new(rng.range(-0.2, 0.2), rng.range(0.9, 1.8), rng.range(-0.2, 0.2)),
            rng.range(0.7, 1.4),
            rng.range(0.1, 0.2),
            Vec4::new(1.0, 0.62, 0.2, 0.85),
            UV_DISC,
            true,
        );
        self.push(Particle { drag: 0.98, ..p });
    }

    pub fn flame(&mut self, rng: &mut Rng, pos: Vec3) {
        let p = Particle::base(
            pos + Vec3::new(rng.range(-0.15, 0.15), 0.0, rng.range(-0.15, 0.15)),
            Vec3::new(0.0, rng.range(0.8, 1.4), 0.0),
            rng.range(0.35, 0.7),
            rng.range(0.25, 0.4),
            Vec4::new(1.0, 0.55, 0.15, 0.9),
            UV_DISC,
            true,
        );
        self.push(Particle { drag: 0.96, ..p });
    }

    pub fn hit_spark(&mut self, rng: &mut Rng, pos: Vec3, crit: bool) {
        let c = if crit { Vec4::new(1.0, 0.82, 0.29, 1.0) } else { Vec4::new(1.0, 0.95, 0.85, 1.0) };
        for _ in 0..7 {
            let a = rng.range(0.0, std::f32::consts::TAU);
            let s = rng.range(1.5, 3.4);
            let p = Particle::base(
                pos + Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(a.cos() * s, rng.range(0.5, 2.0), a.sin() * s),
                0.35,
                0.14,
                c,
                UV_STAR,
                true,
            );
            self.push(Particle { grav: -3.0, drag: 0.94, ..p });
        }
    }

    pub fn soul_wisp(&mut self, rng: &mut Rng, pos: Vec3) {
        let p = Particle::base(
            pos + Vec3::new(rng.range(-0.3, 0.3), 0.6, rng.range(-0.3, 0.3)),
            Vec3::new(rng.range(-0.5, 0.5), rng.range(0.8, 1.6), rng.range(-0.5, 0.5)),
            1.6,
            0.22,
            Vec4::new(0.35, 1.0, 0.54, 0.95),
            UV_DISC,
            true,
        );
        self.push(Particle { homing: true, drag: 0.97, ..p });
    }

    pub fn heal_green(&mut self, pos: Vec3) {
        let p = Particle::base(
            pos + Vec3::new(0.0, 0.4, 0.0),
            Vec3::new(0.0, 1.4, 0.0),
            0.9,
            0.2,
            Vec4::new(0.4, 1.0, 0.5, 0.9),
            UV_DISC,
            true,
        );
        self.push(p);
    }

    pub fn purple_puff(&mut self, rng: &mut Rng, pos: Vec3) {
        let p = Particle::base(
            pos,
            Vec3::new(rng.range(-0.8, 0.8), rng.range(0.4, 1.6), rng.range(-0.8, 0.8)),
            0.7,
            rng.range(0.16, 0.3),
            Vec4::new(0.62, 0.3, 0.95, 0.85),
            UV_DISC,
            true,
        );
        self.push(Particle { drag: 0.95, ..p });
    }

    pub fn explosion(&mut self, rng: &mut Rng, pos: Vec3) {
        // anneau
        let ring = Particle::base(pos + Vec3::new(0.0, 0.3, 0.0), Vec3::ZERO, 0.5, 3.2, Vec4::new(1.0, 0.7, 0.3, 0.9), UV_RING, true);
        self.push(ring);
        for _ in 0..26 {
            let a = rng.range(0.0, std::f32::consts::TAU);
            let b = rng.range(-0.6, 1.2);
            let s = rng.range(2.0, 6.5);
            let p = Particle::base(
                pos + Vec3::new(0.0, 0.5, 0.0),
                Vec3::new(a.cos() * s, b + 2.0, a.sin() * s),
                rng.range(0.5, 0.9),
                rng.range(0.2, 0.45),
                Vec4::new(1.0, rng.range(0.4, 0.75), 0.15, 1.0),
                UV_DISC,
                true,
            );
            self.push(Particle { grav: -5.0, drag: 0.93, ..p });
        }
    }

    pub fn firework_burst(&mut self, rng: &mut Rng, pos: Vec3) {
        let colors = [
            Vec4::new(1.0, 0.4, 0.3, 1.0),
            Vec4::new(1.0, 0.85, 0.3, 1.0),
            Vec4::new(0.4, 0.7, 1.0, 1.0),
        ];
        for i in 0..24 {
            let a = i as f32 / 24.0 * std::f32::consts::TAU;
            let b = rng.range(-0.5, 1.0);
            let s = rng.range(3.0, 5.5);
            let p = Particle::base(
                pos,
                Vec3::new(a.cos() * s, b, a.sin() * s),
                rng.range(0.5, 0.8),
                0.18,
                colors[i % 3],
                UV_STAR,
                true,
            );
            self.push(Particle { grav: -3.5, drag: 0.95, ..p });
        }
    }

    pub fn horn_wave(&mut self, pos: Vec3) {
        for i in 0..3 {
            let ring = Particle::base(pos + Vec3::new(0.0, 0.4, 0.0), Vec3::ZERO, 0.45 + i as f32 * 0.12, 1.5 + i as f32 * 1.5, Vec4::new(0.6, 0.9, 1.0, 0.8), UV_RING, true);
            self.push(ring);
        }
    }

    pub fn totem_aura(&mut self, rng: &mut Rng, pos: Vec3) {
        let a = rng.range(0.0, std::f32::consts::TAU);
        let r = rng.range(0.5, 2.0);
        let p = Particle::base(
            pos + Vec3::new(a.cos() * r, 0.2, a.sin() * r),
            Vec3::new(0.0, rng.range(0.8, 1.5), 0.0),
            0.8,
            0.16,
            Vec4::new(1.0, 0.85, 0.35, 0.85),
            UV_DISC,
            true,
        );
        self.push(p);
    }

    pub fn chest_sparkle(&mut self, rng: &mut Rng, pos: Vec3) {
        let p = Particle::base(
            pos + Vec3::new(rng.range(-0.3, 0.3), rng.range(0.2, 0.9), rng.range(-0.3, 0.3)),
            Vec3::new(0.0, rng.range(0.5, 1.0), 0.0),
            0.8,
            0.14,
            Vec4::new(1.0, 0.95, 0.6, 0.95),
            UV_STAR,
            true,
        );
        self.push(p);
    }

    pub fn arrow_trail(&mut self, pos: Vec3, friendly: bool) {
        let c = if friendly {
            Vec4::new(0.9, 0.95, 1.0, 0.5)
        } else {
            Vec4::new(0.8, 0.5, 1.0, 0.6)
        };
        let p = Particle::base(pos, Vec3::ZERO, 0.25, 0.1, c, UV_DISC, true);
        self.push(p);
    }

    pub fn teleport_poof(&mut self, rng: &mut Rng, pos: Vec3) {
        let offsets: Vec<Vec3> = (0..12)
            .map(|_| Vec3::new(rng.range(-0.4, 0.4), rng.range(0.2, 1.6), rng.range(-0.4, 0.4)))
            .collect();
        for o in offsets {
            self.purple_puff(rng, pos + o);
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec3) {
        self.list.retain_mut(|p| {
            p.life -= dt;
            if p.life <= 0.0 {
                return false;
            }
            if p.homing {
                let to = player_pos + Vec3::new(0.0, 1.0, 0.0) - p.pos;
                let d = to.length();
                if d < 0.6 {
                    return false;
                }
                p.vel = p.vel.lerp(to.normalize_or_zero() * 7.0, (3.0 * dt).min(1.0));
            }
            p.vel.y += p.grav * dt;
            p.vel *= p.drag;
            p.pos += p.vel * dt;
            true
        });
    }

    /// Remplit les deux lots (alpha, additif) pour le renderer.
    pub fn build_batches(&self, alpha: &mut Vec<FxInstance>, add: &mut Vec<FxInstance>) {
        for p in &self.list {
            let t = p.life / p.max_life;
            let fade = t.min(1.0);
            let color = Vec4::new(p.color.x, p.color.y, p.color.z, p.color.w * fade);
            let inst = FxInstance {
                center_size: [p.pos.x, p.pos.y, p.pos.z, p.size * (0.6 + 0.4 * fade)],
                color: color.to_array(),
                uv_rect: p.uv,
            };
            if p.additive {
                add.push(inst);
            } else {
                alpha.push(inst);
            }
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }
}
