//! Caméra 3/4 façon Minecraft Dungeons : surélevée, suit le joueur en douceur.

use glam::{Mat4, Vec3};

pub const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

#[derive(Clone, Copy)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub fovy: f32,
    pub aspect: f32,
    pub offset: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            eye: Vec3::ZERO,
            target: Vec3::ZERO,
            fovy: 45.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            offset: Vec3::new(0.0, 13.5, 9.5),
        }
    }

    /// Suit doucement une position (lindépendant du framerate).
    pub fn follow(&mut self, player: Vec3, combat_zoom: f32, dt: f32) {
        let want_target = player + Vec3::new(0.0, 1.0, 0.0);
        let want_eye = want_target + self.offset * combat_zoom;
        let k = 1.0 - (-8.0 * dt).exp();
        self.target = self.target.lerp(want_target, k);
        self.eye = self.eye.lerp(want_eye, k);
    }

    /// Téléporte la caméra (changement d'écran).
    pub fn snap(&mut self, player: Vec3) {
        self.target = player + Vec3::new(0.0, 1.0, 0.0);
        self.eye = self.target + self.offset;
    }

    /// Recadre l'aspect quand la fenêtre change.
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn view_proj(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, 0.1, 500.0)
            * Mat4::look_at_rh(self.eye, self.target, UP)
    }

}
