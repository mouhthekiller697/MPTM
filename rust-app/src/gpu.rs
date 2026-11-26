//! GPU sampling without vendor SDKs.
//!
//! * NVIDIA (Linux / Windows): `nvidia-smi` (ships with the driver) – utilisation, VRAM,
//!   temperature, power and per-process GPU usage.
//! * AMD (Linux): amdgpu sysfs counters.
//! * Anything else: `None` – the UI shows "not available".
//!
//! The cheap summary query and the more expensive per-process query are separate, so
//! the sampler can call the latter far less often.

use crate::util::{read_trim, run};
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

/// Summary utilisation + VRAM + thermal data.
pub fn info() -> Option<GpuInfo> {
    nvidia_info().or_else(amd_sysfs)
}

/// pid -> GPU utilisation percent. Empty when no supported source is available.
pub fn process_usage() -> HashMap<u32, f32> {
    let mut per_pid = HashMap::new();
    let Some(pmon) = run("nvidia-smi", &["pmon", "-c", "1", "-s", "u"]) else {
        return per_pid;
    };
    for l in pmon.lines().filter(|l| !l.trim_start().starts_with('#')) {
        // columns: gpu pid type sm mem enc dec command
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

fn nvidia_info() -> Option<GpuInfo> {
    let out = run(
        "nvidia-smi",
        &[
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,driver_version",
            "--format=csv,noheader,nounits",
        ],
    )?;
    let line = out.lines().next()?;
    let mut it = line.split(',').map(str::trim);
    let name = it.next()?.to_string();
    let util = it.next()?.parse().unwrap_or(0.0);
    let mib = |s: &str| s.parse::<f64>().map(|v| (v * 1024.0 * 1024.0) as u64).unwrap_or(0);
    let vram_used = mib(it.next()?);
    let vram_total = mib(it.next()?);
    let temp = it.next().and_then(|s| s.parse().ok());
    let power_w = it.next().and_then(|s| s.parse().ok());
    let driver = it.next().unwrap_or_default().to_string();
    Some(GpuInfo { name, util, vram_used, vram_total, temp, power_w, driver })
}

fn amd_sysfs() -> Option<GpuInfo> {
    for i in 0..4 {
        let base = format!("/sys/class/drm/card{i}/device");
        if let Some(util) = read_trim(&format!("{base}/gpu_busy_percent")).and_then(|s| s.parse::<f32>().ok()) {
            let rd = |n: &str| read_trim(&format!("{base}/{n}")).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
            // temperature lives in hwmon
            let temp = std::fs::read_dir(format!("{base}/hwmon"))
                .ok()
                .and_then(|mut d| d.next())
                .and_then(|e| e.ok())
                .and_then(|e| read_trim(&format!("{}/temp1_input", e.path().display())))
                .and_then(|s| s.parse::<f32>().ok())
                .map(|m| m / 1000.0);
            return Some(GpuInfo {
                name: read_trim(&format!("{base}/product_name")).unwrap_or_else(|| "AMD GPU (amdgpu)".into()),
                util,
                vram_used: rd("mem_info_vram_used"),
                vram_total: rd("mem_info_vram_total"),
                temp,
                power_w: None,
                driver: "amdgpu".into(),
            });
        }
    }
    None
}
