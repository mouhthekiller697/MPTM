//! GPU discovery and best-effort telemetry without vendor SDKs.

#[cfg(target_os = "linux")]
use crate::util::read_trim;
use crate::util::run;
use std::collections::HashMap;

#[derive(Clone, Default, Debug)]
pub struct GpuInfo {
    pub name: String,
    pub util: f32,
    pub vram_used: u64,
    pub vram_total: u64,
    pub temp: Option<f32>,
    pub power_w: Option<f32>,
    pub driver: String,
}

/// Return every detected adapter. Telemetry is populated where the driver exposes it.
pub fn info() -> Vec<GpuInfo> {
    #[cfg(target_os = "linux")]
    {
        let mut gpus = nvidia_info();
        for gpu in linux_drm_info() {
            if gpu.driver == "nvidia" && gpus.iter().any(|known| known.driver != "DRM") {
                continue;
            }
            if !gpus.iter().any(|g| g.name == gpu.name) {
                gpus.push(gpu);
            }
        }
        gpus
    }

    #[cfg(target_os = "windows")]
    {
        run(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance Win32_VideoController | ForEach-Object { $_.Name }",
            ],
        )
        .unwrap_or_default()
        .lines()
        .filter(|name| !name.trim().is_empty())
        .map(|name| GpuInfo {
            name: name.trim().to_string(),
            driver: "Windows display driver".into(),
            ..Default::default()
        })
        .collect()
    }

    #[cfg(target_os = "macos")]
    {
        let output = run("system_profiler", &["SPDisplaysDataType"]).unwrap_or_default();
        output
            .lines()
            .filter_map(|line| line.trim().strip_prefix("Chipset Model:").map(str::trim))
            .map(|name| GpuInfo {
                name: name.to_string(),
                driver: "Apple display driver".into(),
                ..Default::default()
            })
            .collect()
    }
}

/// pid -> GPU utilisation percent. Empty when no supported source is available.
pub fn process_usage() -> HashMap<u32, f32> {
    let mut per_pid = HashMap::new();
    let Some(pmon) = run("nvidia-smi", &["pmon", "-c", "1", "-s", "u"]) else {
        return per_pid;
    };
    for l in pmon.lines().filter(|l| !l.trim_start().starts_with('#')) {
        let mut it = l.split_ascii_whitespace();
        it.next();
        let pid = it.next().and_then(|s| s.parse::<u32>().ok());
        it.next();
        let sm = it.next().and_then(|s| s.parse::<f32>().ok());
        if let (Some(pid), Some(sm)) = (pid, sm) {
            *per_pid.entry(pid).or_insert(0.0) += sm;
        }
    }
    per_pid
}

#[cfg(target_os = "linux")]
fn nvidia_info() -> Vec<GpuInfo> {
    let Some(out) = run(
        "nvidia-smi",
        &[
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,driver_version",
            "--format=csv,noheader,nounits",
        ],
    ) else {
        return Vec::new();
    };
    out.lines()
        .filter_map(|line| {
            let mut it = line.split(',').map(str::trim);
            let name = it.next()?.to_string();
            let util = it.next()?.parse().unwrap_or(0.0);
            let mib = |s: &str| {
                s.parse::<f64>()
                    .map(|v| (v * 1024.0 * 1024.0) as u64)
                    .unwrap_or(0)
            };
            let vram_used = mib(it.next()?);
            let vram_total = mib(it.next()?);
            let temp = it.next().and_then(|s| s.parse().ok());
            let power_w = it.next().and_then(|s| s.parse().ok());
            let driver = it.next().unwrap_or_default().to_string();
            Some(GpuInfo {
                name,
                util,
                vram_used,
                vram_total,
                temp,
                power_w,
                driver,
            })
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn linux_drm_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    let Ok(cards) = std::fs::read_dir("/sys/class/drm") else {
        return gpus;
    };
    for card in cards.flatten() {
        let entry = card.file_name().to_string_lossy().into_owned();
        let Some(index) = entry.strip_prefix("card") else {
            continue;
        };
        if index.is_empty() || !index.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let base = card.path().join("device");
        let vendor = read_trim(&base.join("vendor").to_string_lossy()).unwrap_or_default();
        let driver = std::fs::read_link(base.join("driver"))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "DRM".into());
        let fallback = match vendor.trim_start_matches("0x") {
            "1002" => "AMD GPU",
            "8086" => "Intel GPU",
            "10de" => "NVIDIA GPU",
            _ => "Graphics adapter",
        };
        let name = read_trim(&base.join("product_name").to_string_lossy())
            .or_else(|| read_trim(&base.join("label").to_string_lossy()))
            .unwrap_or_else(|| format!("{fallback} ({driver})"));

        let util = read_trim(&base.join("gpu_busy_percent").to_string_lossy())
            .or_else(|| read_trim(&base.join("gt_busy_percent").to_string_lossy()))
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let read_bytes = |file: &str| {
            read_trim(&base.join(file).to_string_lossy())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        };
        let temp = std::fs::read_dir(base.join("hwmon"))
            .ok()
            .and_then(|entries| {
                entries
                    .flatten()
                    .find_map(|e| read_trim(&e.path().join("temp1_input").to_string_lossy()))
            })
            .and_then(|s| s.parse::<f32>().ok())
            .map(|m| m / 1000.0);
        gpus.push(GpuInfo {
            name,
            util,
            vram_used: read_bytes("mem_info_vram_used"),
            vram_total: read_bytes("mem_info_vram_total"),
            temp,
            power_w: None,
            driver,
        });
    }
    gpus
}
