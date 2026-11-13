//! Every shell command. Each returns an `Out`; streamed output (cargo,
//! ping…) is a list of `(delay_ms, line)` the terminal plays back.

use super::fs::{self, Body};
use super::markup::{md, plain, plain_cls, Line};
use crate::apps::{AppId, APPS};
use crate::data::{certs, packages, profile::{ME, TIMELINE, EDUCATION}, projects::{Project, PROJECTS}};
use crate::state::{Os, Phase, Theme, Wallpaper};
use crate::util::{dom, pad, rng::Rng};
use leptos::{SignalGetUntracked, SignalSet, SignalUpdate};

pub enum Out {
    Nothing,
    Lines(Vec<Line>),
    Stream(Vec<(u32, Line)>),
    Clear,
    Exit,
}

pub struct Shell {
    pub cwd: String,
    pub hist: Vec<String>,
}

pub const COMMANDS: &[(&str, &str)] = &[
    ("help", "this list"),
    ("whoami", "who is behind this OS"),
    ("neofetch", "system summary"),
    ("ls", "list files (-a for hidden)"),
    ("cd", "change directory"),
    ("cat", "print a file"),
    ("tree", "the whole filesystem"),
    ("projects", "list projects"),
    ("skills", "installed crates"),
    ("certs", "awards and certificates"),
    ("education", "academic record"),
    ("contact", "how to reach me"),
    ("resume", "plain-text résumé"),
    ("git", "try `git log`"),
    ("nmap", "scan this portfolio's services"),
    ("scan", "Cypheron demo scan: scan <url>"),
    ("cargo", "run · build · test · tree"),
    ("open", "open <app> — see `apps`"),
    ("apps", "list applications"),
    ("theme", "list or set a theme"),
    ("wallpaper", "list or set a wallpaper"),
    ("code", "open CodeLab"),
    ("top", "System Monitor"),
    ("breach", "play ICE Breaker"),
    ("fortune", "a random truth"),
    ("ferris", "the crab speaks"),
    ("ping", "ping a host"),
    ("crt", "toggle scanlines"),
    ("history", "previous commands"),
    ("echo", "print text"),
    ("date", "current date"),
    ("uname", "kernel info"),
    ("pwd", "current directory"),
    ("clear", "clear the screen (Ctrl+L)"),
    ("lock", "lock the screen"),
    ("reboot", "replay the boot"),
    ("exit", "close the terminal"),
    ("sudo", "…"),
];

pub fn project_text(p: &Project) -> String {
    let mut s = format!("[u]# {}[/]  [m]{} · {}[/]\n{}\n[m]stack:[/] {}", p.name, p.date, p.domain, p.tag, p.stack.join(", "));
    for pt in p.pts {
        s += &format!("\n  • {pt}");
    }
    if let Some(u) = p.live {
        s += &format!("\n[g]● live[/] <<{u}|{}>>", u.trim_start_matches("https://"));
    }
    if let Some(a) = p.app {
        s += &format!("\n{{open {a}|→ launch {} demo}}", p.name);
    }
    if let Some(b) = p.blueprint {
        s += &format!("  {{open blueprints {b}|→ blueprint}}");
    }
    s
}

pub fn resume() -> String {
    let mut s = format!("[u]{}[/]  [m]{} · {}[/]\n\n[y]## Education[/]", ME.name, ME.email, ME.loc);
    for e in EDUCATION {
        s += &format!("\n  {} · {} · {}", e.title, e.years, e.score);
    }
    s += "\n\n[y]## Projects[/]";
    for p in PROJECTS.iter().take(4) {
        s += &format!("\n  {} {}", pad(p.name, 10), p.tag);
    }
    s += "\n\n[y]## Achievements[/]\n  2nd Prize · AgentX 2026 Autonomous AI Agent Ideathon (ISTE TEC × NEAR AI)\n  Top 80 / 500+ · Quantum Arena 1.0 national hackathon (RapidCare)\n\nMore: {skills} {certs} {contact}";
    s
}

fn neofetch(os: &Os) -> Line {
    let mem = dom::wasm_memory_bytes() / 1048576.0;
    Line::Neofetch(vec![
        ("OS", "NishankOS 2026.3 (Kali-inspired)".into()),
        ("Kernel", "wasm32-unknown-unknown".into()),
        ("Runtime", "Rust · Leptos · wasm-bindgen".into()),
        ("User", ME.name.into()),
        ("Degree", "B.E. AI & Data Science · SIES GST".into()),
        ("CGPA", format!("{} / 10", ME.cgpa)),
        ("Uptime", os.uptime()),
        ("Memory", format!("{mem:.1} MiB wasm heap")),
        ("Theme", os.theme.get_untracked().name().into()),
        ("Focus", "sys-design · cyber-sec · web-dev · ml-dev".into()),
        ("Projects", "Cypheron, Fintrix, AvIAI, NishankOS".into()),
        ("Awards", "2nd Prize · AgentX 2026".into()),
    ])
}

fn ls(sh: &Shell, args: &[&str]) -> Out {
    let all = args.iter().any(|a| a.starts_with('-') && a.contains('a'));
    let target = args.iter().find(|a| !a.starts_with('-')).copied().unwrap_or(".");
    let dir = fs::resolve(&sh.cwd, target);
    let Some(items) = fs::list(&dir, all) else {
        return err(&format!("ls: cannot access '{target}': No such file or directory"));
    };
    let rel = if dir == "~" { String::new() } else { format!("{}/", dir.trim_start_matches("~/")) };
    let line = items
        .iter()
        .map(|x| {
            if x.ends_with('/') {
                format!("[b]{{cd {dir}/{x}|{x}}}[/]")
            } else {
                format!("{{cat ~/{rel}{x}|{x}}}")
            }
        })
        .collect::<Vec<_>>()
        .join("  ");
    Out::Lines(md(&line))
}

fn tree() -> Out {
    let mut s = String::from("[b]~[/]  [m](/home/nishank)[/]");
    let top = fs::list("~", true).unwrap_or_default();
    for (i, e) in top.iter().enumerate() {
        let last = i + 1 == top.len();
        s += &format!("\n{}{}", if last { "└── " } else { "├── " }, if e.ends_with('/') { format!("[b]{e}[/]") } else { e.clone() });
        if let Some(name) = e.strip_suffix('/') {
            let kids = fs::list(&format!("~/{name}"), false).unwrap_or_default();
            for (j, k) in kids.iter().enumerate() {
                s += &format!("\n{}{}{{cat ~/{name}/{k}|{k}}}", if last { "    " } else { "│   " }, if j + 1 == kids.len() { "└── " } else { "├── " });
            }
        }
    }
    Out::Lines(md(&s))
}

fn nmap() -> Out {
    let rows = [
        ("22/tcp", "resume", "cat resume.md", "plain-text résumé"),
        ("80/tcp", "web-dev", "open files", "React · FastAPI · Tailwind · Leptos"),
        ("443/tcp", "cyber-sec", "open cypheron", "Cypheron scanner"),
        ("1337/tcp", "ice-breaker", "open breach", "breach protocol (unlocks root)"),
        ("6379/tcp", "realtime", "open aviai", "Redis + WebSockets · AvIAI"),
        ("8000/tcp", "ml-dev", "open fintrix", "BM25 + trigram hybrid RAG"),
        ("8080/tcp", "sys-design", "open blueprints", "architecture blueprints"),
    ];
    let d = js_sys::Date::new_0();
    let mut v = vec![(120, md(&format!("Starting Nmap 7.94 ( simulated ) at {}", String::from(d.to_locale_string("en-IN", &wasm_bindgen::JsValue::UNDEFINED)))).remove(0))];
    v.push((380, md(&format!("Nmap scan report for {} (127.0.0.1)", ME.host)).remove(0)));
    v.push((90, plain("Host is up (0.00042s latency).")));
    v.push((260, plain_cls("PORT      STATE  SERVICE      VERSION", "t-m")));
    for (port, svc, cmd, ver) in rows {
        v.push((110, md(&format!("{} [g]open[/]   {{{cmd}|{}}} {ver}", pad(port, 9), pad(svc, 12))).remove(0)));
    }
    v.push((300, plain("")));
    v.push((60, md("Service detection performed. [g]7 services up[/], 0 vulnerabilities in the candidate.").remove(0)));
    Out::Stream(v)
}

fn cargo(args: &[&str]) -> Out {
    let sub = args.first().copied().unwrap_or("");
    let deps = ["proc-macro2", "unicode-ident", "quote", "syn", "wasm-bindgen-shared", "serde", "js-sys", "web-sys", "leptos_reactive", "leptos_dom", "leptos"];
    let compile = |v: &mut Vec<(u32, Line)>, release: bool| {
        for d in deps {
            v.push((70, md(&format!("   [g]Compiling[/] {d} v{}", packages::version(d))).remove(0)));
        }
        v.push((420, md("   [g]Compiling[/] nishank-os v2026.3.0 (/home/nishank/nishank-os)").remove(0)));
        v.push((900, md(&format!("    [g]Finished[/] `{}` profile [optimized] target(s) in 4.20s", if release { "release" } else { "dev" })).remove(0)));
    };
    let mut v = Vec::new();
    match sub {
        "run" => {
            compile(&mut v, true);
            v.push((200, md("     [g]Running[/] `target/wasm32-unknown-unknown/release/nishank-os.wasm`").remove(0)));
            v.push((500, md("[c]You're already running it.[/] Every pixel on this screen is that binary. 🦀").remove(0)));
        }
        "build" => compile(&mut v, args.contains(&"--release")),
        "test" => {
            compile(&mut v, false);
            v.push((200, md("     [g]Running[/] unittests src/main.rs").remove(0)));
            v.push((150, plain("")));
            v.push((120, plain("running 8 tests")));
            for t in [
                "nishank::cgpa_is_above_nine",
                "nishank::ships_real_projects",
                "nishank::wins_hackathons",
                "nishank::learns_fast",
                "nishank::writes_secure_code",
                "nishank::explains_clearly",
                "nishank::is_open_to_internships",
                "nishank::would_be_a_great_hire",
            ] {
                v.push((160, md(&format!("test {t} ... [g]ok[/]")).remove(0)));
            }
            v.push((200, plain("")));
            v.push((60, md("test result: [g]ok[/]. 8 passed; 0 failed; 0 ignored; finished in 0.00s").remove(0)));
            v.push((60, md("{sudo hire-me|→ next step: sudo hire-me}").remove(0)));
        }
        "tree" => {
            let mut s = String::from("nishank-os v2026.3.0 (/home/nishank/nishank-os)");
            for (i, sec) in packages::SECTIONS.iter().enumerate() {
                let last = i + 1 == packages::SECTIONS.len();
                s += &format!("\n{}[y]{sec}[/]", if last { "└── " } else { "├── " });
                let ps: Vec<_> = packages::PKGS.iter().filter(|p| p.1 == *sec).collect();
                for (j, p) in ps.iter().enumerate() {
                    s += &format!("\n{}{}{} v{}", if last { "    " } else { "│   " }, if j + 1 == ps.len() { "└── " } else { "├── " }, p.0, packages::version(p.0));
                }
            }
            s += "\n{open packages|→ open Cargo app}";
            return Out::Lines(md(&s));
        }
        _ => return Out::Lines(md("Rust's package manager\n\nUsage: cargo <command>\n  {cargo run|run}     build and run NishankOS\n  {cargo build --release|build}   compile only\n  {cargo test|test}    run the candidate's test suite\n  {cargo tree|tree}    skill dependency tree")),
    }
    Out::Stream(v)
}

fn git(args: &[&str]) -> Out {
    match args.first().copied() {
        Some("log") => {
            let oneline = args.contains(&"--oneline");
            let mut s = String::new();
            for (i, m) in TIMELINE.iter().enumerate() {
                let head = if i == 0 { " [c](HEAD -> main, origin/main)[/]" } else { "" };
                if oneline {
                    s += &format!("[y]{}[/]{head} {}({}): {}\n", m.hash, m.kind, m.scope, m.msg);
                } else {
                    s += &format!("[y]commit {}{}[/]{head}\nAuthor: {} <{}>\nDate:   {}\n\n    {}({}): {}\n\n", m.hash, "a3f9e1c7b2d40", ME.name, ME.email, m.date, m.kind, m.scope, m.msg);
                }
            }
            Out::Lines(md(s.trim_end()))
        }
        Some("status") => Out::Lines(md("On branch [g]main[/]\nYour branch is up to date with 'origin/main'.\n\nnothing to commit, working tree clean\n[m](but always something new to learn)[/]")),
        Some("blame") => Out::Lines(md("[y]f3a91c0[/] (Nishank Khadpe 2026-11) every line of this portfolio")),
        Some("remote") => Out::Lines(md(&format!("origin  <<{0}|{0}>> (fetch)\norigin  <<{0}|{0}>> (push)", ME.github))),
        _ => Out::Lines(md("usage: git <command>\n  {git log}  {git log --oneline}  {git status}  {git remote -v|git remote}  {git blame}")),
    }
}

fn ping(host: &str) -> Out {
    let host = if host.is_empty() { "nishank.dev" } else { host };
    let mut r = Rng::new(crate::util::rng::fnv1a(host));
    let ip = format!("{}.{}.{}.{}", 10 + r.int(200), r.int(255), r.int(255), 1 + r.int(254));
    let mut v = vec![(80, plain(format!("PING {host} ({ip}) 56(84) bytes of data.")))];
    let mut sum = 0.0;
    for i in 1..=4 {
        let t = r.range(8.0, 42.0);
        sum += t;
        v.push((700, plain(format!("64 bytes from {ip}: icmp_seq={i} ttl=57 time={t:.1} ms"))));
    }
    v.push((200, plain("")));
    v.push((40, plain(format!("--- {host} ping statistics ---"))));
    v.push((40, md(&format!("4 packets transmitted, 4 received, [g]0% packet loss[/], avg {:.1} ms", sum / 4.0)).remove(0)));
    Out::Stream(v)
}

const FORTUNES: &[&str] = &[
    "The best way to predict the future is to build it — preferably with tests.",
    "A security scanner that finds nothing has found something: a bug in itself.",
    "Retrieval first, generation second. Hallucinations hate citations.",
    "There are 10 kinds of people: those who read the RFC and those who get pwned.",
    "Fearless concurrency, careful curiosity.",
    "Ship it, measure it, then make it beautiful.",
    "Every CTF flag is a lesson somebody else learned the hard way.",
    "The quieter you become, the more you are able to hear.",
];

fn ferris_say(msg: &str) -> Out {
    let msg = if msg.is_empty() { "Hire Nishank. He rewrote his portfolio in Rust for fun." } else { msg };
    let w = msg.chars().count().min(56);
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in msg.split_whitespace() {
        if cur.chars().count() + word.chars().count() + 1 > w && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    lines.push(cur);
    let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
    let mut out = vec![plain(format!(" {}", "_".repeat(w + 2)))];
    for l in &lines {
        out.push(plain(format!("< {} >", pad(l, w))));
    }
    out.push(plain(format!(" {}", "-".repeat(w + 2))));
    for l in ["        \\", "         \\", "            _~^~^~_", "        \\) /  o o  \\ (/", "          '_   -   _'", "          / '-----' \\"] {
        out.push(plain_cls(l, "t-y"));
    }
    Out::Lines(out)
}

fn err(s: &str) -> Out {
    Out::Lines(vec![plain_cls(s, "t-r")])
}

/// Execute one command line.
pub fn run(sh: &mut Shell, os: Os, raw: &str) -> Out {
    let line = raw.trim();
    if line.is_empty() {
        return Out::Nothing;
    }
    sh.hist.push(line.to_string());
    let parts: Vec<&str> = line.split_whitespace().collect();
    let (cmd, args) = (parts[0].to_lowercase(), &parts[1..]);
    let rest = args.join(" ");

    match cmd.as_str() {
        "help" | "?" => {
            let mut s = String::from("[y]NishankOS zsh · available commands[/]\n");
            for chunk in COMMANDS.chunks(2) {
                let cell = |(c, d): &(&str, &str)| format!("{{{c}|{}}} [m]{}[/]", pad(c, 10), pad(d, 30));
                s += &format!("  {}\n", chunk.iter().map(cell).collect::<Vec<_>>().join(" "));
            }
            s += "[m]Tab completes · ↑/↓ history · Ctrl+L clears · Ctrl+C cancels · click anything underlined.[/]";
            Out::Lines(md(&s))
        }
        "whoami" => Out::Lines(md(&format!("{}\n[m]{} — {}[/]\n{{open about|→ open profile}}", ME.handle, ME.name, ME.role))),
        "id" => Out::Lines(md(&format!("uid=1000({0}) gid=1000({0}) groups=1000({0}),27(builders),1337(hackers)", ME.handle))),
        "neofetch" | "fastfetch" => Out::Lines(vec![neofetch(&os), plain(""), md("Type {help} or click any underlined command.").remove(0)]),
        "ls" | "ll" | "la" | "dir" => ls(sh, if cmd == "ll" || cmd == "la" { &["-a"] } else { args }),
        "cd" => {
            let d = fs::resolve(&sh.cwd, args.first().copied().unwrap_or("~"));
            if fs::is_dir(&d) {
                sh.cwd = d;
                Out::Nothing
            } else {
                err(&format!("cd: no such file or directory: {}", args.first().unwrap_or(&"")))
            }
        }
        "pwd" => Out::Lines(vec![plain(sh.cwd.replacen('~', "/home/nishank", 1))]),
        "cat" | "less" | "more" | "bat" => {
            let Some(a) = args.first() else { return err("cat: missing file operand") };
            let p = fs::resolve(&sh.cwd, a);
            match fs::read(&p) {
                Some(Body::Markup(s)) => {
                    if p.ends_with(".secrets") {
                        os.toast("flag", "Achievement unlocked", "Found ~/.secrets — curiosity is a feature.", None);
                    }
                    Out::Lines(md(&s))
                }
                Some(Body::Plain(s)) => Out::Lines(s.lines().map(|l| plain(l.to_string())).collect()),
                None if fs::is_dir(&p) => err(&format!("cat: {a}: Is a directory")),
                None => err(&format!("cat: {a}: No such file or directory")),
            }
        }
        "tree" => tree(),
        "projects" => {
            let mut s = String::new();
            for p in PROJECTS {
                let live = if p.live.is_some() { "[g]●[/]" } else { " " };
                s += &format!("{live} {{cat ~/projects/{}.md|{}}} [m]{}[/] {}\n", p.id, pad(p.name, 32), pad(p.date, 9), p.domain);
            }
            s += "[g]●[/] [m]= deployed live[/]\n{open files|→ open Projects}  {open blueprints|→ system blueprints}";
            Out::Lines(md(&s))
        }
        "skills" => {
            let mut s = String::new();
            for sec in packages::SECTIONS {
                let names: Vec<&str> = packages::PKGS.iter().filter(|p| p.1 == *sec).map(|p| p.0).collect();
                s += &format!("[y]{}[/] {}\n", pad(sec, 9), names.join(" "));
            }
            s += "{open packages|→ open Packages}  {cargo tree|→ cargo tree}";
            Out::Lines(md(&s))
        }
        "certs" => {
            let s = certs::CERTS
                .iter()
                .map(|c| {
                    let (mark, cls) = if c.kind == certs::Kind::Award { ("★", "y") } else { ("✓", "g") };
                    format!("  [{cls}]{mark}[/] {{cat ~/certificates/{}.cert|{}}} [m]· {}[/]", c.id, c.title, c.date)
                })
                .collect::<Vec<_>>()
                .join("\n");
            Out::Lines(md(&s))
        }
        "education" => {
            let s = EDUCATION.iter().map(|e| format!("  [d]{}[/]\n  {}  [m]{}[/]  [g]{}[/]", e.title, e.place, e.years, e.score)).collect::<Vec<_>>().join("\n");
            Out::Lines(md(&s))
        }
        "contact" => Out::Lines(md(&format!(
            "  email     <<mailto:{0}|{0}>>\n  phone     {1}\n  linkedin  <<{2}|{3}>>\n  github    <<{4}|{5}>>\n  status    [g]{6}[/]\n{{open contact|→ open Contact}}  {{download vcard|→ download vCard}}",
            ME.email, ME.phone, ME.linkedin, ME.linkedin.trim_start_matches("https://"), ME.github, ME.github.trim_start_matches("https://"), ME.status
        ))),
        "download" if args.first() == Some(&"vcard") => {
            dom::download("nishank-khadpe.vcf", "text/vcard", &crate::apps::contact_vcard());
            Out::Lines(md("[g]✓[/] contact.vcf downloaded"))
        }
        "resume" | "cv" => Out::Lines(md(&resume())),
        "apps" => {
            let s = APPS.iter().map(|a| format!("  {{open {}|{}}} [m]{}[/]", a.key, pad(a.key, 11), a.desc)).collect::<Vec<_>>().join("\n");
            Out::Lines(md(&s))
        }
        "open" | "xdg-open" | "launch" => match args.first().and_then(|k| AppId::from_key(k)) {
            Some(app) => {
                os.open(app, args.get(1).map(|s| s.to_string()));
                Out::Lines(vec![plain(format!("launching {}…", app.meta().key))])
            }
            None => Out::Lines(md(&format!("[r]open: unknown app '{}'.[/] Try {{apps}}", args.first().unwrap_or(&"")))),
        },
        "scan" => {
            let Some(u) = args.first() else { return Out::Lines(md("usage: scan <url>   e.g. {scan https://shop.example}")) };
            let u = if u.starts_with("http") { u.to_string() } else { format!("https://{u}") };
            os.open(AppId::Cypheron, Some(u.clone()));
            Out::Lines(md(&format!("[g][cypheron][/] scan queued for {u}")))
        }
        "nmap" => nmap(),
        "cargo" => cargo(args),
        "rustc" => Out::Lines(md("rustc 1.x (stable) · target wasm32-unknown-unknown\n[m]try[/] {cargo run}")),
        "git" => git(args),
        "code" | "leetcode" | "vim" | "nvim" | "hx" | "nano" => {
            os.open(AppId::CodeLab, args.first().map(|s| s.to_string()));
            Out::Lines(vec![plain("opening CodeLab… JavaScript editor, graded by Rust")])
        }
        "top" | "htop" | "btop" => {
            os.open(AppId::Monitor, None);
            Out::Lines(vec![plain("opening System Monitor…")])
        }
        "breach" | "hack" => {
            os.open(AppId::Breach, None);
            Out::Lines(md("[g]Ethical hacking only.[/] Launching ICE Breaker…"))
        }
        "theme" => match args.first() {
            None => {
                let cur = os.theme.get_untracked();
                let unlocked = os.root_unlocked.get_untracked();
                let s = Theme::ALL
                    .iter()
                    .map(|t| {
                        if *t == Theme::Root && !unlocked {
                            format!("  [m]{} ████ locked — beat {{breach|ICE Breaker}}[/]", pad(t.key(), 10))
                        } else {
                            format!("  {{theme {}|{}}} {}{}", t.key(), pad(t.key(), 10), t.name(), if *t == cur { " [g](current)[/]" } else { "" })
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                Out::Lines(md(&s))
            }
            Some(k) => match Theme::from_key(k) {
                Some(Theme::Root) if !os.root_unlocked.get_untracked() => err("theme: permission denied (uid 1000). Beat ICE Breaker first."),
                Some(t) => {
                    os.theme.set(t);
                    Out::Lines(md(&format!("theme set to [u]{}[/]", t.name())))
                }
                None => Out::Lines(md(&format!("[r]theme: unknown '{k}'.[/] Try {{theme}}"))),
            },
        },
        "wallpaper" => match args.first() {
            None => {
                let cur = os.wallpaper.get_untracked();
                let s = Wallpaper::BUILTIN
                    .iter()
                    .map(|w| format!("  {{wallpaper {}|{}}} {}{}", w.key(), pad(w.key(), 9), w.name(), if *w == cur { " [g](current)[/]" } else { "" }))
                    .collect::<Vec<_>>()
                    .join("\n");
                Out::Lines(md(&(s + "\n  {open settings|→ open Appearance} to upload your own")))
            }
            Some(k) => match Wallpaper::from_key(k) {
                Some(w) => {
                    os.wallpaper.set(w);
                    Out::Lines(md(&format!("wallpaper set to [u]{}[/]", w.name())))
                }
                None => Out::Lines(md(&format!("[r]wallpaper: unknown '{k}'.[/] Try {{wallpaper}}"))),
            },
        },
        "matrix" => {
            os.wallpaper.set(Wallpaper::Rain);
            Out::Lines(md("[g]Wake up, Neo…[/] (wallpaper set to Code Rain)"))
        }
        "crt" => {
            os.crt.update(|c| *c = !*c);
            Out::Lines(vec![plain(format!("scanlines {}", if os.crt.get_untracked() { "on" } else { "off" }))])
        }
        "fortune" => {
            let mut r = Rng::from_time();
            Out::Lines(md(&format!("[c]“{}”[/]", r.pick(FORTUNES))))
        }
        "ferris" | "cowsay" | "ferrissay" => ferris_say(&rest),
        "ping" => ping(args.first().copied().unwrap_or("")),
        "curl" | "wget" => Out::Lines(md("[r]curl: (7) outbound network is sandboxed in this OS.[/] Try {scan https://shop.example} instead.")),
        "uname" => Out::Lines(vec![plain(if args.contains(&"-a") {
            "NishankOS 2026.3 nishank-os 6.x-nishank #1 SMP wasm32-unknown-unknown Rust/Leptos"
        } else {
            "NishankOS"
        })]),
        "date" => Out::Lines(vec![plain(String::from(js_sys::Date::new_0().to_string()))]),
        "echo" => Out::Lines(vec![plain(rest)]),
        "history" => Out::Lines(sh.hist.iter().enumerate().map(|(i, h)| plain(format!("  {:>3}  {h}", i + 1))).collect()),
        "clear" | "cls" => Out::Clear,
        "lock" => {
            os.phase.set(Phase::Locked);
            Out::Nothing
        }
        "reboot" | "shutdown" | "poweroff" => {
            os.reboot();
            Out::Nothing
        }
        "exit" | "logout" => Out::Exit,
        "sudo" | "please" | "doas" => {
            if rest.contains("rm -rf") {
                err("Nice try. This portfolio is immutable — it's compiled.")
            } else if rest.starts_with("hire") {
                os.open(AppId::Contact, Some("hire".into()));
                Out::Stream(vec![
                    (100, plain(format!("[sudo] password for {}: ", ME.handle))),
                    (700, md("[g]✓ privilege granted.[/] Escalating to Contact…").remove(0)),
                    (200, md(&format!("[m]Drafting an email to {}. Be kind — he reads everything.[/]", ME.email)).remove(0)),
                ])
            } else {
                Out::Lines(md(&format!("[r]{} is not in the sudoers file.[/] This incident will be reported.\n[m](Hire him and he might be.)[/]  {{sudo hire-me}}", ME.handle)))
            }
        }
        "rm" | "mv" | "touch" | "mkdir" | "chmod" => err(&format!("{cmd}: permission denied — read-only portfolio filesystem.")),
        "exit()" | "quit" | ":q" | ":wq" => Out::Lines(md("[m]You're in zsh, not vim. But nice reflexes.[/] Type {exit}.")),
        "make" => Out::Lines(md("make: *** No rule to make target 'coffee'. Stop.\n[m]try[/] {cargo build --release|cargo build}")),
        "hello" | "hi" | "hey" => Out::Lines(md(&format!("Hey! I'm {}'s shell. Start with {{help}}, {{whoami}} or {{projects}}.", ME.name.split(' ').next().unwrap_or("")))),
        _ => {
            // Suggest the closest known command.
            let best = COMMANDS
                .iter()
                .filter_map(|(c, _)| crate::util::fuzzy::score(&cmd, c).map(|s| (s, *c)))
                .max_by_key(|p| p.0)
                .map(|p| p.1);
            let hint = best.map(|b| format!(" — did you mean {{{b}}}?")).unwrap_or_else(|| " — try {help}".into());
            Out::Lines(md(&format!("[r]zsh: command not found: {cmd}[/]{hint}")))
        }
    }
}

/// Candidates for Tab completion of the last word.
pub fn complete(sh: &Shell, input: &str) -> Vec<String> {
    let parts: Vec<&str> = input.split(' ').collect();
    let last = *parts.last().unwrap_or(&"");
    if parts.len() == 1 {
        return COMMANDS.iter().map(|c| c.0.to_string()).filter(|c| c.starts_with(last)).collect();
    }
    match parts[0] {
        "open" => APPS.iter().map(|a| a.key.to_string()).filter(|k| k.starts_with(last)).collect(),
        "theme" => Theme::ALL.iter().map(|t| t.key().to_string()).filter(|k| k.starts_with(last)).collect(),
        "wallpaper" => Wallpaper::BUILTIN.iter().map(|w| w.key().to_string()).filter(|k| k.starts_with(last)).collect(),
        "cargo" => ["run", "build", "test", "tree"].iter().map(|s| s.to_string()).filter(|k| k.starts_with(last)).collect(),
        "git" => ["log", "status", "remote", "blame"].iter().map(|s| s.to_string()).filter(|k| k.starts_with(last)).collect(),
        _ => {
            let (dir_part, file_part) = last.rsplit_once('/').map(|(d, f)| (format!("{d}/"), f)).unwrap_or((String::new(), last));
            let dir = fs::resolve(&sh.cwd, if dir_part.is_empty() { "." } else { &dir_part });
            fs::list(&dir, file_part.starts_with('.'))
                .unwrap_or_default()
                .into_iter()
                .filter(|e| e.starts_with(file_part))
                .map(|e| format!("{dir_part}{e}"))
                .collect()
        }
    }
}
