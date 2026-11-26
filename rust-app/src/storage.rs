//! Storage analyser: multi-threaded folder size scan + installed application sizes.

use crate::util::{powershell, run};
use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Default)]
pub struct Scan {
    pub root: PathBuf,
    pub entries: Vec<Entry>,
    /// Immutable snapshot handed to the render thread; cloning the `Arc` is O(1),
    /// so the UI never deep-clones the file list while painting.
    pub view: Arc<Vec<Entry>>,
    pub running: bool,
    pub done: usize,
    pub total: usize,
}

pub struct ScanHandle {
    pub state: Arc<Mutex<Scan>>,
    cancel: Arc<AtomicBool>,
}

impl ScanHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Measure every direct child of `root` in parallel (each directory is walked recursively).
pub fn start(root: PathBuf, ctx: egui::Context) -> ScanHandle {
    let state = Arc::new(Mutex::new(Scan { root: root.clone(), running: true, ..Default::default() }));
    let cancel = Arc::new(AtomicBool::new(false));
    let (st, cn) = (state.clone(), cancel.clone());

    thread::spawn(move || {
        let children: Vec<(String, PathBuf, bool)> = match std::fs::read_dir(&root) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| {
                    let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    (e.file_name().to_string_lossy().into_owned(), e.path(), is_dir)
                })
                .collect(),
            Err(_) => vec![],
        };
        st.lock().unwrap().total = children.len();

        let next = AtomicUsize::new(0);
        let workers = thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);
        thread::scope(|sc| {
            for _ in 0..workers {
                sc.spawn(|| loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= children.len() || cn.load(Ordering::Relaxed) {
                        break;
                    }
                    let (name, path, is_dir) = &children[i];
                    let size = if *is_dir { dir_size(path, &cn) } else { path.symlink_metadata().map(|m| m.len()).unwrap_or(0) };
                    {
                        let mut s = st.lock().unwrap();
                        s.entries.push(Entry { name: name.clone(), path: path.clone(), size, is_dir: *is_dir });
                        s.done += 1;
                        // Publish in batches: sorting/cloning on every single completion
                        // would become O(n²) on directories with many children.
                        if s.done % 16 == 0 || s.done == s.total {
                            s.entries.sort_unstable_by(|a, b| b.size.cmp(&a.size));
                            s.view = Arc::new(s.entries.clone());
                            drop(s);
                            ctx.request_repaint();
                        }
                    }
                });
            }
        });
        {
            let mut s = st.lock().unwrap();
            s.entries.sort_unstable_by(|a, b| b.size.cmp(&a.size));
            s.view = Arc::new(s.entries.clone());
            s.running = false;
        }
        ctx.request_repaint();
    });

    ScanHandle { state, cancel }
}

fn dir_size(p: &Path, cancel: &AtomicBool) -> u64 {
    let mut total = 0u64;
    for e in WalkDir::new(p).follow_links(false).same_file_system(true).into_iter().filter_map(|e| e.ok()) {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        if e.file_type().is_file() {
            total += e.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    total
}

/// Installed applications and their (estimated) disk footprint.
pub fn installed_apps() -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = vec![];

    #[cfg(target_os = "linux")]
    {
        let _ = powershell; // silence unused import on this platform
        if let Some(o) = run("dpkg-query", &["-W", "-f=${Installed-Size}\t${Package}\n"]) {
            for l in o.lines() {
                if let Some((kb, name)) = l.split_once('\t') {
                    v.push((name.to_string(), kb.trim().parse::<u64>().unwrap_or(0) * 1024));
                }
            }
        } else if let Some(o) = run("rpm", &["-qa", "--queryformat", "%{SIZE}\t%{NAME}\n"]) {
            for l in o.lines() {
                if let Some((b, name)) = l.split_once('\t') {
                    v.push((name.to_string(), b.trim().parse().unwrap_or(0)));
                }
            }
        } else if let Some(o) = run("pacman", &["-Qi"]) {
            let (mut name, mut size) = (String::new(), 0u64);
            for l in o.lines() {
                if let Some(n) = l.strip_prefix("Name") { name = n.trim_start_matches([' ', ':']).trim().to_string(); }
                if let Some(s) = l.strip_prefix("Installed Size") {
                    let s = s.trim_start_matches([' ', ':']).trim();
                    let mut it = s.split_whitespace();
                    let n: f64 = it.next().and_then(|x| x.parse().ok()).unwrap_or(0.0);
                    let mul = match it.next() { Some("KiB") => 1024.0, Some("MiB") => 1048576.0, Some("GiB") => 1073741824.0, _ => 1.0 };
                    size = (n * mul) as u64;
                    v.push((name.clone(), size));
                }
            }
            let _ = size;
        }
        if let Some(o) = run("flatpak", &["list", "--columns=name,size"]) {
            for l in o.lines() {
                if let Some((name, size)) = l.split_once('\t') {
                    let mut it = size.split_whitespace();
                    let n: f64 = it.next().and_then(|x| x.replace(',', ".").parse().ok()).unwrap_or(0.0);
                    let mul = match it.next() { Some("kB") | Some("KB") => 1e3, Some("MB") => 1e6, Some("GB") => 1e9, _ => 1.0 };
                    v.push((format!("{name} (flatpak)"), (n * mul) as u64));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = (powershell, run);
        let token = AtomicBool::new(false);
        if let Ok(rd) = std::fs::read_dir("/Applications") {
            for e in rd.filter_map(|e| e.ok()) {
                let name = e.file_name().to_string_lossy().trim_end_matches(".app").to_string();
                v.push((name, dir_size(&e.path(), &token)));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let _ = run;
        let script = r#"Get-ItemProperty HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*,HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\* -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -and $_.EstimatedSize } | ForEach-Object { "$($_.EstimatedSize)`t$($_.DisplayName)" }"#;
        if let Some(o) = powershell(script) {
            for l in o.lines() {
                if let Some((kb, name)) = l.split_once('\t') {
                    v.push((name.trim().to_string(), kb.trim().parse::<u64>().unwrap_or(0) * 1024));
                }
            }
        }
    }

    v.retain(|(_, s)| *s > 0);
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v
}
