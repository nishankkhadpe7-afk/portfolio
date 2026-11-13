# NishankOS — Architecture

A tour of how a desktop operating system fits in one Rust crate.

## 1. The big picture

```
                    ┌────────────────────────── browser ──────────────────────────┐
 cargo + trunk      │                                                              │
 ───────────────►   │  nishank_os_bg.wasm ──► Leptos runtime ──► DOM (surgical      │
 src/**/*.rs        │        │                    │              updates only)     │
                    │        │                    ├──► Canvas 2D (wallpapers,      │
                    │        │                    │     radar) via rAF loops        │
                    │        │                    └──► Web Worker (CodeLab sandbox) │
                    │        └──► localStorage (theme, wallpaper, solved, drafts)   │
                    └──────────────────────────────────────────────────────────────┘
```

There is no virtual DOM. Leptos signals are subscribed to by exactly the text nodes and
attributes that read them, so dragging a window rewrites four inline style properties per
frame and nothing else.

## 2. The kernel — `state.rs`

`Os` is a `Copy` bundle of signals provided as context at the root. Every component calls
`os()` to reach it.

```
Os
├── phase        Boot | Desktop | Locked
├── wins         Vec<Win>          (each Win is itself a bundle of signals)
├── focused      Option<AppId>
├── theme, wallpaper, custom_wall, crt, ferris, motion, root_unlocked
├── menu, palette, expose, ctx     overlay visibility
├── snap_hint    where a dragged window would snap
├── desk         desktop size in CSS px
└── toasts
```

`Win { app, x, y, w, h, z, min, snap, arg }` — because every field is a signal, a `Win` is
`Copy` and can be captured by any closure. `arg` is the deep-link channel: the palette,
terminal and other apps write to it (`open cypheron https://…`, `open certs agentx`) and the
target app consumes it in an effect, then resets it to `None`.

Window operations (`open`, `focus`, `close`, `minimize`, `toggle_max`, `task_click`) are
methods on `Os`, so the panel, palette, terminal and System Monitor all drive windows
through one API.

Preferences are mirrored to `localStorage` by effects created in `Os::new`, and the theme is
applied by setting `<body class="theme-…">`.

## 3. Boot → desktop ⇄ lock — `app.rs`

```
          any key / click              lock button, `lock`
 Boot ───────────────────► Desktop ◄──────────────────────► Locked
   ▲                          │       any key / click
   └──────── reboot ──────────┘
```

The desktop stays mounted under the lock screen, so open windows keep their state.
`app.rs` also owns the global hotkeys and the Konami-code detector.

## 4. The window frame — `os/window.rs`

- **Drag**: `pointerdown` on the title bar records the start, then window-level
  `pointermove`/`pointerup` listeners (stored in a `StoredValue` and removed on release) update
  `x`/`y`. Grabbing a snapped window un-snaps it with the cursor at the same relative spot.
- **Snap**: within 4 px of the top/left/right edge, `snap_hint` is set; the desktop renders a
  ghost; on release the hint becomes `win.snap`. Snapping is pure CSS (`.max`, `.snap-l`,
  `.snap-r`), so restoring returns the exact previous geometry.
- **Resize**: eight invisible grips; the direction string (`"nw"`, `"e"`…) decides which of
  `x/y/w/h` change, clamped to a minimum size and the desktop bounds.

## 5. Animation — `util/dom.rs`

`use_raf(f)` starts a `requestAnimationFrame` loop whose closure is owned by an `Rc`; when
the owning component unmounts, `on_cleanup` flips an `alive` flag and the closure is dropped
on the next macrotask (never while executing). `dt` is clamped so a backgrounded tab doesn't
teleport anything. `use_interval` does the same for timers.

## 6. Wallpapers — `os/wallpaper/`

`WallCanvas` owns a canvas and a `Scene` enum (`Network`, `Rain`, `Horizon`, `Aurora`,
`Life`, `Midnight`). Each frame it:

1. fits the backing store to CSS size × DPR (capped at 2, or 1 for thumbnails),
2. rebuilds the scene if the wallpaper, size or theme changed,
3. calls `scene.frame(&Frame { cx, w, h, t, dt, acc, ice, bg, pointer })`.

Thumbnails in Appearance are the same component with `preview = true` (30 fps, no pointer).
With *Motion* off, a scene renders one frame and then idles. The desktop instance also
measures frame rate for the System Monitor.

## 7. Apps

| Module | Notes |
|---|---|
| `terminal/markup.rs` | Output is `Vec<Line>`; a mini-markup (`[y]…[/]`, `{cmd\|label}`, `<<url\|label>>`) keeps commands readable. User input is never parsed. |
| `terminal/commands.rs` | Each command returns `Out::{Lines, Stream, Clear, Exit}`; streams are `(delay_ms, Line)` lists played back with cancellable timeouts (`Ctrl C`). |
| `terminal/fs.rs` | Read-only tree rooted at `~`; `~/src` embeds real files with `include_str!`. |
| `fintrix/retrieval.rs` | Tokenise → light stemming → BM25 (k1 = 1.4, b = 0.75) fused with trigram cosine (α = 0.55). An answer is *grounded* only if BM25 > 0.6, fused > 0.5 **and** IDF-weighted query coverage ≥ 42 %. Unit-tested. |
| `codelab/runner.rs` | Blob → Worker per run; JSON in/out; 4 s deadline then `terminate()`. |
| `codelab/highlight.rs` | Hand-written JS tokenizer → escaped HTML overlay. |
| `data/problems.rs` | Each problem carries a Rust `reference` fn; the judge compares visitor output against it (with an order-insensitive mode for pairs). |
| `aviai.rs` | Planes in `Rc<RefCell<Vec<Plane>>>` shared by the rAF loop, the clearance timer and the click picker; strips are a signal refreshed every 1.5 s. |
| `breach.rs` | Generates a valid 7-step path first, then cuts the three daemons from it. |

## 8. Styles — `styles/`

| File | Concern |
|---|---|
| `00-tokens.css` | colours, fonts, easing; each theme is a body class re-pointing the variables. `--acc-rgb` / `--ice-rgb` let translucent tints follow the theme. |
| `01-base.css` | reset, icon primitive, scrollbars, splash |
| `02-boot-lock.css` | boot log, progress bar, lock screen |
| `03-shell.css` | panel, desktop, icons, base window, menu, toasts |
| `04-wm.css` | window states, resize grips, snap ghost |
| `05-components.css` | shared primitives: buttons, chips, severities, split layouts |
| `06-overlays.css` | palette, overview, toast life bar |
| `07-ferris-crt.css` | the crab, scanlines |
| `apps/*.css` | one file per app |
| `98-responsive.css` | ≤ 760 px: windows go full-screen, grids collapse |

Trunk concatenates them in the order listed in `index.html`.

## 9. What is simulated

Being honest matters in a portfolio: Cypheron's findings, AvIAI's traffic and the System
Monitor's CPU numbers are simulations (clearly labelled in each app). Fintrix retrieval,
CodeLab judging, the wasm-memory/DOM/fps readings, the filesystem's `~/src`, and everything
the window manager does are real.
