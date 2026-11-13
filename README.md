<div align="center">

# ⬡ NishankOS

**Nishank Khadpe's portfolio, as a Kali-inspired desktop operating system — written in Rust, compiled to WebAssembly.**

`Rust` · `Leptos` · `wasm-bindgen` · `Canvas 2D` · `Web Workers` · `Trunk`

</div>

---

## What it is

Open the page and a computer boots. There's a panel, a desktop, draggable windows and a shell.
Every app is a way to explore Nishank's work — and every part of it, from the window manager to
the radar physics to the search engine, is Rust running in your browser.

| App | What it does | What's real |
|---|---|---|
| **Terminal** | zsh-flavoured shell over a virtual filesystem — `help`, `neofetch`, `git log`, `cargo test`, `nmap`, `sudo hire-me` | `~/src` contains this OS's *actual* source, embedded at compile time |
| **whoami** | profile, skill radar, journey timeline, education | radar geometry computed in Rust from typed data |
| **Projects** | file-explorer view of everything shipped | — |
| **Cypheron** | simulated web-security scan with grading, AI-style triage and a downloadable report | deterministic per URL (FNV-seeded), real Markdown export |
| **AvIAI Radar** | 60 NM approach radar with sweep, trails, conflict alerts and AI clearances | kinematics + STCA conflict detection in Rust, painted every frame |
| **Fintrix** | compliance Q&A over RBI/FEMA with a live pipeline view and What-If panel | **real BM25 + trigram-cosine hybrid retrieval**, with an IDF-weighted grounding guard |
| **Blueprints** | system-design diagrams for each project, hover to trace connections | Bézier edge routing generated from node/edge data |
| **CodeLab** | LeetCode-style JavaScript practice | code runs in a sandboxed Worker; expected answers come from **Rust reference solutions** |
| **System Monitor** | htop for the portfolio; kill a process to close its window | wasm heap, DOM nodes, fps, uptime are live measurements |
| **ICE Breaker** | breach-protocol puzzle | generated from a hidden solution path — always solvable |
| **Packages** | skills as a `cargo`-style crate table | — |
| **Certificates** | scanned + generated certificates, verification links | — |
| **Contact** | copyable details, message composer → your mail app, vCard download | nothing leaves the page |
| **Appearance** | 5 themes, 6 live wallpapers (each thumbnail is a running scene), effects | persisted to `localStorage` |

### Shell chrome

- **Window manager** — drag, 8-way resize, double-click to maximise, **edge snapping** (top = maximise, sides = tile) with a live preview, minimise animations, z-ordering.
- **Command palette** (`Ctrl K`) — one fuzzy search over apps, projects, blueprints, certificates, problems, themes, wallpapers, shell commands and actions.
- **Overview** (`F3`) — every open window as a schematic card at its real relative position.
- **Wallpapers** — Neural Network (reaches for your cursor), Code Rain, Horizon Grid, Aurora, **Game of Life** (draw cells with the mouse), Midnight.
- **Ferris** — a hand-drawn crab who patrols the desktop and gives tips when clicked.
- Boot sequence with live wasm memory readout, lock screen, toasts, context menu, CRT scanlines.

### Keyboard

| Keys | Action |
|---|---|
| `Ctrl K` / `Ctrl Space` | command palette |
| `Ctrl Alt T` | terminal |
| `F3` / `Ctrl ↑` | overview |
| `Alt W` | close focused window |
| ``Alt ` `` | cycle windows |
| `Esc` | close overlays |
| `Ctrl Enter` / `Ctrl Shift Enter` | CodeLab run / submit |

<details><summary>Easter eggs (spoilers)</summary>

- `ls -a` in the terminal, then read what you find.
- Win ICE Breaker with the `ROOT_SHELL` daemon — or type ↑ ↑ ↓ ↓ ← → ← → B A — to unlock the **root** theme.
- `cargo test`, `sudo hire-me`, `ferris <message>`, `matrix`, `git blame`, `:q`.
</details>

---

## Run it

```bash
# one-time setup
rustup target add wasm32-unknown-unknown
cargo install trunk --locked  # or: cargo binstall trunk

# develop (live reload on http://127.0.0.1:8080)
trunk serve

# production build → ./dist
trunk build --release

# unit tests (run natively — no browser needed)
cargo test
```

### Deploy

- **Vercel** — `vercel.json` + `scripts/vercel-build.sh` install Rust and Trunk on the build machine and publish `dist/`. Import the repo and deploy; no settings to change.
- **GitHub Pages** — `.github/workflows/ci.yml` tests, lints, builds with the right `--public-url`, and deploys on every push to `main` (enable *Pages → Source: GitHub Actions* once).
- **Anything static** — upload `dist/`. Serve `.wasm` as `application/wasm`.

---

## Project layout

```
nishank-os/
├── Cargo.toml              crate + size-optimised release profile
├── Trunk.toml              bundler config
├── index.html              Trunk entry: stylesheets, the rust crate, assets
├── assets/                 favicon, scanned certificate
├── styles/                 one concern per file (see ARCHITECTURE.md)
│   ├── 00-tokens.css       design tokens + the 5 themes
│   ├── 01-base.css … 07-ferris-crt.css
│   ├── apps/*.css          one stylesheet per app
│   └── 98-responsive.css
├── src/
│   ├── main.rs             entry point
│   ├── app.rs              root: boot → desktop ⇄ lock, global hotkeys
│   ├── state.rs            the "kernel": windows, z-order, theme, toasts
│   ├── os/                 shell chrome
│   │   ├── boot.rs  lock.rs  panel.rs  desktop.rs  window.rs
│   │   ├── menu.rs  palette.rs  context_menu.rs  expose.rs  toasts.rs
│   │   ├── ferris.rs  icons.rs
│   │   └── wallpaper/      engine (mod.rs) + six scenes (scenes.rs)
│   ├── apps/               one module per application
│   │   ├── mod.rs          registry: AppId, metadata, render switch
│   │   ├── terminal/       mod.rs · commands.rs · fs.rs · markup.rs
│   │   ├── fintrix/        mod.rs · retrieval.rs (BM25 + trigram)
│   │   ├── codelab/        mod.rs · runner.rs (Worker) · highlight.rs
│   │   └── about · projects · cypheron · aviai · blueprints · packages
│   │       certs · contact · settings · monitor · breach
│   ├── data/               every fact about Nishank, as typed consts
│   └── util/               rng · dom glue · fuzzy matcher · storage
├── scripts/vercel-build.sh
├── .github/workflows/ci.yml
└── legacy/index.html       the previous single-file HTML version
```

**Editing content?** Everything lives in `src/data/`. Change a project there and the
Projects app, terminal, palette and neofetch all update — and a typo in a field name is a
compile error, not a broken page.

**Adding an app?** Add a variant to `AppId`, a row in `APPS`, a module, and an arm in
`apps::render`. It automatically gets a desktop icon, a launcher entry, a palette entry,
`open <key>` in the shell and a process in the System Monitor.

See **[ARCHITECTURE.md](ARCHITECTURE.md)** for how the pieces fit together.

---

<sub>Built by Nishank Khadpe · B.E. AI & Data Science, SIES GST · khadpenishank@gmail.com</sub>
