//! Entrées clavier/souris via scancodes physiques (AZERTY-safe).

use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event::{ElementState, MouseButton};

pub struct Input {
    keys: HashSet<KeyCode>,
    prev_keys: HashSet<KeyCode>,
    mouse: [bool; 2],
    prev_mouse: [bool; 2],
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys: HashSet::new(),
            prev_keys: HashSet::new(),
            mouse: [false, false],
            prev_mouse: [false, false],
        }
    }

    pub fn key_event(&mut self, state: ElementState, physical: PhysicalKey) {
        if let PhysicalKey::Code(code) = physical {
            match state {
                ElementState::Pressed => self.keys.insert(code),
                ElementState::Released => self.keys.remove(&code),
            };
        }
    }

    pub fn mouse_event(&mut self, state: ElementState, button: MouseButton) {
        let idx = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            _ => return,
        };
        self.mouse[idx] = state == ElementState::Pressed;
    }

    pub fn pressed(&self, code: KeyCode) -> bool {
        self.keys.contains(&code)
    }

    pub fn just_pressed(&self, code: KeyCode) -> bool {
        self.keys.contains(&code) && !self.prev_keys.contains(&code)
    }

    pub fn mouse_just_pressed(&self, left: bool) -> bool {
        let i = usize::from(!left);
        self.mouse[i] && !self.prev_mouse[i]
    }


    pub fn end_frame(&mut self) {
        self.prev_keys = self.keys.clone();
        self.prev_mouse = self.mouse;
    }
}

/// Directions de déplacement (scancodes, indépendant du layout AZERTY/QWERTY).
pub fn move_axis(input: &Input) -> glam::Vec2 {
    let mut v = glam::Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) {
        v.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        v.y += 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        v.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        v.x += 1.0;
    }
    v.normalize_or_zero()
}
