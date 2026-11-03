//! Boucle de jeu : écrans, mission Creeper Woods, combat, objectifs, ramassages.

use crate::camera::Camera;
use crate::input::Input;
use crate::level::{Level, SLOT_GOLD, SLOT_PORTAL};
use crate::mobs::{self, AiCtx, Mob, MobEvent, MobKind, MobState};
use crate::particles::Particles;
use crate::player::{Player, HORN_SOUL_COST, MAX_HP};
use crate::rng::Rng;
use crate::vertex::{push_box, DynVertex};
use glam::{Mat4, Vec3, Vec4};
use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Title,
    Briefing,
    Playing,
    Paused,
    Map,
    Inventory,
    GameOver,
    Victory,
    Controls,
}

#[derive(Clone, PartialEq)]
pub enum Quest {
    ReachVillage,
    FreeVillagers { freed: usize },
    FindCaravan,
    Boss,
    Escape,
}

impl Quest {
    pub fn label(&self) -> String {
        match self {
            Quest::ReachVillage => "Rejoindre le village".to_string(),
            Quest::FreeVillagers { freed } => format!("Libérer les villageois — {}/5", freed),
            Quest::FindCaravan => "Trouver la caravane".to_string(),
            Quest::Boss => "Affronter le Nécromancien".to_string(),
            Quest::Escape => "S'échapper par le temple".to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProjectileKind {
    ArrowFriendly,
    ArrowEnemy,
    Orb,
}

pub struct Projectile {
    pub pos: Vec3,
    pub vel: Vec3,
    pub kind: ProjectileKind,
    pub life: f32,
}

pub enum PickupKind {
    Emerald,
    Arrows,
}

pub struct Pickup {
    pub kind: PickupKind,
    pub pos: Vec3,
    pub bob: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InteractKind {
    Cage(usize),
    Chest(usize),
    Merchant,
    Rune,
    CryptChest,
    Portal,
}

pub struct Interactable {
    pub kind: InteractKind,
    pub pos: Vec3,
    pub done: bool,
}

impl Interactable {
    pub fn label(&self, crypt_open: bool) -> &'static str {
        match self.kind {
            InteractKind::Cage(_) => "Ouvrir la cage",
            InteractKind::Chest(_) => "Ouvrir le coffre",
            InteractKind::Merchant => "Délivrer le marchand",
            InteractKind::Rune => "Appuyer sur la rune",
            InteractKind::CryptChest => "Piller le trésor",
            InteractKind::Portal => {
                if crypt_open {
                    "S'échapper !"
                } else {
                    "Le portail est scellé"
                }
            }
        }
    }
}

pub struct FloatText {
    pub pos: Vec3,
    pub text: String,
    pub color: [f32; 4],
    pub life: f32,
}

pub struct Game {
    pub screen: Screen,
    pub level: Level,
    pub player: Player,
    pub mobs: Vec<Mob>,
    pub projectiles: Vec<Projectile>,
    pub pickups: Vec<Pickup>,
    pub interactables: Vec<Interactable>,
    pub particles: Particles,
    pub float_texts: Vec<FloatText>,
    pub quest: Quest,
    pub rng: Rng,
    pub time: f32,
    pub mission_time: f32,
    pub cam: Camera,
    pub checkpoint: Vec3,
    pub boss_alive: bool,
    pub crypt_open: bool,
    pub prompt: Option<String>,
    pub title_cursor: usize,
    pub damage_flash: f32,
    pub spawn_timer: f32,
    pub dynamic_lights: Vec<(Vec3, f32, Vec3, f32)>,
    pub victory_time: f32,
    pub controls_from: Screen,
}

impl Game {
    pub fn new(level: Level) -> Game {
        let mut cam = Camera::new();
        cam.snap(level.anchors.checkpoints[0]);
        let mut game = Game {
            screen: Screen::Title,
            player: Player::new(level.anchors.checkpoints[0]),
            mobs: Vec::new(),
            projectiles: Vec::new(),
            pickups: Vec::new(),
            interactables: Vec::new(),
            particles: Particles::new(),
            float_texts: Vec::new(),
            quest: Quest::ReachVillage,
            rng: Rng::new(0xBEEF_2026),
            time: 0.0,
            mission_time: 0.0,
            cam,
            checkpoint: level.anchors.checkpoints[0],
            boss_alive: false,
            crypt_open: false,
            prompt: None,
            title_cursor: 0,
            damage_flash: 0.0,
            spawn_timer: 2.0,
            dynamic_lights: Vec::new(),
            victory_time: 0.0,
            controls_from: Screen::Title,
            level,
        };
        game.setup_interactables();
        game
    }

    fn setup_interactables(&mut self) {
        let a = &self.level.anchors;
        self.interactables = Vec::new();
        for (i, c) in a.cages.iter().enumerate() {
            self.interactables.push(Interactable { kind: InteractKind::Cage(i), pos: *c, done: false });
        }
        for (i, c) in a.chests.iter().enumerate() {
            self.interactables.push(Interactable { kind: InteractKind::Chest(i), pos: *c, done: false });
        }
        self.interactables.push(Interactable { kind: InteractKind::Merchant, pos: a.merchant, done: false });
        self.interactables.push(Interactable { kind: InteractKind::Rune, pos: a.rune_tomb + Vec3::new(-1.9, 0.0, 0.0), done: false });
        self.interactables.push(Interactable { kind: InteractKind::CryptChest, pos: a.crypt_chest, done: false });
        self.interactables.push(Interactable { kind: InteractKind::Portal, pos: a.exit_portal + Vec3::new(0.0, 0.0, 1.6), done: false });
    }

    /// Démarre (ou redémarre) la mission.
    pub fn start_mission(&mut self) {
        let spawn = self.level.anchors.checkpoints[0];
        self.player = Player::new(spawn);
        self.mobs.clear();
        self.projectiles.clear();
        self.pickups.clear();
        self.particles.clear();
        self.float_texts.clear();
        self.quest = Quest::ReachVillage;
        self.mission_time = 0.0;
        self.checkpoint = spawn;
        self.boss_alive = false;
        self.crypt_open = false;
        self.damage_flash = 0.0;
        self.spawn_timer = 2.0;
        self.setup_interactables();
        self.cam.snap(spawn);
        // garnison de départ
        for i in 0..5 {
            let a = i as f32 / 5.0 * std::f32::consts::TAU;
            let pos = spawn + Vec3::new(a.cos() * 10.0, 0.0, a.sin() * 10.0);
            self.spawn_mob(MobKind::Zombie, pos, Some(false));
        }
        self.spawn_mob(MobKind::Skeleton, spawn + Vec3::new(-8.0, 0.0, 6.0), Some(false));
        // villageois dans les cages
        for c in self.level.anchors.cages {
            self.spawn_mob(MobKind::Villager, c, None);
        }
        self.screen = Screen::Briefing;
    }

    fn spawn_mob(&mut self, kind: MobKind, pos: Vec3, enchanted: Option<bool>) {
        let enc = enchanted.unwrap_or_else(|| {
            kind != MobKind::Villager && self.rng.f32() < 0.15
        });
        self.mobs.push(Mob::new(kind, pos, enc));
    }

    pub fn update(&mut self, dt: f32, input: &mut Input) {
        self.time += dt;
        match self.screen {
            Screen::Title => self.update_title(input),
            Screen::Briefing => {
                if input.just_pressed(KeyCode::Enter) || input.mouse_just_pressed(true) {
                    self.screen = Screen::Playing;
                }
            }
            Screen::Playing => self.simulate(dt, input),
            Screen::Paused | Screen::Map | Screen::Inventory | Screen::Controls => {
                self.update_overlays(input)
            }
            Screen::GameOver => {
                if input.just_pressed(KeyCode::Enter) {
                    self.start_mission();
                    self.screen = Screen::Playing;
                }
            }
            Screen::Victory => {
                if input.just_pressed(KeyCode::Enter) {
                    self.screen = Screen::Title;
                    self.title_cursor = 0;
                }
            }
        }
        input.end_frame();
    }

    fn update_title(&mut self, input: &Input) {
        // caméra qui orbite lentement autour du camp
        let t = self.time * 0.08;
        let center = Vec3::new(0.0, 0.0, -6.0);
        self.cam.target = center + Vec3::new(0.0, 1.2, 0.0);
        self.cam.eye = center + Vec3::new(t.cos() * 14.0, 9.0, t.sin() * 14.0);
        // ambiance du camp
        let fire = Vec3::new(0.0, 0.7, -6.0);
        if self.rng.f32() < 0.5 {
            self.particles.flame(&mut self.rng, fire);
            self.particles.ember(&mut self.rng, fire);
        }
        let up = [KeyCode::KeyW, KeyCode::ArrowUp];
        let down = [KeyCode::KeyS, KeyCode::ArrowDown];
        if up.iter().any(|&k| input.just_pressed(k)) {
            self.title_cursor = (self.title_cursor + 2) % 3;
        }
        if down.iter().any(|&k| input.just_pressed(k)) {
            self.title_cursor = (self.title_cursor + 1) % 3;
        }
        if input.just_pressed(KeyCode::Enter) || input.mouse_just_pressed(true) {
            match self.title_cursor {
                0 => self.start_mission(),
                1 => {
                    self.controls_from = Screen::Title;
                    self.screen = Screen::Controls;
                }
                _ => std::process::exit(0),
            }
        }
    }

    fn update_overlays(&mut self, input: &Input) {
        match self.screen {
            Screen::Paused => {
                if input.just_pressed(KeyCode::Escape) {
                    self.screen = Screen::Playing;
                } else if input.just_pressed(KeyCode::KeyM) {
                    self.screen = Screen::Map;
                } else if input.just_pressed(KeyCode::KeyI) {
                    self.screen = Screen::Inventory;
                } else if input.just_pressed(KeyCode::KeyC) {
                    self.controls_from = Screen::Paused;
                    self.screen = Screen::Controls;
                }
            }
            Screen::Map => {
                if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::KeyM) {
                    self.screen = Screen::Playing;
                }
            }
            Screen::Inventory => {
                if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::KeyI) {
                    self.screen = Screen::Playing;
                }
            }
            Screen::Controls
                if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::Enter) =>
            {
                self.screen = self.controls_from;
            }
            _ => {}
        }
    }

    // ==================== SIMULATION EN JEU ====================

    fn simulate(&mut self, dt: f32, input: &Input) {
        if input.just_pressed(KeyCode::Escape) {
            self.screen = Screen::Paused;
            return;
        }
        if input.just_pressed(KeyCode::KeyM) {
            self.screen = Screen::Map;
            return;
        }
        if input.just_pressed(KeyCode::KeyI) {
            self.screen = Screen::Inventory;
            return;
        }
        self.mission_time += dt;
        self.damage_flash = (self.damage_flash - dt * 2.0).max(0.0);
        self.dynamic_lights.clear();
        self.prompt = None;

        // --- Mort du héros ---
        if self.player.dead {
            self.player.dead_timer -= dt;
            if self.player.dead_timer <= 0.0 {
                if self.player.lives > 1 {
                    self.player.lives -= 1;
                    self.player.dead = false;
                    self.player.hp = MAX_HP;
                    self.player.iframes = 2.0;
                    self.player.pos = self.checkpoint;
                    self.cam.snap(self.checkpoint);
                } else {
                    self.player.lives = 0;
                    self.screen = Screen::GameOver;
                    return;
                }
            }
        } else {
            self.player.tick(input, &self.level, dt);
            self.handle_combat(input);
        }

        // --- Artefacts ---
        self.handle_artifacts(input);

        // --- IA des mobs ---
        let player_pos = self.player.pos;
        let player_alive = !self.player.dead;
        let mut events: Vec<MobEvent> = Vec::new();
        for i in 0..self.mobs.len() {
            let is_boss = self.mobs[i].kind == MobKind::Necromancer;
            if !is_boss && self.mobs[i].pos.distance(player_pos) > 70.0 {
                continue;
            }
            let mut mob =
                std::mem::replace(&mut self.mobs[i], Mob::new(MobKind::Villager, Vec3::ZERO, false));
            {
                let mut ctx = AiCtx {
                    player_pos,
                    player_alive,
                    level: &self.level,
                    dt,
                    rng: &mut self.rng,
                    events: &mut events,
                };
                mobs::tick(&mut mob, &mut ctx);
            }
            self.mobs[i] = mob;
        }
        self.handle_mob_events(events);

        // --- Nettoyage des morts finis ---
        self.mobs
            .retain(|m| !m.dead || m.fade > 0.0 || m.kind == MobKind::Necromancer);

        // --- Projectiles ---
        let mut projs = std::mem::take(&mut self.projectiles);
        projs.retain_mut(|pr| self.update_projectile(pr, dt));
        self.projectiles = projs;

        // --- Ramassages ---
        let mut pickups = std::mem::take(&mut self.pickups);
        pickups.retain_mut(|pk| self.update_pickup(pk, dt));
        self.pickups = pickups;

        // --- Interactions ---
        self.update_interactions(input);

        // --- Objectifs ---
        self.update_quests();

        // --- Apparition d'ambiance ---
        self.spawn_timer -= dt;
        let alive = self
            .mobs
            .iter()
            .filter(|m| !m.dead && m.kind != MobKind::Villager)
            .count();
        if self.spawn_timer <= 0.0 && alive < 15 && !self.boss_alive {
            self.spawn_timer = 3.5;
            self.spawn_ambient();
        }

        // --- Particules d'ambiance ---
        if self.rng.f32() < dt * 14.0 {
            self.particles.leaf(&mut self.rng, self.cam.target, 18.0);
        }
        for f in &self.level.flame_positions {
            if f.distance(self.cam.target) < 40.0 && self.rng.f32() < dt * 9.0 {
                self.particles.flame(&mut self.rng, *f);
                self.particles.ember(&mut self.rng, *f);
            }
        }
        if let Some(tp) = self.player.totem_pos {
            if self.rng.f32() < dt * 16.0 {
                self.particles.totem_aura(&mut self.rng, tp);
            }
        }
        let ppos = self.player.pos;
        self.particles.update(dt, ppos);

        // --- Textes flottants ---
        self.float_texts.retain_mut(|t| {
            t.life -= dt;
            t.pos.y += dt * 1.2;
            t.life > 0.0
        });

        // --- Caméra ---
        let enemies_near = self
            .mobs
            .iter()
            .filter(|m| !m.dead && m.kind != MobKind::Villager && m.pos.distance(self.player.pos) < 10.0)
            .count();
        let zoom = if enemies_near >= 4 { 1.14 } else { 1.0 };
        self.cam.follow(self.player.pos, zoom, dt);
    }

    fn handle_combat(&mut self, input: &Input) {
        // Mêlée (clic gauche) : combo 3 coups mémorisé
        if input.mouse_just_pressed(true) && self.player.attack_cd <= 0.0 {
            self.player.combo_stage = if self.player.combo_timer > 0.0 {
                self.player.combo_stage % 3 + 1
            } else {
                1
            };
            self.player.combo_timer = 1.1;
            self.player.attack_cd = 0.42;
            self.player.attack_anim = 1.0;
            let dmg = [12.0f32, 12.0, 20.0][self.player.combo_stage as usize - 1];
            let crit = self.player.combo_stage == 3;
            self.melee_hit(dmg, crit);
        }
        // Arc (clic droit) avec auto-visée
        if input.mouse_just_pressed(false) && self.player.arrows > 0 && self.player.attack_cd <= 0.15 {
            self.player.arrows -= 1;
            self.player.attack_cd = 0.35;
            self.player.attack_anim = 0.7;
            let dir = self.aim_direction();
            self.projectiles.push(Projectile {
                pos: self.player.pos + Vec3::new(0.0, 1.4, 0.0) + dir * 0.5,
                vel: dir * 22.0,
                kind: ProjectileKind::ArrowFriendly,
                life: 2.0,
            });
        }
    }

    fn aim_direction(&self) -> Vec3 {
        let fwd = Vec3::new(self.player.yaw.cos(), 0.0, self.player.yaw.sin());
        let mut best: Option<(f32, Vec3)> = None;
        for m in &self.mobs {
            if m.dead || m.kind == MobKind::Villager {
                continue;
            }
            let to = m.pos - self.player.pos;
            let d = to.length();
            if d < 2.0 || d > 22.0 {
                continue;
            }
            let dot = to.normalize_or_zero().dot(fwd);
            if dot > 0.5 && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, to.normalize_or_zero()));
            }
        }
        best.map(|(_, d)| d).unwrap_or(fwd)
    }

    fn melee_hit(&mut self, dmg: f32, crit: bool) {
        let fwd = Vec3::new(self.player.yaw.cos(), 0.0, self.player.yaw.sin());
        let hit_pos = self.player.pos + fwd * 1.3 + Vec3::new(0.0, 1.0, 0.0);
        let mut killed: Vec<usize> = Vec::new();
        for i in 0..self.mobs.len() {
            let m = &mut self.mobs[i];
            if m.dead || m.kind == MobKind::Villager {
                continue;
            }
            let to = m.pos - self.player.pos;
            let d = to.length();
            if d < 2.3 && to.normalize_or_zero().dot(fwd) > 0.35 {
                m.hp -= dmg;
                m.hurt_flash = 1.0;
                if crit {
                    let kb = to.normalize_or_zero() * 3.5;
                    let r = m.radius();
                    self.level.slide(&mut m.pos, kb, r);
                }
                self.float_texts.push(FloatText {
                    pos: m.pos + Vec3::new(0.0, 1.8, 0.0),
                    text: format!("{}", dmg as i32),
                    color: if crit { [1.0, 0.82, 0.29, 1.0] } else { [1.0, 1.0, 1.0, 1.0] },
                    life: 0.8,
                });
                if m.hp <= 0.0 {
                    m.dead = true;
                    killed.push(i);
                }
            }
        }
        self.particles.hit_spark(&mut self.rng, hit_pos, crit);
        for i in killed {
            self.on_mob_death(i);
        }
    }

    fn on_mob_death(&mut self, idx: usize) {
        let kind = self.mobs[idx].kind;
        let enchanted = self.mobs[idx].enchanted;
        let pos = self.mobs[idx].pos;
        match kind {
            MobKind::Necromancer => {
                self.boss_alive = false;
                self.quest = Quest::Escape;
                self.checkpoint = self.level.anchors.boss_arena;
                self.particles.explosion(&mut self.rng, pos);
                self.particles.teleport_poof(&mut self.rng, pos);
                self.float_texts.push(FloatText {
                    pos: pos + Vec3::new(0.0, 2.5, 0.0),
                    text: "Le Nécromancien est vaincu !".to_string(),
                    color: [0.8, 0.5, 1.0, 1.0],
                    life: 3.0,
                });
                self.player.add_xp(60);
                for _ in 0..12 {
                    self.pickups.push(Pickup {
                        kind: PickupKind::Emerald,
                        pos: pos + Vec3::new(self.rng.range(-2.0, 2.0), 0.0, self.rng.range(-2.0, 2.0)),
                        bob: self.rng.range(0.0, 6.0),
                    });
                }
            }
            MobKind::Villager => {}
            _ => {
                self.player.add_souls(if enchanted { 24 } else { 12 });
                self.player.add_xp(if enchanted { 6 } else { 3 });
                self.particles.soul_wisp(&mut self.rng, pos);
                let n_em = self.rng.i64(1, 3) as u32 + u32::from(enchanted) * 2;
                for _ in 0..n_em {
                    self.pickups.push(Pickup {
                        kind: PickupKind::Emerald,
                        pos: pos + Vec3::new(self.rng.range(-0.8, 0.8), 0.0, self.rng.range(-0.8, 0.8)),
                        bob: self.rng.range(0.0, 6.0),
                    });
                }
                if self.rng.f32() < 0.28 {
                    self.pickups.push(Pickup {
                        kind: PickupKind::Arrows,
                        pos: pos + Vec3::new(self.rng.range(-0.5, 0.5), 0.0, self.rng.range(-0.5, 0.5)),
                        bob: 0.0,
                    });
                }
                if enchanted {
                    self.particles.teleport_poof(&mut self.rng, pos);
                }
            }
        }
    }

    fn handle_artifacts(&mut self, input: &Input) {
        // 1 : Flèche d'artifice
        if input.just_pressed(KeyCode::Digit1) && self.player.artifact_ready(0) {
            let targets: Vec<Vec3> = self
                .mobs
                .iter()
                .filter(|m| !m.dead && m.kind != MobKind::Villager && m.pos.distance(self.player.pos) < 18.0)
                .take(3)
                .map(|m| m.pos)
                .collect();
            if !targets.is_empty() {
                self.player.use_artifact(0, 12.0);
                for t in targets {
                    self.particles.firework_burst(&mut self.rng, t + Vec3::new(0.0, 1.2, 0.0));
                    self.dynamic_lights.push((t + Vec3::new(0.0, 1.2, 0.0), 8.0, Vec3::new(1.0, 0.6, 0.25), 1.4));
                    self.area_damage(t, 2.4, 26.0);
                }
            }
        }
        // 2 : Cor de vent (coûte 60 âmes)
        if input.just_pressed(KeyCode::Digit2) && self.player.artifact_ready(1) && self.player.souls >= HORN_SOUL_COST {
            self.player.souls -= HORN_SOUL_COST;
            self.player.use_artifact(1, 18.0);
            self.particles.horn_wave(self.player.pos);
            for i in 0..self.mobs.len() {
                let m = &mut self.mobs[i];
                if m.dead || m.kind == MobKind::Villager {
                    continue;
                }
                let to = m.pos - self.player.pos;
                if to.length() < 5.5 {
                    let push = to.normalize_or_zero() * 6.5;
                    let r = m.radius();
                    self.level.slide(&mut m.pos, push, r);
                    m.hp -= 5.0;
                    m.hurt_flash = 1.0;
                    if m.hp <= 0.0 {
                        m.dead = true;
                    }
                }
            }
        }
        // 3 : Totem de régénération
        if input.just_pressed(KeyCode::Digit3) && self.player.artifact_ready(2) {
            self.player.use_artifact(2, 45.0);
            self.player.totem_pos = Some(self.player.pos);
            self.player.totem_timer = 8.0;
        }
        // Potion
        if self.player.wants_potion(input) {
            self.player.drink_potion();
            self.particles.heal_green(self.player.pos);
            self.float_texts.push(FloatText {
                pos: self.player.pos + Vec3::new(0.0, 2.0, 0.0),
                text: "+75 PV".to_string(),
                color: [0.4, 1.0, 0.5, 1.0],
                life: 1.0,
            });
        }
    }

    fn area_damage(&mut self, center: Vec3, radius: f32, dmg: f32) {
        let mut killed: Vec<usize> = Vec::new();
        for i in 0..self.mobs.len() {
            let m = &mut self.mobs[i];
            if m.dead || m.kind == MobKind::Villager {
                continue;
            }
            if m.pos.distance(center) < radius {
                m.hp -= dmg;
                m.hurt_flash = 1.0;
                self.float_texts.push(FloatText {
                    pos: m.pos + Vec3::new(0.0, 1.8, 0.0),
                    text: format!("{}", dmg as i32),
                    color: [1.0, 0.82, 0.29, 1.0],
                    life: 0.8,
                });
                if m.hp <= 0.0 {
                    m.dead = true;
                    killed.push(i);
                }
            }
        }
        for i in killed {
            self.on_mob_death(i);
        }
        // le héros subit aussi les explosions
        if self.player.pos.distance(center) < radius + 0.5 && self.player.take_damage(dmg * 0.9) {
            self.damage_flash = 1.0;
        }
    }

    fn handle_mob_events(&mut self, events: Vec<MobEvent>) {
        for ev in events {
            match ev {
                MobEvent::ShootArrow { pos, dir } => {
                    self.projectiles.push(Projectile {
                        pos,
                        vel: dir * 16.0 + Vec3::new(0.0, 0.6, 0.0),
                        kind: ProjectileKind::ArrowEnemy,
                        life: 2.2,
                    });
                }
                MobEvent::Explode { pos } => {
                    self.particles.explosion(&mut self.rng, pos);
                    self.dynamic_lights.push((pos + Vec3::new(0.0, 1.0, 0.0), 9.0, Vec3::new(1.0, 0.5, 0.15), 1.6));
                    self.area_damage(pos, 3.6, 32.0);
                }
                MobEvent::HitPlayer { dmg } => {
                    if self.player.take_damage(dmg) {
                        self.damage_flash = 1.0;
                    }
                }
                MobEvent::CastOrbs { pos, base_yaw } => {
                    for i in 0..3 {
                        let a = base_yaw + (i as f32 - 1.0) * 0.5;
                        self.projectiles.push(Projectile {
                            pos: pos + Vec3::new(0.0, 1.3, 0.0),
                            vel: Vec3::new(a.cos(), 0.0, a.sin()) * 7.0,
                            kind: ProjectileKind::Orb,
                            life: 3.0,
                        });
                    }
                }
                MobEvent::SummonZombie { pos } => {
                    let zombies = self
                        .mobs
                        .iter()
                        .filter(|m| !m.dead && m.kind == MobKind::Zombie)
                        .count();
                    if zombies < 7 {
                        self.particles.purple_puff(&mut self.rng, pos + Vec3::new(0.0, 0.8, 0.0));
                        self.spawn_mob(MobKind::Zombie, pos, Some(false));
                    }
                }
            }
        }
    }

    fn update_projectile(&mut self, pr: &mut Projectile, dt: f32) -> bool {
        pr.life -= dt;
        if pr.life <= 0.0 {
            return false;
        }
        pr.pos += pr.vel * dt;
        match pr.kind {
            ProjectileKind::ArrowFriendly => {
                if self.rng.f32() < 0.4 {
                    self.particles.arrow_trail(pr.pos, true);
                }
                for i in 0..self.mobs.len() {
                    let m = &mut self.mobs[i];
                    if m.dead || m.kind == MobKind::Villager {
                        continue;
                    }
                    if (m.pos + Vec3::new(0.0, 1.0, 0.0)).distance(pr.pos) < 0.9 {
                        m.hp -= 14.0;
                        m.hurt_flash = 1.0;
                        self.float_texts.push(FloatText {
                            pos: m.pos + Vec3::new(0.0, 1.8, 0.0),
                            text: "14".to_string(),
                            color: [0.85, 0.95, 1.0, 1.0],
                            life: 0.8,
                        });
                        if m.hp <= 0.0 {
                            m.dead = true;
                            self.on_mob_death(i);
                        }
                        return false;
                    }
                }
            }
            ProjectileKind::ArrowEnemy => {
                if self.rng.f32() < 0.3 {
                    self.particles.arrow_trail(pr.pos, false);
                }
                if (self.player.pos + Vec3::new(0.0, 1.0, 0.0)).distance(pr.pos) < 0.85 {
                    if self.player.take_damage(6.0) {
                        self.damage_flash = 1.0;
                    }
                    return false;
                }
            }
            ProjectileKind::Orb => {
                self.particles.purple_puff(&mut self.rng, pr.pos);
                if self.player.pos.distance(pr.pos) < 1.1 {
                    if self.player.take_damage(10.0) {
                        self.damage_flash = 1.0;
                    }
                    let push = (self.player.pos - pr.pos).normalize_or_zero() * 3.0;
                    self.level.slide(&mut self.player.pos, push, 0.38);
                    return false;
                }
            }
        }
        true
    }

    fn update_pickup(&mut self, pk: &mut Pickup, dt: f32) -> bool {
        pk.bob += dt * 3.0;
        if pk.pos.distance(self.player.pos) < 1.4 {
            match pk.kind {
                PickupKind::Emerald => {
                    let n = self.rng.i64(1, 4) as u32;
                    self.player.emeralds += n;
                }
                PickupKind::Arrows => {
                    self.player.arrows += 5;
                }
            }
            self.particles.chest_sparkle(&mut self.rng, pk.pos);
            false
        } else {
            true
        }
    }

    fn update_interactions(&mut self, input: &Input) {
        let mut nearest: Option<usize> = None;
        let mut best_d = 2.4;
        for (i, it) in self.interactables.iter().enumerate() {
            if it.done {
                continue;
            }
            let d = it.pos.distance(self.player.pos);
            if d < best_d {
                best_d = d;
                nearest = Some(i);
            }
        }
        if let Some(i) = nearest {
            let crypt_open = self.crypt_open;
            let label = self.interactables[i].label(crypt_open).to_string();
            self.prompt = Some(format!("[F] {label}"));
            if input.just_pressed(KeyCode::KeyF) {
                let kind = self.interactables[i].kind;
                let pos = self.interactables[i].pos;
                self.interactables[i].done = true;
                match kind {
                    InteractKind::Cage(_) => {
                        self.float_texts.push(FloatText {
                            pos: pos + Vec3::new(0.0, 2.2, 0.0),
                            text: "Villageois libéré !".to_string(),
                            color: [0.5, 1.0, 0.6, 1.0],
                            life: 1.6,
                        });
                        self.particles.heal_green(pos);
                    }
                    InteractKind::Chest(_) => {
                        let em = self.rng.i64(8, 18) as u32;
                        self.player.emeralds += em;
                        self.player.arrows += 5;
                        self.player.add_xp(10);
                        for _ in 0..8 {
                            self.particles.chest_sparkle(&mut self.rng, pos);
                        }
                        self.float_texts.push(FloatText {
                            pos: pos + Vec3::new(0.0, 2.0, 0.0),
                            text: format!("Coffre : +{em} émeraudes, +5 flèches"),
                            color: [1.0, 0.9, 0.5, 1.0],
                            life: 2.0,
                        });
                    }
                    InteractKind::Merchant => {
                        self.player.emeralds += 15;
                        self.player.add_xp(8);
                        self.float_texts.push(FloatText {
                            pos: pos + Vec3::new(0.0, 2.2, 0.0),
                            text: "Marchand délivré : +15 émeraudes".to_string(),
                            color: [0.5, 1.0, 0.6, 1.0],
                            life: 2.0,
                        });
                    }
                    InteractKind::Rune => {
                        self.crypt_open = true;
                        self.float_texts.push(FloatText {
                            pos: pos + Vec3::new(0.0, 2.0, 0.0),
                            text: "Un passage s'ouvre… Creepy Crypt".to_string(),
                            color: [0.7, 0.5, 1.0, 1.0],
                            life: 2.6,
                        });
                        self.particles.teleport_poof(&mut self.rng, self.level.anchors.crypt_entry);
                        self.spawn_mob(MobKind::Zombie, self.level.anchors.crypt_entry + Vec3::new(1.5, 0.0, 0.0), Some(true));
                        self.spawn_mob(MobKind::Zombie, self.level.anchors.crypt_entry + Vec3::new(-1.5, 0.0, 0.0), Some(true));
                    }
                    InteractKind::CryptChest => {
                        self.player.emeralds += 25;
                        self.player.add_xp(20);
                        self.player.arrows += 10;
                        for _ in 0..12 {
                            self.particles.chest_sparkle(&mut self.rng, pos);
                        }
                        self.float_texts.push(FloatText {
                            pos: pos + Vec3::new(0.0, 2.0, 0.0),
                            text: "Trésor de la crypte ! +25 émeraudes".to_string(),
                            color: [1.0, 0.85, 0.4, 1.0],
                            life: 2.4,
                        });
                    }
                    InteractKind::Portal => {
                        if self.quest == Quest::Escape {
                            self.victory_time = self.mission_time;
                            self.screen = Screen::Victory;
                        } else {
                            self.interactables[i].done = false;
                            self.float_texts.push(FloatText {
                                pos: pos + Vec3::new(0.0, 2.4, 0.0),
                                text: "Le Nécromancien scelle le portail !".to_string(),
                                color: [1.0, 0.5, 0.4, 1.0],
                                life: 1.6,
                            });
                        }
                    }
                }
            }
        }
    }

    fn update_quests(&mut self) {
        match self.quest.clone() {
            Quest::ReachVillage => {
                if self.player.pos.distance(self.level.anchors.village) < 14.0 {
                    self.quest = Quest::FreeVillagers { freed: 0 };
                    self.checkpoint = self.level.anchors.village;
                }
            }
            Quest::FreeVillagers { freed } => {
                let now_freed = self
                    .interactables
                    .iter()
                    .filter(|i| matches!(i.kind, InteractKind::Cage(_)) && i.done)
                    .count();
                if now_freed >= 5 {
                    self.quest = Quest::FindCaravan;
                } else if now_freed != freed {
                    self.quest = Quest::FreeVillagers { freed: now_freed };
                }
            }
            Quest::FindCaravan => {
                if self.player.pos.distance(self.level.anchors.caravan) < 7.0 {
                    self.quest = Quest::Boss;
                    self.checkpoint = self.level.anchors.checkpoints[3];
                    self.spawn_boss();
                }
            }
            Quest::Boss => {
                if self.player.pos.distance(self.level.anchors.boss_arena) < 26.0 {
                    self.checkpoint = self.level.anchors.checkpoints[4];
                }
            }
            Quest::Escape => {}
        }
    }

    /// Invoque le boss (utilisé aussi par le mode QA).
    pub fn spawn_boss(&mut self) {
        let arena = self.level.anchors.boss_arena;
        self.boss_alive = true;
        let mut boss = Mob::new(MobKind::Necromancer, arena + Vec3::new(0.0, 0.0, -6.0), false);
        boss.state = MobState::Summon;
        boss.timer = 3.0;
        self.mobs.push(boss);
        for i in 0..3 {
            let a = i as f32 * 2.1;
            self.spawn_mob(MobKind::Zombie, arena + Vec3::new(a.cos() * 6.0, 0.0, a.sin() * 6.0), Some(i == 0));
        }
        self.float_texts.push(FloatText {
            pos: arena + Vec3::new(0.0, 4.0, 0.0),
            text: "Le Nécromancien surgit !".to_string(),
            color: [0.85, 0.4, 1.0, 1.0],
            life: 3.0,
        });
    }

    fn spawn_ambient(&mut self) {
        let p = self.player.pos;
        for _ in 0..8 {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let d = self.rng.range(15.0, 24.0);
            let pos = p + Vec3::new(a.cos() * d, 0.0, a.sin() * d);
            let (minx, maxx, minz, maxz) = self.level.bounds;
            if pos.x < minx + 2.0 || pos.x > maxx - 2.0 || pos.z < minz + 2.0 || pos.z > maxz - 2.0 {
                continue;
            }
            if self.level.blocked(pos.x, pos.z, 0.5) {
                continue;
            }
            let roll = self.rng.f32();
            let kind = if roll < 0.55 {
                MobKind::Zombie
            } else if roll < 0.85 {
                MobKind::Skeleton
            } else {
                MobKind::Creeper
            };
            // pas de creepers près du village
            let kind = if pos.distance(self.level.anchors.village) < 16.0 {
                MobKind::Zombie
            } else {
                kind
            };
            self.spawn_mob(kind, pos, None);
            return;
        }
    }

    // ==================== RENDU ====================

    /// Construit les vertex dynamiques du monde (héros, mobs, objets).
    pub fn build_dynamic(&self, out: &mut Vec<DynVertex>) {
        // Ombres portées (disques au sol)
        let shadow = |out: &mut Vec<DynVertex>, pos: Vec3, size: f32| {
            push_box(
                out,
                Mat4::from_translation(Vec3::new(pos.x, 0.03, pos.z)),
                Vec3::new(size, 0.01, size),
                [32; 6],
                Vec4::ONE,
                [0.0, 0.5, 0.0, 0.0],
            );
        };
        shadow(out, self.player.pos, 0.55);
        for m in &self.mobs {
            if m.fade > 0.0 {
                shadow(out, m.pos, m.radius() + 0.25);
            }
        }
        // Héros
        crate::player::build_mesh(&self.player, out);
        // Mobs
        for m in &self.mobs {
            mobs::build_mesh(m, out);
        }
        // Projectiles
        for pr in &self.projectiles {
            let (slot, tint) = match pr.kind {
                ProjectileKind::ArrowFriendly => (SLOT_GOLD, Vec4::new(1.3, 1.3, 1.4, 1.0)),
                ProjectileKind::ArrowEnemy => (SLOT_GOLD, Vec4::new(1.0, 0.8, 0.7, 1.0)),
                ProjectileKind::Orb => (SLOT_PORTAL, Vec4::new(1.8, 1.3, 2.4, 1.0)),
            };
            let yaw = pr.vel.z.atan2(pr.vel.x);
            let half = if matches!(pr.kind, ProjectileKind::Orb) {
                Vec3::splat(0.16)
            } else {
                Vec3::new(0.28, 0.04, 0.04)
            };
            push_box(
                out,
                Mat4::from_translation(pr.pos) * Mat4::from_rotation_y(-yaw),
                half,
                [slot; 6],
                tint,
                [1.0, 1.0, 0.0, 0.0],
            );
        }
        // Ramassages (petits cubes flottants)
        for pk in &self.pickups {
            let (slot, tint) = match pk.kind {
                PickupKind::Emerald => (SLOT_GOLD, Vec4::new(0.6, 2.0, 1.0, 1.0)),
                PickupKind::Arrows => (SLOT_GOLD, Vec4::new(0.8, 1.4, 2.0, 1.0)),
            };
            let y = 0.35 + (pk.bob * 0.15).sin() * 0.12;
            push_box(
                out,
                Mat4::from_translation(Vec3::new(pk.pos.x, y, pk.pos.z)) * Mat4::from_rotation_y(pk.bob),
                Vec3::splat(0.14),
                [slot; 6],
                tint,
                [1.0, 1.0, 0.0, 0.0],
            );
        }
        // Totem posé
        if let Some(tp) = self.player.totem_pos {
            push_box(
                out,
                Mat4::from_translation(tp + Vec3::new(0.0, 0.6, 0.0)),
                Vec3::new(0.16, 0.6, 0.16),
                [SLOT_GOLD; 6],
                Vec4::new(1.6, 1.3, 0.6, 1.0),
                [1.0, 1.0, 0.0, 0.0],
            );
        }
    }

    /// Lumières visibles (statiques les plus proches + dynamiques de la frame).
    pub fn collect_lights(&self) -> Vec<(Vec3, f32, Vec3, f32)> {
        let focus = self.cam.target;
        let mut all: Vec<(f32, (Vec3, f32, Vec3, f32))> = self
            .level
            .lights
            .iter()
            .map(|l| (l.0.distance_squared(focus), *l))
            .collect();
        for l in &self.dynamic_lights {
            all.push((l.0.distance_squared(focus) - 1000.0, *l)); // priorité aux dynamiques
        }
        all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        all.into_iter().take(16).map(|(_, l)| l).collect()
    }
}
