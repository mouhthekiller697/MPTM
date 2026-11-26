//! Startup applications.
//!
//! * Linux: XDG autostart (`~/.config/autostart`, `/etc/xdg/autostart`) – can be toggled.
//! * macOS: `~/Library/LaunchAgents` + `/Library/LaunchAgents` – toggled with `launchctl`.
//! * Windows: `Run` registry keys (HKCU / HKLM) – listed (read-only, to stay safe).

use crate::util::run;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub command: String,
    pub location: String,
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub can_toggle: bool,
}

pub fn list() -> Vec<Item> {
    let mut v = vec![];
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let user_dir = PathBuf::from(&home).join(".config/autostart");
        let mut seen = std::collections::HashSet::new();
        for (dir, label) in [(user_dir, "User autostart"), (PathBuf::from("/etc/xdg/autostart"), "System autostart")] {
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd.filter_map(|e| e.ok()) {
                    let p = e.path();
                    if p.extension().and_then(|x| x.to_str()) != Some("desktop") {
                        continue;
                    }
                    let fname = e.file_name().to_string_lossy().into_owned();
                    if !seen.insert(fname) {
                        continue; // the user's copy overrides the system one
                    }
                    let txt = std::fs::read_to_string(&p).unwrap_or_default();
                    let get = |k: &str| txt.lines().find_map(|l| l.strip_prefix(k)).map(|s| s.trim().to_string());
                    let hidden = get("Hidden=").as_deref() == Some("true") || get("X-GNOME-Autostart-enabled=").as_deref() == Some("false");
                    v.push(Item {
                        name: get("Name=").unwrap_or_else(|| p.file_stem().unwrap_or_default().to_string_lossy().into_owned()),
                        command: get("Exec=").unwrap_or_default(),
                        location: label.into(),
                        enabled: !hidden,
                        path: Some(p),
                        can_toggle: true,
                    });
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        // One process spawn for all agents instead of one per file.
        let disabled_list = run("launchctl", &["print-disabled", &format!("gui/{}", unsafe_uid())]).unwrap_or_default();
        for (dir, label) in [(format!("{home}/Library/LaunchAgents"), "User LaunchAgent"), ("/Library/LaunchAgents".to_string(), "System LaunchAgent")] {
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd.filter_map(|e| e.ok()) {
                    let p = e.path();
                    if p.extension().and_then(|x| x.to_str()) != Some("plist") {
                        continue;
                    }
                    let label_name = p.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                    let needle = format!("\"{label_name}\"");
                    let disabled = disabled_list.lines().any(|l| l.contains(&needle) && l.contains("disabled"));
                    v.push(Item { name: label_name, command: p.display().to_string(), location: label.into(), enabled: !disabled, path: Some(p), can_toggle: true });
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        for (key, label) in [
            (r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "Current user (registry)"),
            (r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run", "All users (registry)"),
        ] {
            if let Some(o) = run("reg", &["query", key]) {
                for l in o.lines() {
                    let parts: Vec<&str> = l.trim().splitn(3, "    ").collect();
                    if parts.len() == 3 && parts[1].starts_with("REG_") {
                        v.push(Item { name: parts[0].into(), command: parts[2].trim().into(), location: label.into(), enabled: true, path: None, can_toggle: false });
                    }
                }
            }
        }
    }
    let _ = run;
    v.sort_by_key(|i| i.name.to_lowercase());
    v
}

#[cfg(target_os = "macos")]
fn unsafe_uid() -> String {
    run("id", &["-u"]).unwrap_or_default().trim().to_string()
}

/// Enable / disable a startup entry. Returns an error message on failure.
pub fn set_enabled(item: &Item, enable: bool) -> Result<(), String> {
    let path = item.path.as_ref().ok_or("This entry cannot be changed from here.")?;

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").map_err(|e| e.to_string())?;
        let user_dir = PathBuf::from(home).join(".config/autostart");
        std::fs::create_dir_all(&user_dir).map_err(|e| e.to_string())?;
        let target = user_dir.join(path.file_name().ok_or("bad path")?);
        let txt = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut lines: Vec<String> = txt
            .lines()
            .filter(|l| !l.starts_with("X-GNOME-Autostart-enabled=") && !l.starts_with("Hidden="))
            .map(|s| s.to_string())
            .collect();
        lines.push(format!("X-GNOME-Autostart-enabled={}", if enable { "true" } else { "false" }));
        std::fs::write(&target, lines.join("\n") + "\n").map_err(|e| e.to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let uid = unsafe_uid();
        let sub = if enable { "enable" } else { "disable" };
        let label = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        run("launchctl", &[sub, &format!("gui/{uid}/{label}")]).map(|_| ()).ok_or_else(|| "launchctl failed".to_string())
    }
    #[cfg(target_os = "windows")]
    {
        let _ = (path, enable);
        Err("Toggling registry entries is disabled on Windows – use Task Manager › Startup.".into())
    }
}
