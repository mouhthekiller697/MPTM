//! Linux desktop-file icon lookup for process rows.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct DesktopIcons {
    apps: HashMap<String, PathBuf>,
}

impl DesktopIcons {
    pub fn discover() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        let mut icon_files = HashMap::new();
        for dir in [
            home.join(".local/share/icons"),
            home.join(".icons"),
            PathBuf::from("/usr/share/icons/hicolor"),
            PathBuf::from("/usr/share/icons/Adwaita"),
            PathBuf::from("/usr/share/pixmaps"),
        ] {
            for entry in WalkDir::new(dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if !matches!(
                    path.extension()
                        .and_then(|e| e.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("png" | "jpg" | "jpeg")
                ) {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    icon_files
                        .entry(stem.to_ascii_lowercase())
                        .or_insert_with(|| path.to_path_buf());
                }
            }
        }

        let mut apps = HashMap::new();
        for dir in [
            home.join(".local/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/usr/share/applications"),
        ] {
            for entry in WalkDir::new(dir)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file()
                    || entry.path().extension().and_then(|e| e.to_str()) != Some("desktop")
                {
                    continue;
                }
                let Ok(contents) = std::fs::read_to_string(entry.path()) else {
                    continue;
                };
                let mut exec = None;
                let mut icon = None;
                for line in contents.lines() {
                    if let Some(value) = line.strip_prefix("Exec=") {
                        exec = value.split_ascii_whitespace().next().map(str::to_string);
                    } else if let Some(value) = line.strip_prefix("Icon=") {
                        icon = Some(value.trim().to_string());
                    }
                }
                let Some(icon) = icon else { continue };
                let icon_path = if Path::new(&icon).is_file() {
                    Some(PathBuf::from(&icon))
                } else {
                    let name = Path::new(&icon)
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or(&icon)
                        .to_ascii_lowercase();
                    icon_files.get(&name).cloned()
                };
                let Some(icon_path) = icon_path else { continue };
                if let Some(exec) = exec {
                    let name = Path::new(exec.trim_matches('"'))
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if !name.is_empty() {
                        apps.entry(name).or_insert_with(|| icon_path.clone());
                    }
                }
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    apps.entry(stem.to_ascii_lowercase()).or_insert(icon_path);
                }
            }
        }
        Self { apps }
    }

    pub fn for_process(&self, name: &str) -> Option<PathBuf> {
        let name = name.to_ascii_lowercase();
        self.apps
            .get(&name)
            .or_else(|| self.apps.get(name.trim_end_matches(".bin")))
            .cloned()
    }
}
