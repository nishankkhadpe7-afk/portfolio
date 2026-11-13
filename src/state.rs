//! The kernel. One `Os` value — a bundle of signals — is provided as context
//! at the root and every component reaches the window manager through it.

use crate::apps::AppId;
use crate::util::{dom, store};
use leptos::*;

pub const PANEL_H: f64 = 36.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Boot,
    Desktop,
    Locked,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Snap {
    Max,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Kali,
    Phosphor,
    Amber,
    Synthwave,
    Root,
}

impl Theme {
    pub const ALL: [Theme; 5] = [Theme::Kali, Theme::Phosphor, Theme::Amber, Theme::Synthwave, Theme::Root];
    pub fn key(self) -> &'static str {
        match self {
            Theme::Kali => "kali",
            Theme::Phosphor => "phosphor",
            Theme::Amber => "amber",
            Theme::Synthwave => "synthwave",
            Theme::Root => "root",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Theme::Kali => "Kali Deep Blue",
            Theme::Phosphor => "Phosphor",
            Theme::Amber => "Amber CRT",
            Theme::Synthwave => "Synthwave",
            Theme::Root => "uid=0 (root)",
        }
    }
    pub fn from_key(k: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.key() == k)
    }
    /// Accent and highlight colours, for canvas code that cannot read CSS.
    pub fn rgb(self) -> ((u8, u8, u8), (u8, u8, u8)) {
        match self {
            Theme::Kali => ((61, 139, 255), (114, 214, 255)),
            Theme::Phosphor => ((43, 217, 107), (141, 255, 180)),
            Theme::Amber => ((255, 166, 43), (255, 211, 138)),
            Theme::Synthwave => ((255, 79, 216), (79, 240, 255)),
            Theme::Root => ((255, 59, 85), (255, 154, 168)),
        }
    }
    pub fn swatch(self) -> [&'static str; 3] {
        match self {
            Theme::Kali => ["#040a15", "#3d8bff", "#72d6ff"],
            Theme::Phosphor => ["#020a05", "#2bd96b", "#8dffb4"],
            Theme::Amber => ["#0d0700", "#ffa62b", "#ffd38a"],
            Theme::Synthwave => ["#0b0418", "#ff4fd8", "#4ff0ff"],
            Theme::Root => ["#0d0204", "#ff3b55", "#ff9aa8"],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wallpaper {
    Network,
    Rain,
    Horizon,
    Aurora,
    Life,
    Midnight,
    Custom,
}

impl Wallpaper {
    pub const BUILTIN: [Wallpaper; 6] = [
        Wallpaper::Network,
        Wallpaper::Rain,
        Wallpaper::Horizon,
        Wallpaper::Aurora,
        Wallpaper::Life,
        Wallpaper::Midnight,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Wallpaper::Network => "network",
            Wallpaper::Rain => "rain",
            Wallpaper::Horizon => "horizon",
            Wallpaper::Aurora => "aurora",
            Wallpaper::Life => "life",
            Wallpaper::Midnight => "midnight",
            Wallpaper::Custom => "custom",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Wallpaper::Network => "Neural Network",
            Wallpaper::Rain => "Code Rain",
            Wallpaper::Horizon => "Horizon Grid",
            Wallpaper::Aurora => "Aurora",
            Wallpaper::Life => "Game of Life",
            Wallpaper::Midnight => "Midnight",
            Wallpaper::Custom => "Your image",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Wallpaper::Network => "nodes that reach for your cursor",
            Wallpaper::Rain => "glyphs falling in columns",
            Wallpaper::Horizon => "an endless synth grid at dusk",
            Wallpaper::Aurora => "slow ribbons of light",
            Wallpaper::Life => "Conway's automaton — draw with the mouse",
            Wallpaper::Midnight => "still, quiet, battery-friendly",
            Wallpaper::Custom => "uploaded from your device",
        }
    }
    pub fn from_key(k: &str) -> Option<Wallpaper> {
        Wallpaper::BUILTIN.into_iter().find(|w| w.key() == k)
    }
}

/// One open window. All fields are signals, so the struct is `Copy` and can
/// be handed to any closure without cloning ceremony.
#[derive(Clone, Copy)]
pub struct Win {
    pub app: AppId,
    pub x: RwSignal<f64>,
    pub y: RwSignal<f64>,
    pub w: RwSignal<f64>,
    pub h: RwSignal<f64>,
    pub z: RwSignal<u32>,
    pub min: RwSignal<bool>,
    pub snap: RwSignal<Option<Snap>>,
    /// Deep-link argument (a URL to scan, a problem id, a command to run…).
    /// Apps watch it and reset it to `None` once consumed.
    pub arg: RwSignal<Option<String>>,
}

#[derive(Clone)]
pub struct Toast {
    pub id: u32,
    pub icon: &'static str,
    pub title: String,
    pub body: String,
    pub open: Option<AppId>,
}

#[derive(Clone, Copy)]
pub struct Os {
    pub phase: RwSignal<Phase>,
    pub wins: RwSignal<Vec<Win>>,
    pub focused: RwSignal<Option<AppId>>,
    zc: StoredValue<u32>,
    pub toasts: RwSignal<Vec<Toast>>,
    toast_n: StoredValue<u32>,
    pub theme: RwSignal<Theme>,
    pub wallpaper: RwSignal<Wallpaper>,
    pub custom_wall: RwSignal<Option<String>>,
    pub crt: RwSignal<bool>,
    pub ferris: RwSignal<bool>,
    pub motion: RwSignal<bool>,
    pub root_unlocked: RwSignal<bool>,
    pub menu: RwSignal<bool>,
    pub palette: RwSignal<bool>,
    pub expose: RwSignal<bool>,
    pub ctx: RwSignal<Option<(f64, f64)>>,
    pub snap_hint: RwSignal<Option<Snap>>,
    pub desk: RwSignal<(f64, f64)>,
    pub boot_at: f64,
}

pub fn os() -> Os {
    expect_context::<Os>()
}

impl Os {
    /// Create the kernel, restoring persisted preferences.
    pub fn new() -> Self {
        let root_unlocked = store::get_bool("root", false);
        let theme = store::get("theme")
            .and_then(|k| Theme::from_key(&k))
            .filter(|t| *t != Theme::Root || root_unlocked)
            .unwrap_or(Theme::Kali);
        let wallpaper = store::get("wall").and_then(|k| Wallpaper::from_key(&k)).unwrap_or(Wallpaper::Network);
        let skip_boot = store::get_bool("booted", false) && dom::prefers_reduced_motion();
        let os = Os {
            phase: create_rw_signal(if skip_boot { Phase::Desktop } else { Phase::Boot }),
            wins: create_rw_signal(Vec::new()),
            focused: create_rw_signal(None),
            zc: store_value(20),
            toasts: create_rw_signal(Vec::new()),
            toast_n: store_value(0),
            theme: create_rw_signal(theme),
            wallpaper: create_rw_signal(wallpaper),
            custom_wall: create_rw_signal(None),
            crt: create_rw_signal(store::get_bool("crt", false)),
            ferris: create_rw_signal(store::get_bool("ferris", true)),
            motion: create_rw_signal(store::get_bool("motion", !dom::prefers_reduced_motion())),
            root_unlocked: create_rw_signal(root_unlocked),
            menu: create_rw_signal(false),
            palette: create_rw_signal(false),
            expose: create_rw_signal(false),
            ctx: create_rw_signal(None),
            snap_hint: create_rw_signal(None),
            desk: create_rw_signal((1280.0, 760.0)),
            boot_at: dom::now(),
        };
        os.persist();
        os
    }

    /// Mirror preferences into localStorage and the theme onto <body>.
    fn persist(&self) {
        let s = *self;
        create_effect(move |_| {
            let t = s.theme.get();
            store::set("theme", t.key());
            let body = document().body().expect("body");
            body.set_class_name(&format!("theme-{}", t.key()));
        });
        create_effect(move |_| {
            let w = s.wallpaper.get();
            if w != Wallpaper::Custom {
                store::set("wall", w.key());
            }
        });
        create_effect(move |_| store::set_bool("crt", s.crt.get()));
        create_effect(move |_| store::set_bool("ferris", s.ferris.get()));
        create_effect(move |_| store::set_bool("motion", s.motion.get()));
        create_effect(move |_| store::set_bool("root", s.root_unlocked.get()));
    }

    fn win(&self, app: AppId) -> Option<Win> {
        self.wins.with_untracked(|v| v.iter().find(|w| w.app == app).copied())
    }

    fn next_z(&self) -> u32 {
        self.zc.update_value(|z| *z += 1);
        self.zc.get_value()
    }

    /// Open (or raise) an application, optionally passing it an argument.
    pub fn open(&self, app: AppId, arg: Option<String>) {
        self.menu.set(false);
        self.palette.set(false);
        self.expose.set(false);
        self.ctx.set(None);
        if let Some(w) = self.win(app) {
            if arg.is_some() {
                w.arg.set(arg);
            }
            w.min.set(false);
            self.focus(app);
            return;
        }
        let meta = app.meta();
        let (dw, dh) = self.desk.get_untracked();
        let n = self.wins.with_untracked(|v| v.len()) as f64;
        let w = meta.w.min(dw - 24.0).max(320.0);
        let h = meta.h.min(dh - 24.0).max(240.0);
        // Cascade new windows from a spot that leaves the desktop icons visible.
        let x = ((dw - w) / 2.0 + 90.0 + (n % 6.0) * 28.0 - 70.0).clamp(8.0, (dw - w - 8.0).max(8.0));
        let y = ((dh - h) / 2.0 + (n % 6.0) * 26.0 - 40.0).clamp(8.0, (dh - h - 8.0).max(8.0));
        let win = Win {
            app,
            x: create_rw_signal(x),
            y: create_rw_signal(y),
            w: create_rw_signal(w),
            h: create_rw_signal(h),
            z: create_rw_signal(self.next_z()),
            min: create_rw_signal(false),
            snap: create_rw_signal(if dom::is_mobile() { Some(Snap::Max) } else { None }),
            arg: create_rw_signal(arg),
        };
        self.wins.update(|v| v.push(win));
        self.focused.set(Some(app));
    }

    pub fn focus(&self, app: AppId) {
        if let Some(w) = self.win(app) {
            if self.focused.get_untracked() != Some(app) || w.z.get_untracked() < self.zc.get_value() {
                w.z.set(self.next_z());
            }
            self.focused.set(Some(app));
        }
    }

    pub fn close(&self, app: AppId) {
        self.wins.update(|v| v.retain(|w| w.app != app));
        if self.focused.get_untracked() == Some(app) {
            self.focus_topmost();
        }
    }

    pub fn minimize(&self, app: AppId) {
        if let Some(w) = self.win(app) {
            w.min.set(true);
            if self.focused.get_untracked() == Some(app) {
                self.focus_topmost();
            }
        }
    }

    pub fn toggle_max(&self, app: AppId) {
        if let Some(w) = self.win(app) {
            w.snap.update(|s| *s = if s.is_some() { None } else { Some(Snap::Max) });
        }
    }

    /// Task-bar click semantics: focus if hidden or behind, minimise if on top.
    pub fn task_click(&self, app: AppId) {
        if let Some(w) = self.win(app) {
            if w.min.get_untracked() {
                w.min.set(false);
                self.focus(app);
            } else if self.focused.get_untracked() == Some(app) {
                self.minimize(app);
            } else {
                self.focus(app);
            }
        }
    }

    fn focus_topmost(&self) {
        let top = self.wins.with_untracked(|v| {
            v.iter().filter(|w| !w.min.get_untracked()).max_by_key(|w| w.z.get_untracked()).map(|w| w.app)
        });
        self.focused.set(top);
    }

    pub fn close_focused(&self) {
        if let Some(a) = self.focused.get_untracked() {
            self.close(a);
        }
    }

    pub fn toast(&self, icon: &'static str, title: impl Into<String>, body: impl Into<String>, open: Option<AppId>) {
        self.toast_n.update_value(|n| *n += 1);
        let id = self.toast_n.get_value();
        self.toasts.update(|t| {
            t.push(Toast { id, icon, title: title.into(), body: body.into(), open });
            if t.len() > 4 {
                t.remove(0);
            }
        });
        let toasts = self.toasts;
        set_timeout(move || toasts.update(|t| t.retain(|x| x.id != id)), std::time::Duration::from_millis(5600));
    }

    pub fn dismiss_toast(&self, id: u32) {
        self.toasts.update(|t| t.retain(|x| x.id != id));
    }

    pub fn unlock_root(&self) {
        if !self.root_unlocked.get_untracked() {
            self.root_unlocked.set(true);
            self.toast("spark", "Root theme unlocked", "uid=0(root) gid=0(root). Find it in Appearance.", Some(AppId::Settings));
        }
    }

    pub fn cycle_theme(&self) {
        let unlocked = self.root_unlocked.get_untracked();
        let all: Vec<Theme> = Theme::ALL.into_iter().filter(|t| *t != Theme::Root || unlocked).collect();
        let cur = self.theme.get_untracked();
        let i = all.iter().position(|t| *t == cur).unwrap_or(0);
        self.theme.set(all[(i + 1) % all.len()]);
    }

    pub fn cycle_wallpaper(&self) {
        let cur = self.wallpaper.get_untracked();
        let all = Wallpaper::BUILTIN;
        let i = all.iter().position(|w| *w == cur).map(|i| i + 1).unwrap_or(0);
        self.wallpaper.set(all[i % all.len()]);
    }

    pub fn uptime(&self) -> String {
        let s = ((dom::now() - self.boot_at) / 1000.0) as u64;
        format!("{}m {:02}s", s / 60, s % 60)
    }

    pub fn reboot(&self) {
        self.wins.set(Vec::new());
        self.focused.set(None);
        self.menu.set(false);
        self.phase.set(Phase::Boot);
    }
}
