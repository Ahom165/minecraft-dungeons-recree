//! Niveau statique « Creeper Woods » : terrain, arbres, bâtiments, colliders,
//! lumières fixes et ancres de zones — assemblé une seule fois.

use crate::rng::Rng;
use crate::vertex::{push_box, push_quad, DynVertex};
use glam::{Mat4, Vec3, Vec4};

pub const SLOT_GRASS: usize = 0;
pub const SLOT_DIRT: usize = 2;
pub const SLOT_PATH: usize = 3;
pub const SLOT_STONE: usize = 4;
pub const SLOT_MOSSY: usize = 5;
pub const SLOT_COBBLE: usize = 6;
pub const SLOT_GRAVEL: usize = 7;
pub const SLOT_TRUNK: usize = 8;
pub const SLOT_LEAVES: usize = 10;
pub const SLOT_LEAVES_DARK: usize = 11;
pub const SLOT_BUSH_RED: usize = 12;
pub const SLOT_PLANK: usize = 13;
pub const SLOT_ROOF_DARK: usize = 14;
pub const SLOT_ROOF_RED: usize = 15;
pub const SLOT_WATER: usize = 16;
pub const SLOT_LAVA: usize = 17;
pub const SLOT_MUSH_STEM: usize = 18;
pub const SLOT_MUSH_CAP: usize = 19;
pub const SLOT_GLOW_MOSS: usize = 20;
pub const SLOT_RUNE: usize = 21;
pub const SLOT_TOMB: usize = 22;
pub const SLOT_TEMPLE: usize = 23;
pub const SLOT_GOLD: usize = 24;
pub const SLOT_BOOKSHELF: usize = 25;
pub const SLOT_DOOR: usize = 26;
pub const SLOT_BANNER: usize = 27;
pub const SLOT_OBSIDIAN: usize = 28;
pub const SLOT_REDSTONE: usize = 29;
pub const SLOT_HAY: usize = 37;
pub const SLOT_PORTAL: usize = 39;

pub struct Zone {
    pub name: &'static str,
    pub center: Vec3,
}

pub struct Collider {
    pub x: f32,
    pub z: f32,
    pub hx: f32,
    pub hz: f32,
}

/// Positions interactives lues par le jeu.
pub struct Anchors {
    pub cages: [Vec3; 5],
    pub chests: [Vec3; 2],
    pub merchant: Vec3,
    pub rune_tomb: Vec3,
    pub crypt_entry: Vec3,
    pub crypt_chest: Vec3,
    pub boss_arena: Vec3,
    pub exit_portal: Vec3,
    pub caravan: Vec3,
    pub village: Vec3,
    pub checkpoints: Vec<Vec3>,
}

pub struct Level {
    pub static_verts: Vec<DynVertex>,
    pub colliders: Vec<Collider>,
    pub lights: Vec<(Vec3, f32, Vec3, f32)>,
    pub waypoints: Vec<Vec3>,
    pub zones: Vec<Zone>,
    pub water_rects: Vec<[f32; 4]>,
    pub lava_rects: Vec<[f32; 4]>,
    pub anchors: Anchors,
    pub bounds: (f32, f32, f32, f32),
    pub flame_positions: Vec<Vec3>,
}

/// Chemin central de la mission (polyline) — couvre les 10 zones du vrai niveau.
const PATH: [(f32, f32); 26] = [
    (0.0, 0.0),
    (18.0, 8.0),
    (42.0, 14.0),
    (66.0, 28.0),
    (82.0, 52.0),
    (95.0, 68.0),
    (108.0, 84.0),
    (100.0, 104.0),
    (84.0, 122.0),
    (72.0, 138.0),
    (66.0, 152.0),
    (52.0, 162.0),
    (40.0, 172.0),
    (28.0, 178.0),
    (12.0, 182.0),
    (-4.0, 190.0),
    (-16.0, 202.0),
    (-24.0, 214.0),
    (-34.0, 224.0),
    (-48.0, 234.0),
    (-62.0, 244.0),
    (-72.0, 254.0),
    (-78.0, 264.0),
    (-82.0, 274.0),
    (-84.0, 284.0),
    (-84.0, 292.0),
];

fn dist_to_path(x: f32, z: f32) -> f32 {
    let p = Vec3::new(x, 0.0, z);
    let mut best = f32::MAX;
    for w in PATH.windows(2) {
        let a = Vec3::new(w[0].0, 0.0, w[0].1);
        let b = Vec3::new(w[1].0, 0.0, w[1].1);
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
        let d = (p - (a + ab * t)).length();
        best = best.min(d);
    }
    best
}

fn in_water(x: f32, z: f32) -> bool {
    // Lac de départ (berge au sud : le camp reste au sec)
    if x > -30.0 && x < 25.0 && z > -26.0 && z < -4.0 {
        return true;
    }
    // Rivière (bande traversée par des ponts)
    if z > 128.0 && z < 141.0 && x > -92.0 && x < 58.0 {
        return !(x > 56.0 || x < 44.0 || (x > 16.0 && x < 30.0) || (x > -18.0 && x < -6.0));
    }
    false
}

fn in_lava(x: f32, z: f32) -> bool {
    x > -104.0 && x < -56.0 && z > 244.0 && z < 272.0
}

/// Tuile un plan liquide en quads de 2 unités (répétition du texel 16×16 par
/// bloc, comme Minecraft) au lieu d'un seul quad étiré et flou.
fn push_liquid_grid(
    v: &mut Vec<DynVertex>,
    rect: [f32; 4],
    y: f32,
    slot: usize,
    alpha: f32,
    rng: &mut Rng,
) {
    let [x0, z0, w, d] = rect;
    let cell = 2.0f32;
    let mut gz = z0;
    while gz < z0 + d {
        let dh = cell.min(z0 + d - gz);
        let mut gx = x0;
        while gx < x0 + w {
            let dw = cell.min(x0 + w - gx);
            let n = rng.noise2(gx * 0.7, gz * 0.7);
            let tint = Vec4::new(0.92 + n * 0.16, 0.92 + n * 0.16, 0.94 + n * 0.12, alpha);
            push_quad(
                v,
                [
                    Vec3::new(gx, y, gz + dh),
                    Vec3::new(gx + dw, y, gz + dh),
                    Vec3::new(gx + dw, y, gz),
                    Vec3::new(gx, y, gz),
                ],
                Vec3::Y,
                slot,
                tint,
                if slot == SLOT_LAVA { [1.0, 1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0, 0.0] },
            );
            gx += dw;
        }
        gz += dh;
    }
}

impl Level {
    pub fn build() -> Level {
        let mut v: Vec<DynVertex> = Vec::with_capacity(160_000);
        let mut colliders: Vec<Collider> = Vec::new();
        let mut lights: Vec<(Vec3, f32, Vec3, f32)> = Vec::new();
        let mut flames: Vec<Vec3> = Vec::new();
        let mut rng = Rng::new(20_260_501);

        let bounds = (-120.0f32, 150.0f32, -40.0f32, 310.0f32);

        // ================= TERRAIN =================
        let cell = 3.0;
        let mut gx = bounds.0;
        while gx < bounds.1 {
            let mut gz = bounds.2;
            while gz < bounds.3 {
                let cx = gx + cell * 0.5;
                let cz = gz + cell * 0.5;
                let d = dist_to_path(cx, cz);
                let n = rng.noise2(cx * 0.13, cz * 0.13);
                let (slot, tint) = if in_lava(cx, cz) {
                    (SLOT_COBBLE, Vec4::new(0.5, 0.42, 0.4, 1.0))
                } else if in_water(cx, cz) {
                    (SLOT_DIRT, Vec4::new(0.45, 0.4, 0.36, 1.0))
                } else if d < 3.2 {
                    (SLOT_PATH, Vec4::new(0.92 + n * 0.16, 0.86 + n * 0.14, 0.8 + n * 0.12, 1.0))
                } else if d < 5.5 {
                    (SLOT_GRAVEL, Vec4::new(0.9 + n * 0.2, 0.9 + n * 0.2, 0.9 + n * 0.2, 1.0))
                } else {
                    let g = 0.75 + n * 0.5;
                    (SLOT_GRASS, Vec4::new(g, g, g, 1.0))
                };
                let y = if in_lava(cx, cz) {
                    -0.3
                } else if in_water(cx, cz) {
                    -0.5
                } else {
                    0.0
                };
                push_box(
                    &mut v,
                    Mat4::from_translation(Vec3::new(cx, y - 0.25, cz)),
                    Vec3::new(cell * 0.5, 0.25, cell * 0.5),
                    [slot; 6],
                    tint,
                    [0.0, 1.0, 0.0, 0.0],
                );
                gz += cell;
            }
            gx += cell;
        }

        // ---- TEST : terrain seul + petit ----
        if std::env::var("MCD_SMALL_TERRAIN").is_ok() {
            let (minx_t, maxx_t, minz_t, maxz_t) = (-8.0, 8.0, -8.0, 8.0);
            let cell_t = 2.0;
            let mut gt = minx_t;
            while gt < maxx_t {
                let mut gz = minz_t;
                while gz < maxz_t {
                    push_quad(
                        &mut v,
                        [
                            Vec3::new(gt, 0.0, gz + cell_t),
                            Vec3::new(gt + cell_t, 0.0, gz + cell_t),
                            Vec3::new(gt + cell_t, 0.0, gz),
                            Vec3::new(gt, 0.0, gz),
                        ],
                        Vec3::Y,
                        SLOT_GRASS,
                        Vec4::ONE,
                        [0.0, 1.0, 0.0, 0.0],
                    );
                    gz += cell_t;
                }
                gt += cell_t;
            }
            let waypoints_t = vec![Vec3::ZERO];
            let zones_t = vec![Zone { name: "Test", center: Vec3::ZERO }];
            return Level {
                static_verts: v,
                colliders: Vec::new(),
                lights: Vec::new(),
                waypoints: waypoints_t,
                zones: zones_t,
                water_rects: Vec::new(),
                lava_rects: Vec::new(),
                anchors: Anchors {
                    cages: [Vec3::ZERO; 5],
                    chests: [Vec3::ZERO; 2],
                    merchant: Vec3::ZERO,
                    rune_tomb: Vec3::ZERO,
                    crypt_entry: Vec3::ZERO,
                    crypt_chest: Vec3::ZERO,
                    boss_arena: Vec3::ZERO,
                    exit_portal: Vec3::ZERO,
                    caravan: Vec3::ZERO,
                    village: Vec3::ZERO,
                    checkpoints: vec![Vec3::new(0.0, 0.0, 3.0)],
                },
                bounds: (-10.0, 10.0, -10.0, 10.0),
                flame_positions: Vec::new(),
            };
        }
        // ---- TEST : terrain seul ----
        if std::env::var("MCD_TERRAIN_ONLY").is_ok() {
            let waypoints_t = PATH
                .iter()
                .map(|(x, z)| Vec3::new(*x, 0.0, *z))
                .collect();
            let zones_t = vec![Zone { name: "Test", center: Vec3::ZERO }];
            return Level {
                static_verts: v,
                colliders,
                lights,
                waypoints: waypoints_t,
                zones: zones_t,
                water_rects: Vec::new(),
                lava_rects: Vec::new(),
                anchors: Anchors {
                    cages: [Vec3::ZERO; 5],
                    chests: [Vec3::ZERO; 2],
                    merchant: Vec3::ZERO,
                    rune_tomb: Vec3::ZERO,
                    crypt_entry: Vec3::ZERO,
                    crypt_chest: Vec3::ZERO,
                    boss_arena: Vec3::ZERO,
                    exit_portal: Vec3::ZERO,
                    caravan: Vec3::ZERO,
                    village: Vec3::new(60.0, 0.0, 100.0),
                    checkpoints: vec![Vec3::new(0.0, 0.0, 6.0)],
                },
                bounds,
                flame_positions: Vec::new(),
            };
        }

        // ================= EAU / LAVE (tuilées par bloc, comme Minecraft) =================
        let mut water_rects = Vec::new();
        let mut lava_rects = Vec::new();
        // Lac de départ
        water_rects.push([-30.0, -26.0, 55.0, 22.0]);
        push_liquid_grid(&mut v, [-30.0, -26.0, 55.0, 22.0], -0.15, SLOT_WATER, 0.82, &mut rng);
        // Rivière
        water_rects.push([-92.0, 128.0, 150.0, 13.0]);
        push_liquid_grid(&mut v, [-92.0, 128.0, 150.0, 13.0], -0.15, SLOT_WATER, 0.82, &mut rng);
        // Lave (arène du boss)
        lava_rects.push([-104.0, 244.0, 48.0, 28.0]);
        push_liquid_grid(&mut v, [-104.0, 244.0, 48.0, 28.0], -0.02, SLOT_LAVA, 1.0, &mut rng);
        lights.push((Vec3::new(-80.0, 1.5, 258.0), 26.0, Vec3::new(1.0, 0.42, 0.12), 0.9));

        // Pont de pierre sur la lave (arène)
        for i in 0..10 {
            let z = 246.0 + i as f32 * 2.6;
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(-80.0, 0.22, z)),
                Vec3::new(5.5, 0.3, 1.4),
                [SLOT_COBBLE, SLOT_COBBLE, SLOT_COBBLE, SLOT_STONE, SLOT_COBBLE, SLOT_COBBLE],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
            for sx in [-5.4f32, 5.4] {
                push_box(
                    &mut v,
                    Mat4::from_translation(Vec3::new(-80.0 + sx, 1.1, z)),
                    Vec3::new(0.22, 0.6, 1.4),
                    [SLOT_STONE; 6],
                    Vec4::ONE,
                    [0.0, 1.0, 0.0, 0.0],
                );
            }
        }
        colliders.push(Collider { x: -80.0, z: 258.0, hx: 6.2, hz: 14.5 });

        // Ponts de planches sur la rivière (alignés au chemin)
        for (bx, bz) in [(50.0f32, 134.5f32), (23.0, 134.5), (-12.0, 134.5)] {
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(bx, 0.1, bz)),
                Vec3::new(4.0, 0.12, 6.5),
                [SLOT_PLANK; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
        }

        // Ponton du lac
        push_box(
            &mut v,
            Mat4::from_translation(Vec3::new(-2.0, 0.1, -14.0)),
            Vec3::new(2.4, 0.12, 8.0),
            [SLOT_PLANK; 6],
            Vec4::ONE,
            [0.0, 1.0, 0.0, 0.0],
        );

        // ================= ARBRES (densité façon Creeper Woods) =================
        let mut placed = 0usize;
        while placed < 620 {
            let x = rng.range(bounds.0 + 3.0, bounds.1 - 3.0);
            let z = rng.range(bounds.2 + 3.0, bounds.3 - 3.0);
            let d = dist_to_path(x, z);
            if d < 6.8 || d > 44.0 {
                continue;
            }
            if in_water(x, z) || in_lava(x, z) {
                continue;
            }
            // éclaircie autour du camp de départ
            if (x - 0.0).powi(2) + (z + 6.0).powi(2) < 14.0f32.powi(2) {
                continue;
            }
            let n = rng.noise2(x * 0.05, z * 0.05);
            if n < 0.42 {
                continue;
            }
            // Éclaircies autour des zones construites
            let near_build = [(95.0f32, 68.0f32), (35.0, 176.0), (0.0, 202.0), (-35.0, 226.0), (-84.0, 290.0)]
                .iter()
                .any(|&(bx, bz)| (x - bx).powi(2) + (z - bz).powi(2) < 18.0f32.powi(2));
            if near_build {
                continue;
            }
            let h = rng.range(2.8, 4.6);
            let dark = rng.f32() > 0.55;
            let leaf_slot = if dark { SLOT_LEAVES_DARK } else { SLOT_LEAVES };
            let xj = x + rng.range(-0.3, 0.3);
            let zj = z + rng.range(-0.3, 0.3);
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(xj, h * 0.5, zj)),
                Vec3::new(0.42, h * 0.5, 0.42),
                [SLOT_TRUNK; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
            let lw = rng.range(1.7, 2.6);
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(xj, h + 0.9, zj)),
                Vec3::new(lw, 1.1, lw),
                [leaf_slot; 6],
                Vec4::new(1.05, 1.05, 1.05, 1.0),
                [0.0, 1.0, 0.0, 0.0],
            );
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(xj, h + 2.6, zj)),
                Vec3::new(lw * 0.72, 1.0, lw * 0.72),
                [leaf_slot; 6],
                Vec4::new(1.1, 1.1, 1.1, 1.0),
                [0.0, 1.0, 0.0, 0.0],
            );
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(xj, h + 4.0, zj)),
                Vec3::new(lw * 0.4, 0.7, lw * 0.4),
                [leaf_slot; 6],
                Vec4::new(1.15, 1.15, 1.15, 1.0),
                [0.0, 1.0, 0.0, 0.0],
            );
            colliders.push(Collider { x: xj, z: zj, hx: 0.5, hz: 0.5 });
            placed += 1;
        }

        // Buissons rouges + rochers + souches en bordure du chemin
        for _ in 0..240 {
            let x = rng.range(bounds.0 + 3.0, bounds.1 - 3.0);
            let z = rng.range(bounds.2 + 3.0, bounds.3 - 3.0);
            let d = dist_to_path(x, z);
            if d < 4.4 || d > 14.0 || in_water(x, z) || in_lava(x, z) {
                continue;
            }
            let kind = rng.i64(0, 3);
            match kind {
                0 => push_box(
                    &mut v,
                    Mat4::from_translation(Vec3::new(x, 0.5, z)),
                    Vec3::new(0.9, 0.5, 0.9),
                    [SLOT_BUSH_RED; 6],
                    Vec4::ONE,
                    [0.0, 1.0, 0.0, 0.0],
                ),
                1 => {
                    let s = rng.range(0.4, 0.9);
                    push_box(
                        &mut v,
                        Mat4::from_translation(Vec3::new(x, s * 0.5, z))
                            * Mat4::from_rotation_y(rng.range(0.0, 3.0)),
                        Vec3::new(s, s * 0.5, s * 0.8),
                        [SLOT_STONE; 6],
                        Vec4::ONE,
                        [0.0, 1.0, 0.0, 0.0],
                    );
                    colliders.push(Collider { x, z, hx: s, hz: s * 0.8 });
                }
                _ => push_box(
                    &mut v,
                    Mat4::from_translation(Vec3::new(x, 0.22, z)),
                    Vec3::new(0.55, 0.22, 0.55),
                    [SLOT_TRUNK; 6],
                    Vec4::ONE,
                    [0.0, 1.0, 0.0, 0.0],
                ),
            }
        }

        // ================= CHAMPIGNONS LUMINEUX =================
        for _ in 0..34 {
            let x = rng.range(bounds.0 + 5.0, bounds.1 - 5.0);
            let z = rng.range(bounds.2 + 5.0, bounds.3 - 5.0);
            let d = dist_to_path(x, z);
            if d < 3.5 || d > 20.0 || in_water(x, z) || in_lava(x, z) {
                continue;
            }
            let h = rng.range(1.2, 3.4);
            let r = h * 0.55;
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(x, h * 0.5, z)),
                Vec3::new(0.22, h * 0.5, 0.22),
                [SLOT_MUSH_STEM; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(x, h + 0.25, z)),
                Vec3::new(r, 0.42, r),
                [SLOT_MUSH_CAP; 6],
                Vec4::new(1.25, 1.2, 1.2, 1.0),
                [1.0, 1.0, 0.0, 0.0],
            );
            lights.push((Vec3::new(x, h + 0.6, z), 6.5, Vec3::new(1.0, 0.55, 0.3), 0.55));
        }

        // Torches le long du chemin
        let mut tdist = 6.0;
        while tdist < 520.0 {
            // avance le long du chemin
            let mut acc = 0.0;
            let mut pos = None;
            for w in PATH.windows(2) {
                let a = Vec3::new(w[0].0, 0.0, w[0].1);
                let b = Vec3::new(w[1].0, 0.0, w[1].1);
                let seg = (b - a).length();
                if acc + seg >= tdist {
                    let t = (tdist - acc) / seg;
                    pos = Some(a.lerp(b, t));
                    break;
                }
                acc += seg;
            }
            let Some(base) = pos else { break };
            let side = if (tdist as i64) % 2 == 0 { 2.6 } else { -2.6 };
            let dir = Vec3::new(1.0, 0.0, 0.0);
            let off = dir * side;
            let px = base.x + off.x;
            let pz = base.z + off.z;
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(px, 1.0, pz)),
                Vec3::new(0.14, 1.0, 0.14),
                [SLOT_TRUNK; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
            flames.push(Vec3::new(px, 2.15, pz));
            lights.push((Vec3::new(px, 2.2, pz), 5.5, Vec3::new(1.0, 0.62, 0.25), 0.6));
            tdist += 16.0;
        }

        // ================= AUBERGE (coffre 1) =================
        Self::building(&mut v, &mut colliders, Vec3::new(95.0, 0.0, 68.0), 9.0, 7.0, SLOT_ROOF_RED);
        lights.push((Vec3::new(95.0, 2.5, 64.0), 6.0, Vec3::new(1.0, 0.6, 0.28), 0.8));

        // ================= VILLAGE (5 cages) =================
        let village = Vec3::new(35.0, 0.0, 176.0);
        let cage_pos = [
            Vec3::new(27.0, 0.0, 170.0),
            Vec3::new(43.0, 0.0, 171.0),
            Vec3::new(26.0, 0.0, 182.0),
            Vec3::new(44.0, 0.0, 183.0),
            Vec3::new(35.0, 0.0, 186.0),
        ];
        // maisons autour
        for (i, &(dx, dz)) in [(14.0f32, -6.0f32), (-13.0, -4.0), (12.0, 8.0), (-11.0, 9.0), (2.0, -12.0)]
            .iter()
            .enumerate()
        {
            let roof = if i % 2 == 0 { SLOT_ROOF_DARK } else { SLOT_ROOF_RED };
            Self::building(&mut v, &mut colliders, Vec3::new(village.x + dx, 0.0, village.z + dz), 6.0, 5.0, roof);
        }
        for c in cage_pos {
            Self::cage(&mut v, c);
            colliders.push(Collider { x: c.x, z: c.z, hx: 0.95, hz: 0.95 });
        }
        lights.push((Vec3::new(village.x, 3.0, village.z), 9.0, Vec3::new(1.0, 0.65, 0.3), 0.9));

        // ================= CIMETIÈRE =================
        let cem = Vec3::new(-14.0, 0.0, 206.0);
        for i in 0..14 {
            let tx = cem.x - 7.0 + (i % 5) as f32 * 3.4;
            let tz = cem.z + (i / 5) as f32 * 3.6;
            if (tx - cem.x).abs() < 2.0 && i == 7 {
                continue;
            }
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(tx, 0.75, tz)) * Mat4::from_rotation_y(rng.range(-0.12, 0.12)),
                Vec3::new(0.55, 0.75, 0.16),
                [SLOT_TOMB; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
        }
        // clôture de pierre
        for i in 0..12 {
            let fx = cem.x - 9.0 + i as f32 * 1.8;
            push_box(&mut v, Mat4::from_translation(Vec3::new(fx, 0.6, cem.z - 8.0)), Vec3::new(0.85, 0.6, 0.2), [SLOT_MOSSY; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            push_box(&mut v, Mat4::from_translation(Vec3::new(fx, 0.6, cem.z + 8.0)), Vec3::new(0.85, 0.6, 0.2), [SLOT_MOSSY; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }
        // tombe à rune (secret Creepy Crypt)
        let rune_tomb = Vec3::new(cem.x + 2.0, 0.0, cem.z);
        push_box(&mut v, Mat4::from_translation(Vec3::new(rune_tomb.x, 0.9, rune_tomb.z)), Vec3::new(1.6, 0.9, 1.2), [SLOT_RUNE; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        push_box(&mut v, Mat4::from_translation(Vec3::new(rune_tomb.x - 1.9, 0.35, rune_tomb.z)), Vec3::new(0.3, 0.35, 0.3), [SLOT_REDSTONE; 6], Vec4::new(1.4, 1.2, 1.2, 1.0), [1.0, 1.0, 0.0, 0.0]);
        lights.push((Vec3::new(rune_tomb.x, 1.4, rune_tomb.z), 5.0, Vec3::new(0.3, 0.7, 1.0), 0.7));

        // ================= CAMP DE DÉPART (écran-titre) =================
        let camp = Vec3::new(0.0, 0.0, -6.0);
        // feu de camp
        for i in 0..4 {
            let a = i as f32 * std::f32::consts::FRAC_PI_2;
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(camp.x + a.cos() * 0.5, 0.18, camp.z + a.sin() * 0.5))
                    * Mat4::from_rotation_y(a),
                Vec3::new(0.55, 0.14, 0.16),
                [SLOT_TRUNK; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
        }
        flames.push(Vec3::new(camp.x, 0.55, camp.z));
        lights.push((Vec3::new(camp.x, 1.2, camp.z), 8.5, Vec3::new(1.0, 0.58, 0.22), 0.85));
        for (bx, bz) in [(2.2f32, -4.0f32), (-2.4, -7.5)] {
            push_box(&mut v, Mat4::from_translation(Vec3::new(bx, 0.5, bz)), Vec3::new(0.45, 0.5, 0.45), [SLOT_TRUNK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }

        // ================= CARAVANE =================
        let caravan = Vec3::new(-38.0, 0.0, 226.0);
        for i in 0..2 {
            let wx = caravan.x + i as f32 * 5.0;
            push_box(&mut v, Mat4::from_translation(Vec3::new(wx, 0.9, caravan.z)), Vec3::new(1.8, 0.7, 1.1), [SLOT_HAY; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            push_box(&mut v, Mat4::from_translation(Vec3::new(wx, 0.25, caravan.z)), Vec3::new(1.6, 0.25, 0.95), [SLOT_TRUNK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }
        // charrette renversée
        push_box(&mut v, Mat4::from_translation(Vec3::new(caravan.x + 3.0, 0.5, caravan.z - 2.6)) * Mat4::from_rotation_y(0.6), Vec3::new(1.2, 0.4, 0.7), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);

        // ================= GROTTE D'ARAIGNÉES (recoin sombre) =================
        let cave = Vec3::new(120.0, 0.0, 112.0);
        for i in 0..5 {
            let t = i as f32 / 4.0;
            let cx = 104.0 + t * 16.0;
            let cz = 104.0 + t * 8.0;
            for s in [-2.2f32, 2.2] {
                push_box(&mut v, Mat4::from_translation(Vec3::new(cx + s, 1.6, cz)), Vec3::new(0.5, 1.6, 2.4), [SLOT_STONE; 6], Vec4::new(0.5, 0.5, 0.55, 1.0), [0.0, 1.0, 0.0, 0.0]);
                colliders.push(Collider { x: cx + s, z: cz, hx: 0.5, hz: 2.4 });
            }
        }
        push_box(&mut v, Mat4::from_translation(Vec3::new(cave.x, 0.06, cave.z)), Vec3::new(3.4, 0.06, 3.4), [SLOT_COBBLE; 6], Vec4::new(0.6, 0.6, 0.65, 1.0), [0.0, 1.0, 0.0, 0.0]);
        // toiles
        push_box(&mut v, Mat4::from_translation(Vec3::new(cave.x - 1.4, 1.4, cave.z - 1.4)) * Mat4::from_rotation_y(0.7), Vec3::new(0.04, 1.4, 1.4), [30; 6], Vec4::ONE, [0.0, 0.85, 0.0, 0.0]);
        push_box(&mut v, Mat4::from_translation(Vec3::new(cave.x + 1.6, 1.2, cave.z + 1.2)) * Mat4::from_rotation_y(-0.5), Vec3::new(0.04, 1.2, 1.2), [30; 6], Vec4::ONE, [0.0, 0.85, 0.0, 0.0]);
        lights.push((Vec3::new(cave.x, 1.0, cave.z), 4.0, Vec3::new(0.4, 0.5, 0.8), 0.35));

        // ================= CREEPER ROOM =================
        let croom = Vec3::new(88.0, 0.0, 152.0);
        push_box(&mut v, Mat4::from_translation(Vec3::new(croom.x, 0.04, croom.z)), Vec3::new(5.0, 0.04, 5.0), [SLOT_MOSSY; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        for i in 0..4 {
            let a = i as f32 * std::f32::consts::FRAC_PI_2 + 0.4;
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(croom.x + a.cos() * 4.2, 1.5, croom.z + a.sin() * 4.2)),
                Vec3::new(0.6, 1.5, 0.6),
                [SLOT_OBSIDIAN; 6],
                Vec4::ONE,
                [0.0, 1.0, 0.0, 0.0],
            );
            colliders.push(Collider { x: croom.x + a.cos() * 4.2, z: croom.z + a.sin() * 4.2, hx: 0.6, hz: 0.6 });
        }
        lights.push((Vec3::new(croom.x, 1.6, croom.z), 5.5, Vec3::new(0.4, 1.0, 0.45), 0.55));

        // ================= TEMPLE DE SORTIE =================
        let temple = Vec3::new(-84.0, 0.0, 296.0);
        // marches
        for i in 0..3 {
            push_box(&mut v, Mat4::from_translation(Vec3::new(temple.x, 0.15 + i as f32 * 0.3, temple.z - 5.0 + i as f32 * 1.6)), Vec3::new(4.5, 0.3, 1.6), [SLOT_TEMPLE; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }
        // piliers + linteau
        for sx in [-3.6f32, 3.6] {
            push_box(&mut v, Mat4::from_translation(Vec3::new(temple.x + sx, 2.2, temple.z + 1.5)), Vec3::new(0.7, 2.2, 0.7), [SLOT_TEMPLE; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            colliders.push(Collider { x: temple.x + sx, z: temple.z + 1.5, hx: 0.7, hz: 0.7 });
        }
        push_box(&mut v, Mat4::from_translation(Vec3::new(temple.x, 4.6, temple.z + 1.5)), Vec3::new(4.4, 0.6, 0.9), [SLOT_TEMPLE; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        // portail de sortie
        push_box(&mut v, Mat4::from_translation(Vec3::new(temple.x, 1.6, temple.z + 1.5)), Vec3::new(2.4, 1.6, 0.24), [SLOT_PORTAL; 6], Vec4::new(1.6, 1.4, 1.8, 1.0), [1.0, 1.0, 0.0, 0.02]);
        lights.push((Vec3::new(temple.x, 1.8, temple.z + 1.2), 8.0, Vec3::new(0.6, 0.3, 1.0), 1.0));
        // dallage obsidienne
        push_box(&mut v, Mat4::from_translation(Vec3::new(temple.x, 0.05, temple.z + 4.0)), Vec3::new(4.0, 0.05, 2.5), [SLOT_OBSIDIAN; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);

        // Bannières décoratives le long du chemin
        for (bx, bz) in [(95.0f32, 62.0f32), (35.0, 169.0), (-14.0, 196.0)] {
            push_box(&mut v, Mat4::from_translation(Vec3::new(bx, 1.6, bz)), Vec3::new(0.1, 1.6, 0.1), [SLOT_TRUNK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            push_box(&mut v, Mat4::from_translation(Vec3::new(bx + 0.55, 2.4, bz)), Vec3::new(0.5, 1.1, 0.04), [SLOT_BANNER; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }

        // Plaques de mousse lumineuse au sol
        for _ in 0..46 {
            let x = rng.range(bounds.0 + 4.0, bounds.1 - 4.0);
            let z = rng.range(bounds.2 + 4.0, bounds.3 - 4.0);
            if dist_to_path(x, z) < 3.0 || in_water(x, z) || in_lava(x, z) {
                continue;
            }
            let s = rng.range(0.8, 2.2);
            push_box(
                &mut v,
                Mat4::from_translation(Vec3::new(x, 0.28, z)),
                Vec3::new(s, 0.02, s),
                [SLOT_GLOW_MOSS; 6],
                Vec4::new(0.85, 0.85, 0.85, 1.0),
                [0.85, 1.0, 0.0, 0.0],
            );
        }

        let waypoints = PATH
            .iter()
            .map(|(x, z)| Vec3::new(*x, 0.0, *z))
            .collect();

        let zones = vec![
            Zone { name: "Lac aux têtes de pierre", center: Vec3::new(0.0, 0.0, -10.0) },
            Zone { name: "Bois profonds", center: Vec3::new(60.0, 0.0, 24.0) },
            Zone { name: "Auberge", center: Vec3::new(95.0, 0.0, 68.0) },
            Zone { name: "Grotte d'araignées", center: Vec3::new(120.0, 0.0, 112.0) },
            Zone { name: "Creeper Room", center: Vec3::new(88.0, 0.0, 152.0) },
            Zone { name: "Rivière", center: Vec3::new(23.0, 0.0, 134.0) },
            Zone { name: "Village", center: village },
            Zone { name: "Cimetière", center: cem },
            Zone { name: "Caravane", center: caravan },
            Zone { name: "Pont de lave", center: Vec3::new(-80.0, 0.0, 258.0) },
            Zone { name: "Temple", center: temple },
        ];

        Level {
            static_verts: v,
            colliders,
            lights,
            waypoints,
            zones,
            water_rects,
            lava_rects,
            anchors: Anchors {
                cages: cage_pos,
                chests: [Vec3::new(95.0, 0.0, 65.4), Vec3::new(-41.0, 0.0, 224.0)],
                merchant: Vec3::new(39.0, 0.0, 173.5),
                rune_tomb,
                crypt_entry: Vec3::new(120.0, 0.0, 112.0),
                crypt_chest: Vec3::new(121.5, 0.0, 110.0),
                boss_arena: Vec3::new(-80.0, 0.0, 258.0),
                exit_portal: Vec3::new(temple.x, 0.0, temple.z + 2.2),
                caravan,
                village,
                checkpoints: vec![
                    Vec3::new(0.0, 0.0, -2.0),
                    Vec3::new(95.0, 0.0, 70.0),
                    Vec3::new(35.0, 0.0, 178.0),
                    Vec3::new(-38.0, 0.0, 228.0),
                    Vec3::new(-80.0, 0.0, 252.0),
                ],
            },
            bounds,
            flame_positions: flames,
        }
    }

    /// Bâtiment simple : murs en planches, porte au centre-avant, toit plat.
    fn building(v: &mut Vec<DynVertex>, colliders: &mut Vec<Collider>, base: Vec3, w: f32, d: f32, roof_slot: usize) {
        let h = 3.2;
        let hw = w * 0.5;
        let hd = d * 0.5;
        // sol intérieur
        push_box(v, Mat4::from_translation(base + Vec3::new(0.0, 0.04, 0.0)), Vec3::new(hw - 0.2, 0.04, hd - 0.2), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        // mur arrière (z = -hd)
        push_box(v, Mat4::from_translation(base + Vec3::new(0.0, h * 0.5, -hd)), Vec3::new(hw, h * 0.5, 0.2), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        // murs latéraux
        push_box(v, Mat4::from_translation(base + Vec3::new(hw, h * 0.5, 0.0)), Vec3::new(0.2, h * 0.5, hd), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        push_box(v, Mat4::from_translation(base + Vec3::new(-hw, h * 0.5, 0.0)), Vec3::new(0.2, h * 0.5, hd), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        // mur avant (z = +hd) avec ouverture de porte (1.2 de large, 2.3 de haut)
        let seg = (hw - 0.6) * 0.5;
        let seg_cx = (hw + 0.6) * 0.5;
        for sx in [-seg_cx, seg_cx] {
            push_box(v, Mat4::from_translation(base + Vec3::new(sx, h * 0.5, hd)), Vec3::new(seg, h * 0.5, 0.2), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }
        push_box(v, Mat4::from_translation(base + Vec3::new(0.0, (2.3 + h) * 0.5, hd)), Vec3::new(0.6, (h - 2.3) * 0.5, 0.2), [SLOT_PLANK; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        colliders.push(Collider { x: base.x, z: base.z - hd, hx: hw, hz: 0.2 });
        colliders.push(Collider { x: base.x + hw, z: base.z, hx: 0.2, hz: hd });
        colliders.push(Collider { x: base.x - hw, z: base.z, hx: 0.2, hz: hd });
        colliders.push(Collider { x: base.x - seg_cx, z: base.z + hd, hx: seg, hz: 0.2 });
        colliders.push(Collider { x: base.x + seg_cx, z: base.z + hd, hx: seg, hz: 0.2 });
        // porte
        push_box(v, Mat4::from_translation(base + Vec3::new(0.6, 1.15, hd)), Vec3::new(0.55, 1.15, 0.08), [SLOT_DOOR; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        // toit
        push_box(v, Mat4::from_translation(base + Vec3::new(0.0, h + 0.3, 0.0)), Vec3::new(hw + 0.7, 0.3, hd + 0.7), [roof_slot; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        // fenêtre lumineuse
        push_box(v, Mat4::from_translation(base + Vec3::new(hw + 0.05, 2.1, -hd * 0.4)), Vec3::new(0.06, 0.5, 0.7), [SLOT_GLOW_MOSS; 6], Vec4::new(1.5, 1.4, 1.0, 1.0), [1.0, 1.0, 0.0, 0.0]);
        // bibliothèque intérieure
        push_box(v, Mat4::from_translation(base + Vec3::new(-hw + 0.6, 0.9, -hd + 0.8)), Vec3::new(0.35, 0.9, 1.0), [SLOT_BOOKSHELF; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
    }

    /// Cage à villageois : barreaux fins + toit, ouverture au-dessus.
    fn cage(v: &mut Vec<DynVertex>, base: Vec3) {
        let s = 0.95;
        for i in 0..4 {
            let t = -s + i as f32 * (2.0 * s / 3.0);
            push_box(v, Mat4::from_translation(base + Vec3::new(t, 0.85, -s)), Vec3::new(0.07, 0.85, 0.07), [SLOT_GOLD; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            push_box(v, Mat4::from_translation(base + Vec3::new(t, 0.85, s)), Vec3::new(0.07, 0.85, 0.07), [SLOT_GOLD; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            push_box(v, Mat4::from_translation(base + Vec3::new(-s, 0.85, t)), Vec3::new(0.07, 0.85, 0.07), [SLOT_GOLD; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
            push_box(v, Mat4::from_translation(base + Vec3::new(s, 0.85, t)), Vec3::new(0.07, 0.85, 0.07), [SLOT_GOLD; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        }
        push_box(v, Mat4::from_translation(base + Vec3::new(0.0, 1.75, 0.0)), Vec3::new(s + 0.1, 0.1, s + 0.1), [SLOT_GOLD; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
        push_box(v, Mat4::from_translation(base + Vec3::new(0.0, 0.04, 0.0)), Vec3::new(s, 0.04, s), [SLOT_COBBLE; 6], Vec4::ONE, [0.0, 1.0, 0.0, 0.0]);
    }

    pub fn blocked(&self, x: f32, z: f32, radius: f32) -> bool {
        for c in &self.colliders {
            if (x - c.x).abs() < c.hx + radius && (z - c.z).abs() < c.hz + radius {
                return true;
            }
        }
        false
    }

    /// Déplacement avec glissement le long des colliders + limites du monde.
    pub fn slide(&self, pos: &mut Vec3, vel: Vec3, radius: f32) {
        let (minx, maxx, minz, maxz) = self.bounds;
        let nx = pos.x + vel.x;
        if !self.blocked(nx, pos.z, radius) {
            pos.x = nx.clamp(minx + radius, maxx - radius);
        }
        let nz = pos.z + vel.z;
        if !self.blocked(pos.x, nz, radius) {
            pos.z = nz.clamp(minz + radius, maxz - radius);
        }
    }

    /// Nom de la zone la plus proche (pour l'objectif/le HUD).
    pub fn zone_name(&self, pos: Vec3) -> &'static str {
        let mut best = 0usize;
        let mut best_d = f32::MAX;
        for (i, z) in self.zones.iter().enumerate() {
            let d = (z.center - pos).length_squared();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        self.zones[best].name
    }
}
