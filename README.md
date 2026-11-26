# MPSM — Multipurpose System Manager

MPSM is a cross-platform system monitor and manager with seven sections:

**General** (CPU, GPU, RAM, disk, network and battery) · **Applications** (grouped processes with Linux desktop icons) ·
**Disk Manager** (partitions and S.M.A.R.T.) · **Storage Analyser** · **Specs** (CPU, board, memory, GPUs and OS) ·
**Startup** · **Appearance** (themes, colours, installed/custom fonts and scaling).

## Native Rust app

`rust-app/` contains the native app, implemented in Rust with eframe/egui and sysinfo. The repository-root React/Vite app is a simulated browser preview and cannot inspect local hardware.

```bash
cd rust-app
cargo run --release
```

For a release binary:

```bash
cargo build --release
./target/release/mpsm                 # Linux / macOS
# .\target\release\mpsm.exe           # Windows PowerShell
```

Install Rust with rustup. Linux builds need a C toolchain, pkg-config, X11/Wayland development libraries and OpenGL/Mesa headers; see `rust-app/README.md` for distro-specific package commands.

## Linux portability and other platforms

GitHub Actions builds native binaries for Ubuntu 22.04, Windows and macOS, and packages an x86-64 AppImage for Linux. Download the matching artifact from **Actions → build-mpsm → Artifacts**. The AppImage is intended to run across common Linux distributions without installing MPSM; it still depends on a compatible glibc version and a working X11 or Wayland/OpenGL desktop.

## Hardware and permissions

CPU details use sysinfo across supported operating systems. GPU discovery reports all detected adapters: NVIDIA, AMD and other DRM devices on Linux, and display adapters on Windows/macOS. Available GPU metrics depend on the driver; NVIDIA uses `nvidia-smi`, while Linux DRM telemetry is used where exposed.

MPSM runs as the normal user. On Linux, S.M.A.R.T. reads run directly first and use `pkexec` to request desktop authentication if elevated access is required. Partition editing is handed off to the OS partition tool, which manages its own privilege prompt. MPSM does not format or resize disks itself.

The Specs view shows CPU/GPU vendor wordmarks and the operating-system mark. The Applications view resolves Linux process icons from installed `.desktop` launchers when possible; other platforms use a text fallback.

## Browser preview

```bash
npm install
npm run dev
npm run build
```

See [`rust-app/README.md`](rust-app/README.md) for native build prerequisites, feature details, and known platform limitations.
