//! Générateur pseudo-aléatoire déterministe (xorshift) + bruit, sans dépendance externe.

#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// [0, 1)
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() % 1_000_000) as f32 / 1_000_000.0
    }

    /// [a, b)
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + self.f32() * (b - a)
    }

    pub fn i64(&mut self, a: i64, b: i64) -> i64 {
        a + (self.next_u64() % (b - a).max(1) as u64) as i64
    }

    /// Valeur de bruit lisse 2D dans [0, 1], déterministe (hash de cellule + interpolation).
    pub fn noise2(&mut self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i64;
        let yi = y.floor() as i64;
        let fx = x - xi as f32;
        let fy = y - yi as f32;
        let h = |cx: i64, cy: i64| -> f32 {
            let mut n = (cx.wrapping_mul(374761393)).wrapping_add(cy.wrapping_mul(668265263));
            n ^= n >> 13;
            n = n.wrapping_mul(1274126177);
            // Réduction sur 32 bits AVANT conversion f32 : un i64 -> f32 donnerait des
            // valeurs ~1e18 et un tint GPU explosé (sol blanc en jeu).
            let t = (n ^ (n >> 16)) as u32;
            t as f32 / (u32::MAX as f32)
        };
        let a = h(xi, yi);
        let b = h(xi + 1, yi);
        let c = h(xi, yi + 1);
        let d = h(xi + 1, yi + 1);
        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sy = fy * fy * (3.0 - 2.0 * fy);
        a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
    }
}

