//! Six procedural scenes. Each is plain Rust state plus a `frame` method
//! that paints one frame onto a 2D context — no allocation per frame.

use crate::state::Wallpaper;
use crate::util::rng::Rng;
use std::f64::consts::{PI, TAU};
use web_sys::CanvasRenderingContext2d as Cx;

pub struct Frame<'a> {
    pub cx: &'a Cx,
    pub w: f64,
    pub h: f64,
    /// Seconds since the canvas mounted.
    pub t: f64,
    /// Seconds since the previous frame (0 when motion is off).
    pub dt: f64,
    pub acc: (u8, u8, u8),
    pub ice: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    /// Pointer in canvas-local CSS pixels, if this canvas tracks it.
    pub pointer: Option<(f64, f64)>,
}

fn rgba((r, g, b): (u8, u8, u8), a: f64) -> String {
    format!("rgba({r},{g},{b},{a:.3})")
}

pub enum Scene {
    Network(Network),
    Rain(Rain),
    Horizon(Horizon),
    Aurora(Aurora),
    Life(Life),
    Midnight(Midnight),
}

impl Scene {
    pub fn new(kind: Wallpaper, w: f64, h: f64, preview: bool) -> Scene {
        match kind {
            Wallpaper::Network | Wallpaper::Custom => Scene::Network(Network::new(w, h)),
            Wallpaper::Rain => Scene::Rain(Rain::new(w, h, preview)),
            Wallpaper::Horizon => Scene::Horizon(Horizon::new(w, h)),
            Wallpaper::Aurora => Scene::Aurora(Aurora::new(w, h)),
            Wallpaper::Life => Scene::Life(Life::new(w, h, preview)),
            Wallpaper::Midnight => Scene::Midnight(Midnight::new(w, h)),
        }
    }

    pub fn frame(&mut self, f: &Frame) {
        match self {
            Scene::Network(s) => s.frame(f),
            Scene::Rain(s) => s.frame(f),
            Scene::Horizon(s) => s.frame(f),
            Scene::Aurora(s) => s.frame(f),
            Scene::Life(s) => s.frame(f),
            Scene::Midnight(s) => s.frame(f),
        }
    }
}

// ───────────────────────── Neural Network ─────────────────────────
// Drifting nodes, linked when close; nodes near the cursor reach for it.

struct Node {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    r: f64,
}

pub struct Network {
    nodes: Vec<Node>,
}

impl Network {
    fn new(w: f64, h: f64) -> Self {
        let mut r = Rng::new(7);
        let n = (w * h / 15000.0).clamp(14.0, 95.0) as usize;
        let nodes = (0..n)
            .map(|_| Node { x: r.f() * w, y: r.f() * h, vx: r.range(-0.2, 0.2), vy: r.range(-0.2, 0.2), r: r.range(0.6, 2.2) })
            .collect();
        Network { nodes }
    }

    fn frame(&mut self, f: &Frame) {
        let (cx, w, h) = (f.cx, f.w, f.h);
        cx.clear_rect(0.0, 0.0, w, h);
        let k = f.dt * 60.0;
        let link = (w.min(h) * 0.17).clamp(70.0, 140.0);
        for n in &mut self.nodes {
            if let Some((px, py)) = f.pointer {
                let (dx, dy) = (px - n.x, py - n.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d < 180.0 && d > 1.0 {
                    n.vx += dx / d * 0.012 * k;
                    n.vy += dy / d * 0.012 * k;
                }
            }
            n.vx = n.vx.clamp(-0.6, 0.6) * (1.0 - 0.004 * k);
            n.vy = n.vy.clamp(-0.6, 0.6) * (1.0 - 0.004 * k);
            n.x += n.vx * k;
            n.y += n.vy * k;
            if n.x < 0.0 || n.x > w {
                n.vx = -n.vx;
                n.x = n.x.clamp(0.0, w);
            }
            if n.y < 0.0 || n.y > h {
                n.vy = -n.vy;
                n.y = n.y.clamp(0.0, h);
            }
        }
        cx.set_line_width(1.0);
        for i in 0..self.nodes.len() {
            for j in (i + 1)..self.nodes.len() {
                let (a, b) = (&self.nodes[i], &self.nodes[j]);
                let d = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
                if d < link {
                    cx.set_stroke_style_str(&rgba(f.acc, (1.0 - d / link) * 0.35));
                    cx.begin_path();
                    cx.move_to(a.x, a.y);
                    cx.line_to(b.x, b.y);
                    cx.stroke();
                }
            }
        }
        if let Some((px, py)) = f.pointer {
            for n in &self.nodes {
                let d = ((n.x - px).powi(2) + (n.y - py).powi(2)).sqrt();
                if d < 180.0 {
                    cx.set_stroke_style_str(&rgba(f.ice, (1.0 - d / 180.0) * 0.5));
                    cx.begin_path();
                    cx.move_to(n.x, n.y);
                    cx.line_to(px, py);
                    cx.stroke();
                }
            }
        }
        for n in &self.nodes {
            cx.set_fill_style_str(&rgba(f.ice, 0.8));
            cx.begin_path();
            let _ = cx.arc(n.x, n.y, n.r, 0.0, TAU);
            cx.fill();
        }
    }
}

// ───────────────────────── Code Rain ─────────────────────────

const GLYPHS: &[char] = &[
    '0', '1', '{', '}', '[', ']', '<', '>', '/', '=', ';', ':', '$', '#', 'A', 'B', 'C', 'D', 'E', 'F', '7', '9', 'ア', 'イ',
    'ウ', 'エ', 'カ', 'キ', 'ク', 'ケ', 'ｱ', 'ｲ', 'λ', '∑', '⊕', '&', '*', 'r', 's',
];

pub struct Rain {
    fs: f64,
    drops: Vec<f64>,
    speed: Vec<f64>,
    rng: Rng,
    first: bool,
}

impl Rain {
    fn new(w: f64, h: f64, preview: bool) -> Self {
        let fs = if preview || w < 420.0 { 8.0 } else { 15.0 };
        let cols = (w / fs).ceil() as usize;
        let mut rng = Rng::new(11);
        let drops = (0..cols).map(|_| -rng.f() * h / fs).collect();
        let speed = (0..cols).map(|_| rng.range(0.35, 1.0)).collect();
        Rain { fs, drops, speed, rng, first: true }
    }

    fn frame(&mut self, f: &Frame) {
        let cx = f.cx;
        if self.first {
            cx.set_fill_style_str(&rgba(f.bg, 1.0));
            cx.fill_rect(0.0, 0.0, f.w, f.h);
            self.first = false;
        }
        if f.dt == 0.0 {
            return;
        }
        cx.set_fill_style_str(&rgba(f.bg, 0.09));
        cx.fill_rect(0.0, 0.0, f.w, f.h);
        cx.set_font(&format!("{}px 'IBM Plex Mono', monospace", self.fs));
        let rows = f.h / self.fs;
        for i in 0..self.drops.len() {
            let before = self.drops[i].floor();
            self.drops[i] += self.speed[i] * f.dt * 18.0;
            if self.drops[i].floor() == before {
                continue;
            }
            let y = self.drops[i].floor() * self.fs;
            let ch = *self.rng.pick(GLYPHS);
            let head = self.rng.chance(0.12);
            cx.set_fill_style_str(&if head { rgba(f.ice, 0.95) } else { rgba(f.acc, 0.75) });
            let _ = cx.fill_text(&ch.to_string(), i as f64 * self.fs, y);
            if self.drops[i] > rows && self.rng.chance(0.025) {
                self.drops[i] = 0.0;
            }
        }
    }
}

// ───────────────────────── Horizon Grid ─────────────────────────

pub struct Horizon {
    stars: Vec<(f64, f64, f64)>,
    hills: Vec<f64>,
}

impl Horizon {
    fn new(w: f64, h: f64) -> Self {
        let mut r = Rng::new(3);
        let stars = (0..90).map(|_| (r.f() * w, r.f() * h * 0.55, r.f())).collect();
        // Two octaves of value noise make a believable ridge line.
        let coarse: Vec<f64> = (0..12).map(|_| r.f()).collect();
        let hills = (0..=64)
            .map(|i| {
                let x = i as f64 / 64.0 * 11.0;
                let (a, b) = (coarse[x as usize], coarse[(x as usize + 1).min(11)]);
                let t = x.fract();
                let s = t * t * (3.0 - 2.0 * t);
                a + (b - a) * s + (i as f64 * 1.7).sin() * 0.08
            })
            .collect();
        Horizon { stars, hills }
    }

    fn frame(&mut self, f: &Frame) {
        let (cx, w, h, t) = (f.cx, f.w, f.h, f.t);
        let hz = h * 0.58;
        // sky
        let g = cx.create_linear_gradient(0.0, 0.0, 0.0, hz);
        let _ = g.add_color_stop(0.0, &rgba(f.bg, 1.0));
        let _ = g.add_color_stop(1.0, &rgba(f.acc, 0.28));
        cx.set_fill_style_canvas_gradient(&g);
        cx.fill_rect(0.0, 0.0, w, hz);
        for &(x, y, s) in &self.stars {
            let tw = 0.35 + 0.65 * ((t * (0.6 + s) + x).sin() * 0.5 + 0.5);
            cx.set_fill_style_str(&rgba(f.ice, tw * 0.7 * s));
            cx.fill_rect(x, y, 1.4, 1.4);
        }
        // sun with scan-line cut-outs
        let (sx, sy, sr) = (w * 0.5, hz - h * 0.02, h.min(w) * 0.17);
        let sg = cx.create_linear_gradient(0.0, sy - sr, 0.0, sy + sr);
        let _ = sg.add_color_stop(0.0, &rgba(f.ice, 0.95));
        let _ = sg.add_color_stop(1.0, &rgba(f.acc, 0.85));
        cx.save();
        cx.begin_path();
        let _ = cx.arc(sx, sy, sr, PI, TAU);
        cx.clip();
        cx.set_fill_style_canvas_gradient(&sg);
        cx.fill_rect(sx - sr, sy - sr, sr * 2.0, sr);
        cx.set_fill_style_str(&rgba(f.bg, 1.0));
        for i in 0..7 {
            let yy = sy - sr * 0.08 - i as f64 * sr * 0.12;
            cx.fill_rect(sx - sr, yy, sr * 2.0, 1.0 + (6 - i) as f64 * 0.7);
        }
        cx.restore();
        // ridge
        cx.set_fill_style_str(&rgba(f.bg, 0.92));
        cx.begin_path();
        cx.move_to(0.0, hz);
        for (i, v) in self.hills.iter().enumerate() {
            cx.line_to(i as f64 / 64.0 * w, hz - v * h * 0.09 - 4.0);
        }
        cx.line_to(w, hz);
        cx.close_path();
        cx.fill();
        // ground + perspective grid
        let gg = cx.create_linear_gradient(0.0, hz, 0.0, h);
        let _ = gg.add_color_stop(0.0, &rgba(f.acc, 0.18));
        let _ = gg.add_color_stop(1.0, &rgba(f.bg, 1.0));
        cx.set_fill_style_canvas_gradient(&gg);
        cx.fill_rect(0.0, hz, w, h - hz);
        cx.set_line_width(1.0);
        let n = 18;
        for i in 0..n {
            let z = ((i as f64 + t * 0.9) % n as f64) / n as f64;
            let y = hz + (h - hz) * z.powf(2.4);
            cx.set_stroke_style_str(&rgba(f.acc, 0.15 + z * 0.6));
            cx.begin_path();
            cx.move_to(0.0, y);
            cx.line_to(w, y);
            cx.stroke();
        }
        for i in -14..=14 {
            let xb = w * 0.5 + i as f64 * w * 0.09;
            cx.set_stroke_style_str(&rgba(f.acc, 0.35));
            cx.begin_path();
            cx.move_to(w * 0.5 + i as f64 * 6.0, hz);
            cx.line_to(xb * 1.0 + (xb - w * 0.5) * 1.6, h);
            cx.stroke();
        }
    }
}

// ───────────────────────── Aurora ─────────────────────────

pub struct Aurora {
    stars: Vec<(f64, f64, f64)>,
}

impl Aurora {
    fn new(w: f64, h: f64) -> Self {
        let mut r = Rng::new(5);
        Aurora { stars: (0..120).map(|_| (r.f() * w, r.f() * h, r.f())).collect() }
    }

    fn frame(&mut self, f: &Frame) {
        let (cx, w, h, t) = (f.cx, f.w, f.h, f.t * 0.35 + 3.0);
        cx.set_fill_style_str(&rgba(f.bg, 1.0));
        cx.fill_rect(0.0, 0.0, w, h);
        for &(x, y, s) in &self.stars {
            cx.set_fill_style_str(&rgba((220, 232, 248), 0.15 + 0.45 * s));
            cx.fill_rect(x, y, 1.2, 1.2);
        }
        let _ = cx.set_global_composite_operation("lighter");
        let bands = [(0.30, 0.9, f.acc, 0.16), (0.42, 1.3, f.ice, 0.12), (0.55, 0.7, f.acc, 0.10)];
        for (bi, &(base, freq, col, a)) in bands.iter().enumerate() {
            for layer in 0..5 {
                let off = layer as f64 * h * 0.025;
                cx.begin_path();
                let steps = 48;
                for s in 0..=steps {
                    let x = s as f64 / steps as f64 * w;
                    let u = x / w;
                    let y = h * base
                        + (u * freq * TAU + t * (0.8 + bi as f64 * 0.3)).sin() * h * 0.06
                        + (u * freq * 2.7 * TAU - t * 1.1).sin() * h * 0.025
                        + off;
                    if s == 0 {
                        cx.move_to(x, y);
                    } else {
                        cx.line_to(x, y);
                    }
                }
                cx.line_to(w, h * base + h * 0.22 + off);
                cx.line_to(0.0, h * base + h * 0.22 + off);
                cx.close_path();
                let g = cx.create_linear_gradient(0.0, h * (base - 0.08), 0.0, h * (base + 0.24));
                let _ = g.add_color_stop(0.0, &rgba(col, 0.0));
                let _ = g.add_color_stop(0.35, &rgba(col, a * (1.0 - layer as f64 * 0.15)));
                let _ = g.add_color_stop(1.0, &rgba(col, 0.0));
                cx.set_fill_style_canvas_gradient(&g);
                cx.fill();
            }
        }
        let _ = cx.set_global_composite_operation("source-over");
    }
}

// ───────────────────────── Game of Life ─────────────────────────
// Conway's B3/S23 on a torus. Cells fade out as they die, the cursor
// seeds new life, and the board reseeds itself when it stagnates.

pub struct Life {
    cell: f64,
    cols: usize,
    rows: usize,
    grid: Vec<u8>,
    ghost: Vec<f32>,
    acc: f64,
    rng: Rng,
    history: [u64; 6],
    gen: u64,
}

impl Life {
    fn new(w: f64, h: f64, preview: bool) -> Self {
        let cell = if preview { 5.0 } else { 13.0 };
        let (cols, rows) = ((w / cell).ceil() as usize + 1, (h / cell).ceil() as usize + 1);
        let mut l = Life {
            cell,
            cols,
            rows,
            grid: vec![0; cols * rows],
            ghost: vec![0.0; cols * rows],
            acc: 0.0,
            rng: Rng::from_time(),
            history: [0; 6],
            gen: 0,
        };
        l.seed();
        l
    }

    fn seed(&mut self) {
        for c in &mut self.grid {
            *c = u8::from(self.rng.chance(0.18));
        }
    }

    fn step(&mut self) {
        let (c, r) = (self.cols, self.rows);
        let mut next = vec![0u8; c * r];
        let mut pop = 0usize;
        let mut hash: u64 = 1469598103934665603;
        for y in 0..r {
            for x in 0..c {
                let mut n = 0;
                for (dx, dy) in [(c - 1, r - 1), (0, r - 1), (1, r - 1), (c - 1, 0), (1, 0), (c - 1, 1), (0, 1), (1, 1)] {
                    n += self.grid[((y + dy) % r) * c + (x + dx) % c];
                }
                let i = y * c + x;
                let alive = matches!((self.grid[i], n), (1, 2) | (1, 3) | (0, 3));
                if alive {
                    next[i] = 1;
                    pop += 1;
                    hash = (hash ^ i as u64).wrapping_mul(1099511628211);
                } else if self.grid[i] == 1 {
                    self.ghost[i] = 1.0;
                }
            }
        }
        self.grid = next;
        self.gen += 1;
        // Reseed if the board died out or fell into a short cycle.
        let cycling = self.history.contains(&hash);
        self.history[(self.gen % 6) as usize] = hash;
        if pop < c * r / 40 || (cycling && self.gen.is_multiple_of(60)) {
            self.seed();
        }
    }

    fn frame(&mut self, f: &Frame) {
        if let Some((px, py)) = f.pointer {
            if px >= 0.0 && py >= 0.0 && px < f.w && py < f.h {
                let (gx, gy) = ((px / self.cell) as usize, (py / self.cell) as usize);
                if gx < self.cols && gy < self.rows {
                    self.grid[gy * self.cols + gx] = 1;
                    let nx = (gx + 1) % self.cols;
                    self.grid[gy * self.cols + nx] = 1;
                }
            }
        }
        self.acc += f.dt;
        if self.acc > 0.11 {
            self.acc = 0.0;
            self.step();
        }
        let cx = f.cx;
        cx.clear_rect(0.0, 0.0, f.w, f.h);
        let s = self.cell - 2.0;
        for y in 0..self.rows {
            for x in 0..self.cols {
                let i = y * self.cols + x;
                let (px, py) = (x as f64 * self.cell, y as f64 * self.cell);
                if self.grid[i] == 1 {
                    cx.set_fill_style_str(&rgba(f.acc, 0.55));
                    cx.fill_rect(px, py, s, s);
                } else if self.ghost[i] > 0.02 {
                    cx.set_fill_style_str(&rgba(f.ice, self.ghost[i] as f64 * 0.25));
                    cx.fill_rect(px, py, s, s);
                    self.ghost[i] *= (1.0 - f.dt * 2.2).max(0.0) as f32;
                }
            }
        }
    }
}

// ───────────────────────── Midnight ─────────────────────────

pub struct Midnight {
    stars: Vec<(f64, f64, f64)>,
    drawn: bool,
}

impl Midnight {
    fn new(w: f64, h: f64) -> Self {
        let mut r = Rng::new(9);
        Midnight { stars: (0..160).map(|_| (r.f() * w, r.f() * h, r.f())).collect(), drawn: false }
    }

    fn frame(&mut self, f: &Frame) {
        if self.drawn {
            return; // a still image: paint once, then let the GPU idle
        }
        self.drawn = true;
        let cx = f.cx;
        let g = cx.create_radial_gradient(f.w * 0.7, f.h * 0.25, 0.0, f.w * 0.7, f.h * 0.25, f.w.max(f.h)).unwrap();
        let _ = g.add_color_stop(0.0, &rgba(f.acc, 0.22));
        let _ = g.add_color_stop(0.5, &rgba(f.bg, 1.0));
        let _ = g.add_color_stop(1.0, &rgba(f.bg, 1.0));
        cx.set_fill_style_canvas_gradient(&g);
        cx.fill_rect(0.0, 0.0, f.w, f.h);
        for &(x, y, s) in &self.stars {
            cx.set_fill_style_str(&rgba(f.ice, 0.1 + s * 0.5));
            cx.begin_path();
            let _ = cx.arc(x, y, 0.4 + s, 0.0, TAU);
            cx.fill();
        }
    }
}
