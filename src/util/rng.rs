//! Deterministic randomness. The same URL always yields the same Cypheron
//! scan, the same seed always yields the same radar traffic — so demos are
//! reproducible and screenshots match what visitors see.

/// 32-bit FNV-1a hash of a string.
pub fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for c in s.chars() {
        h ^= c as u32;
        h = h.wrapping_mul(16_777_619);
    }
    h
}

/// Mulberry32: tiny, fast, good-enough PRNG.
#[derive(Clone)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(seed)
    }

    /// Seed from the wall clock (non-deterministic).
    pub fn from_time() -> Self {
        Rng(js_sys::Date::now() as u64 as u32 ^ 0x9E37_79B9)
    }

    /// Uniform float in `[0, 1)`.
    pub fn f(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6D2B_79F5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64) / 4_294_967_296.0
    }

    /// Uniform float in `[a, b)`.
    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }

    /// Uniform integer in `[0, n)`.
    pub fn int(&mut self, n: usize) -> usize {
        ((self.f() * n as f64) as usize).min(n.saturating_sub(1))
    }

    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.int(xs.len())]
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.f() < p
    }
}
