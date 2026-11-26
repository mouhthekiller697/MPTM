//! Static hardware / OS information (the "CPU-Z" tab).

use crate::util::{read_trim, run};
use sysinfo::System;

#[derive(Default, Clone)]
pub struct Specs {
    pub cpu: Vec<(String, String)>,
    pub board: Vec<(String, String)>,
    pub memory: Vec<(String, String)>,
    pub system: Vec<(String, String)>,
}

fn kv(k: &str, v: impl Into<String>) -> (String, String) {
    (k.to_string(), v.into())
}

pub fn collect() -> Specs {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();
    let cpus = sys.cpus();
    let first = cpus.first();

    let mut cpu = vec![
        kv("Name", first.map(|c| c.brand().trim().to_string()).unwrap_or_default()),
        kv("Vendor ID", first.map(|c| c.vendor_id().to_string()).unwrap_or_default()),
        kv("Architecture", std::env::consts::ARCH),
        kv("Physical cores", sys.physical_core_count().map(|n| n.to_string()).unwrap_or("?".into())),
        kv("Logical threads", cpus.len().to_string()),
        kv("Max reported clock", format!("{} MHz", cpus.iter().map(|c| c.frequency()).max().unwrap_or(0))),
    ];
    #[cfg(target_os = "linux")]
    {
        let human = |s: String| match s.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse::<u64>() {
            Ok(n) if s.ends_with('K') => crate::widgets::bytes(n * 1024),
            Ok(n) if s.ends_with('M') => crate::widgets::bytes(n * 1024 * 1024),
            _ => s,
        };
        for i in 0..6 {
            let b = format!("/sys/devices/system/cpu/cpu0/cache/index{i}");
            if let (Some(level), Some(kind), Some(size)) = (read_trim(&format!("{b}/level")), read_trim(&format!("{b}/type")), read_trim(&format!("{b}/size"))) {
                cpu.push(kv(&format!("L{level} {kind} cache"), human(size)));
            }
        }
        if let Ok(flags) = std::fs::read_to_string("/proc/cpuinfo") {
            if let Some(l) = flags.lines().find(|l| l.starts_with("flags")) {
                let wanted = ["sse4_2", "avx", "avx2", "avx512f", "aes", "fma", "sha_ni", "vmx", "svm"];
                let have: Vec<&str> = l.split_whitespace().filter(|f| wanted.contains(f)).collect();
                cpu.push(kv("Instructions", have.join(", ")));
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        for (k, key) in [("L1d cache", "hw.l1dcachesize"), ("L2 cache", "hw.l2cachesize"), ("L3 cache", "hw.l3cachesize")] {
            if let Some(v) = run("sysctl", &["-n", key]) {
                cpu.push(kv(k, format!("{} KB", v.trim().parse::<u64>().unwrap_or(0) / 1024)));
            }
        }
    }

    let mut board = vec![];
    #[cfg(target_os = "linux")]
    for (k, f) in [
        ("Manufacturer", "board_vendor"),
        ("Model", "board_name"),
        ("Version", "board_version"),
        ("BIOS vendor", "bios_vendor"),
        ("BIOS version", "bios_version"),
        ("BIOS date", "bios_date"),
        ("Product", "product_name"),
    ] {
        if let Some(v) = read_trim(&format!("/sys/class/dmi/id/{f}")) {
            board.push(kv(k, v));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(m) = run("sysctl", &["-n", "hw.model"]) {
            board.push(kv("Model identifier", m.trim()));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(o) = crate::util::powershell(r#"$b=Get-CimInstance Win32_BaseBoard; $x=Get-CimInstance Win32_BIOS; "$($b.Manufacturer)|$($b.Product)|$($x.SMBIOSBIOSVersion)|$($x.ReleaseDate)""#) {
            let p: Vec<&str> = o.trim().split('|').collect();
            if p.len() == 4 {
                board.push(kv("Manufacturer", p[0]));
                board.push(kv("Model", p[1]));
                board.push(kv("BIOS version", p[2]));
                board.push(kv("BIOS date", p[3]));
            }
        }
    }
    if board.is_empty() {
        board.push(kv("Info", "Not exposed by this platform (may require elevated privileges)."));
    }
    let _ = (&run, &read_trim);

    let memory = vec![
        kv("Total RAM", crate::widgets::bytes(sys.total_memory())),
        kv("Total swap", crate::widgets::bytes(sys.total_swap())),
    ];

    let system = vec![
        kv("Operating system", System::long_os_version().unwrap_or_default()),
        kv("OS name", System::name().unwrap_or_default()),
        kv("Kernel", System::kernel_version().unwrap_or_default()),
        kv("Hostname", System::host_name().unwrap_or_default()),
        kv("Architecture", std::env::consts::ARCH),
        kv("Family", std::env::consts::OS),
    ];

    Specs { cpu, board, memory, system }
}
