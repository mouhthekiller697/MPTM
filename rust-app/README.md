# MPSM — Multipurpose System Manager

Native cross-platform system monitor and manager built with Rust, eframe/egui and sysinfo.

## Features

| Section | Features |
| --- | --- |
| General | CPU/thread, GPU, memory, disk, network, battery and sensor summaries |
| Applications | Grouped processes, resource sorting/filtering, per-process disk I/O, Linux desktop icons |
| Disk Manager | Physical disks, partitions, mounted volumes, S.M.A.R.T. health and OS partition tool |
| Storage Analyser | Folder-size scan with drill-down and estimated installed-app sizes |
| Specs | CPU, mainboard, memory, all discovered display adapters and operating-system information |
| Startup | Startup entries with supported enable/disable operations |
| Appearance | Themes, accent colour, installed/custom fonts, text size, UI scale and compact mode |

## Build and run

Install the stable Rust toolchain using [rustup](https://rustup.rs).

```bash
cargo run --release
```

Or build and launch directly:

```bash
cargo build --release
./target/release/mpsm               # Linux / macOS
.\target\release\mpsm.exe           # Windows PowerShell
```

### Linux prerequisites

Install a C compiler, pkg-config, X11/Wayland development files and OpenGL/Mesa headers. Examples:

```bash
# Debian / Ubuntu
sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config libxkbcommon-devel wayland-devel libX11-devel mesa-libGL-devel
# Arch
sudo pacman -S base-devel pkgconf libxkbcommon wayland libx11 mesa
```

The GitHub Actions workflow builds the Linux binary on Ubuntu 22.04 and creates an x86-64 AppImage artifact. This is a portable Linux package for common distributions, but requires a compatible glibc and a working X11 or Wayland/OpenGL desktop.

### Windows and macOS

Windows needs the MSVC Rust toolchain and Visual Studio Build Tools with *Desktop development with C++*. macOS needs Xcode Command Line Tools.

## Platform notes

* CPU inventory and usage are collected through sysinfo across supported operating systems.
* Linux detects NVIDIA through `nvidia-smi` and all DRM adapters through `/sys/class/drm`; AMD/Intel telemetry is read when their drivers expose counters. Windows and macOS enumerate their available display adapters, although telemetry depends on platform drivers.
* Linux S.M.A.R.T. reads run as the current user first, then use `pkexec` to request authentication when needed. Install `smartmontools` and polkit for this feature.
* The Applications tab resolves icons from installed Linux `.desktop` launchers. If no matching icon is available, the process name remains visible.
* Font selection lists installed TrueType/OpenType font files and also accepts a font file path. Font files unsupported by egui fall back to the built-in font.
* Disk editing remains delegated to the operating system's partition utility; MPSM does not format or resize disks.

## Checks and benchmarks

```bash
cargo check --all-targets
cargo run --release --example bench
cargo run --release --example ui_bench
```
