//! Disk manager back-end: physical disks, partitions and health.
//!
//! * Linux: `lsblk -J` for the full partition tree (incl. unmounted partitions).
//! * macOS / Windows (and Linux fallback): mounted volumes from `sysinfo`.
//! * Health: `smartctl` (smartmontools) on Linux/macOS, `Get-PhysicalDisk` on Windows.
//! * Destructive operations (create / resize / format) are intentionally delegated to the
//!   operating system's own partition tool, which handles elevation and safety prompts.

use crate::util::launch;
#[cfg(windows)]
use crate::util::powershell;
#[cfg(not(windows))]
use crate::util::program_available;
#[cfg(not(windows))]
use crate::util::run;
#[cfg(target_os = "linux")]
use crate::util::run_as_admin;
#[cfg(target_os = "linux")]
use serde_json::Value;
use sysinfo::Disks;

#[derive(Clone, Default)]
pub struct Part {
    pub name: String,
    pub size: u64,
    pub fstype: String,
    pub mount: String,
    pub used: Option<u64>,
}

#[derive(Clone, Default)]
pub struct PhysDisk {
    pub dev: String,
    pub model: String,
    pub size: u64,
    pub table: String,
    pub kind: String,
    pub parts: Vec<Part>,
}

#[cfg(target_os = "linux")]
fn num(v: &Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}
#[cfg(target_os = "linux")]
fn truthy(v: &Value) -> bool {
    v.as_bool()
        .unwrap_or_else(|| v.as_u64() == Some(1) || v.as_str() == Some("1"))
}
#[cfg(target_os = "linux")]
fn text(v: &Value) -> String {
    v.as_str().unwrap_or("").trim().to_string()
}

pub fn list() -> Vec<PhysDisk> {
    #[cfg(target_os = "linux")]
    if let Some(d) = lsblk() {
        if !d.is_empty() {
            return d;
        }
    }
    fallback()
}

#[cfg(target_os = "linux")]
fn lsblk() -> Option<Vec<PhysDisk>> {
    let out = run(
        "lsblk",
        &[
            "-J",
            "-b",
            "-o",
            "NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT,MODEL,PTTYPE,ROTA,TRAN",
        ],
    )?;
    let json: Value = serde_json::from_str(&out).ok()?;
    let mounted = Disks::new_with_refreshed_list();
    let used_of = |mount: &str| -> Option<u64> {
        mounted
            .iter()
            .find(|d| d.mount_point().to_string_lossy() == mount)
            .map(|d| d.total_space().saturating_sub(d.available_space()))
    };
    let mut res = vec![];
    for d in json.get("blockdevices")?.as_array()? {
        if text(&d["type"]) != "disk" {
            continue; // skip loop devices, optical drives, ...
        }
        let tran = text(&d["tran"]);
        let kind = if tran == "nvme" {
            "NVMe SSD".to_string()
        } else if tran == "usb" {
            "USB".to_string()
        } else if truthy(&d["rota"]) {
            "HDD".to_string()
        } else {
            "SSD".to_string()
        };
        let mut parts = vec![];
        if let Some(ch) = d["children"].as_array() {
            for c in ch {
                let mount = text(&c["mountpoint"]);
                parts.push(Part {
                    name: text(&c["name"]),
                    size: num(&c["size"]),
                    fstype: text(&c["fstype"]),
                    used: if mount.is_empty() {
                        None
                    } else {
                        used_of(&mount)
                    },
                    mount,
                });
            }
        } else {
            let mount = text(&d["mountpoint"]);
            parts.push(Part {
                name: text(&d["name"]),
                size: num(&d["size"]),
                fstype: text(&d["fstype"]),
                used: used_of(&mount),
                mount,
            });
        }
        let model = {
            let m = text(&d["model"]);
            if m.is_empty() {
                if tran == "usb" {
                    "USB storage device".to_string()
                } else {
                    // virtual disks (VMs) usually report no model; expose their bus instead
                    format!(
                        "Virtual disk · {} bus",
                        if tran.is_empty() {
                            "virtual"
                        } else {
                            tran.as_str()
                        }
                    )
                }
            } else {
                m
            }
        };
        res.push(PhysDisk {
            dev: text(&d["name"]),
            model,
            size: num(&d["size"]),
            table: text(&d["pttype"]).to_uppercase(),
            kind,
            parts,
        });
    }
    Some(res)
}

fn fallback() -> Vec<PhysDisk> {
    let mounted = Disks::new_with_refreshed_list();
    mounted
        .iter()
        .map(|d| {
            let total = d.total_space();
            let name = d.name().to_string_lossy().into_owned();
            let mount = d.mount_point().to_string_lossy().into_owned();
            PhysDisk {
                dev: if name.is_empty() {
                    mount.clone()
                } else {
                    name.clone()
                },
                model: if name.is_empty() {
                    "Volume".into()
                } else {
                    name
                },
                size: total,
                table: String::new(),
                kind: d.kind().to_string(),
                parts: vec![Part {
                    name: mount.clone(),
                    size: total,
                    fstype: d.file_system().to_string_lossy().into_owned(),
                    used: Some(total.saturating_sub(d.available_space())),
                    mount,
                }],
            }
        })
        .collect()
}

/// Blocking health query – call it from a worker thread.
pub fn health(dev: &str) -> String {
    #[cfg(windows)]
    {
        let _ = dev;
        return powershell(
            r#"Get-PhysicalDisk | ForEach-Object { "$($_.FriendlyName): $($_.HealthStatus) / $($_.OperationalStatus) ($($_.MediaType))" }"#,
        )
        .unwrap_or_else(|| "Could not query Get-PhysicalDisk.".into());
    }
    #[cfg(not(windows))]
    {
        let path = if dev.starts_with("/dev/") {
            dev.to_string()
        } else {
            format!("/dev/{dev}")
        };
        let args = ["-H", "-A", &path];
        let direct = if program_available("smartctl") {
            run("smartctl", &args)
        } else {
            None
        };
        #[cfg(target_os = "linux")]
        let output = direct.or_else(|| run_as_admin("smartctl", &args));
        #[cfg(not(target_os = "linux"))]
        let output = direct;
        match output {
            Some(o) => o,
            None => "smartctl is unavailable or access was denied. On Linux, install smartmontools and ensure polkit (pkexec) is available; MPSM will request authentication when elevated access is needed.".into(),
        }
    }
}

/// Open the OS partition editor.
pub fn open_partition_tool() -> bool {
    #[cfg(target_os = "windows")]
    {
        return launch("cmd", &["/C", "start", "diskmgmt.msc"]);
    }
    #[cfg(target_os = "macos")]
    {
        return launch("open", &["-a", "Disk Utility"]);
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        [
            "gparted",
            "partitionmanager",
            "gnome-disks",
            "kde-partitionmanager",
        ]
        .iter()
        .any(|t| launch(t, &[]))
    }
}
