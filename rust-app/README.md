# Ferrum Monitor — native Rust app

Cross-platform (Linux · macOS · Windows) system task manager, 100 % Rust,
built with [`egui`/`eframe`](https://github.com/emilk/egui) (GPU-accelerated immediate-mode UI)
and [`sysinfo`](https://crates.io/crates/sysinfo).

> **Web preview:** the repository root contains a React/Vite preview of the same interface.
> This folder is the real, purely-Rust program.

## Tabs

| Tab | What it does | Data source |
| --- | --- | --- |
| General | CPU + per-thread load, GPU, RAM, VRAM, disk activity & read/write speed, network, battery | sysinfo, `battery`, `nvidia-smi` / amdgpu sysfs |
| Applications | CPU, RAM, GPU, network, disk read/write speed per app (same-named processes grouped), sortable + filterable | sysinfo, `nvidia-smi pmon` |
| Disk manager | Physical disks, partition layout, mounted volumes, S.M.A.R.T. health, opens the OS partition tool | `lsblk -J` (Linux), sysinfo, `smartctl`, `Get-PhysicalDisk` |
| Storage analyser | Parallel folder-size scan with drill-down, plus space used by installed applications | walkdir; dpkg / rpm / pacman / flatpak; `/Applications`; registry |
| Specs | CPU-Z style CPU, mainboard, memory, GPU and OS information | sysinfo, DMI sysfs, CPU cache sysfs, `sysctl`, CIM |
| Startup | Lists startup applications and toggles them where supported | XDG autostart files, launchctl, registry `Run` keys |
| Appearance | 6 themes, arbitrary accent colour, font (system / mono / custom `.ttf`/`.otf`), text size, UI scale, corner radius, compact mode, refresh rate | persisted via eframe storage |

## 1. Install Rust

* **Windows:** [rustup-init.exe](https://rustup.rs). During setup keep the default
  **MSVC** toolchain, and install **Visual Studio Build Tools** with
  *Desktop development with C++* (gives you the MSVC linker). No MinGW needed.
* **macOS:** `xcode-select --install`, then
  `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* **Linux:** same rustup command, then optionally:
  * Debian/Ubuntu: `sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev`
  * Fedora: `sudo dnf install gcc pkgconf-pkg-config libxkbcommon-devel wayland-devel libX11-devel mesa-libGL-devel`
  * Arch: `sudo pacman -S base-devel pkgconf libxkbcommon wayland libx11 mesa`

Tested with **rustc 1.100.0**. `rustup update stable` if your toolchain is old.

## 2. Build & run

```bash
cd rust-app
cargo run --release        # compiles, then starts the app
```

Build only:

```bash
cargo build --release
./target/release/ferrum-monitor          # Linux / macOS
.\target\release\ferrum-monitor.exe      # Windows
```

Install the binary into `~/.cargo/bin`:

```bash
cargo install --path .
```

Optional checks and benchmarks:

```bash
cargo check                              # fast type check
cargo run --release --example bench      # sampling cost per pass
cargo run --release --example ui_bench   # headless egui rendering cost
```

### Status in this workspace

| Step | Result |
| --- | --- |
| `cargo check` | success — 0 errors, 0 warnings |
| `cargo build --release` | success — stripped binary ≈ 9.8 MB (includes 2.7 MB of bundled fallback fonts) |
| GUI smoke test under Xvfb (X11 + Mesa) | all 7 tabs opened, real folder scan completed, no panics |
| Sampling benchmark | min 1.2 ms · median ≈1.8 ms · max 2.3–3.6 ms per pass |
| UI benchmark | General 0.079 ms · Applications 0.355 ms median/frame |
| Hardware available in benchmark container | 2 vCPU, 75 processes, no GPU telemetry, no battery |

These numbers are from a container (no GPU compositing), so treat them as a lower bound.
The interface was launched under a virtual X display (Xvfb + Mesa) and every tab was captured.
Some icons needed extra font fallbacks, which now ship inside the binary (`assets/`, open licences).

## 3. Platform notes & permissions

* **GPU:** NVIDIA uses the `nvidia-smi` executable, so ensure it is on `PATH` (it ships with the driver).
  AMD on Linux is read from `/sys/class/drm/card*/device`. Intel and Apple GPUs are not supported.
* **S.M.A.R.T.:** install `smartmontools`. Reading raw device data needs elevation. Grant the capability
  instead of running the whole GUI as root:

  ```bash
  sudo setcap cap_sys_rawio,cap_sys_admin+ep "$(command -v smartctl)"
  ```

* **Partition editing:** “Open partition tool” launches GParted / GNOME Disks (Linux),
  Disk Utility (macOS) or `diskmgmt.msc` (Windows). Ferrum Monitor deliberately does not format or
  resize disks itself.
* **Startup:** Linux toggles write `X-GNOME-Autostart-enabled=` in `~/.config/autostart/*.desktop`;
  macOS uses `launchctl disable/enable`. Windows `Run` keys are listed read-only.
* **Battery:** absent on desktops; the card says so instead of failing.
* **Linux display:** if the window cannot open on Wayland, try
  `WINIT_UNIX_BACKEND=x11 cargo run --release` (or install the X11/Wayland packages above).
  A working OpenGL driver (Mesa on Linux) is required by the `glow` renderer.

## 4. Cross-compiling

Do not cross-compile the GUI; build natively on each OS. `.github/workflows/build.yml` at the repository
root does exactly that on GitHub Actions (Ubuntu, Windows, macOS) and uploads release artifacts.
For manual platform builds, run the same `cargo build --release` command on the target OS.

## 5. Performance design

* **Threading:** all hardware queries run in a named background thread. It publishes one immutable
  `Snapshot` under a mutex; the render thread never waits on `sysinfo` or subprocesses.
* **Reuse:** `Disks`, `Components` and `Networks` are allocated once and refreshed in place. Full
  re-enumeration (hot-plug detection) is every 60 s; temperatures/disks every 5 s; battery every 10 s;
  NVIDIA per-process usage every 6 s (vs. the utilisation query every 2 s).
* **Fewer allocations:** process names are borrowed from `sysinfo` while grouping, so each application
  allocates one `String`, not one per process per tick. The app list uses an unstable sort.
* **Correct I/O rates:** cumulative per-PID read/write counters are diffed and cleared each tick.
  This avoids per-app rates being computed from lifetime totals.
* **Timing:** each pass measures its own duration and subtracts it from the sleep, so the 1 s cadence
  does not drift when sampling is slow.
* **UI:** app filtering and name sorting are case-insensitive without allocating; sparkline area fill is
  a single mesh (previously ~90 small polygons per chart per frame); storage entries are handed to the
  UI as `Arc` snapshots published in batches; `launchctl` is queried once per scan.
* **Style engine:** visuals are applied only when settings change; font definitions are rebuilt only when
  the font selection changes; UI zoom is applied after a drag so the slider cannot jitter.
* **Release profile:** `opt-level = 3`, thin LTO, `codegen-units = 1`, `panic = "abort"`, symbols stripped.

## 6. Known limits

* Per-application **network** utilisation has no portable API. The column/type is wired (`AppRow::net`)
  but remains “—” until a platform back-end is added.
* Disk throughput is the sum of process I/O deltas, not a kernel block-device counter.
* Per-application GPU usage requires NVIDIA's `nvidia-smi pmon`.
* Windows startup entries cannot be toggled from the app (by design, to avoid registry damage).
* The installer/packaging is left to `cargo install` or a bundler such as `cargo-bundle`/`cargo-dist`.

## 7. Source map

```
src/main.rs      app shell, navigation, persistence, appearance application timing
src/sampler.rs   background sampling thread, snapshots, history
src/tabs.rs      one view per tab (General, Apps, Disks, Storage, Specs, Startup)
src/settings.rs  themes, palettes, fonts, UI scale, settings UI
src/widgets.rs   cards, rings, bars, sparklines, formatting
src/gpu.rs       NVIDIA / AMD GPU collection
src/disks.rs     disk & partition enumeration, S.M.A.R.T., partition tool
src/storage.rs   parallel folder scanner, installed-app sizes
src/specs.rs     static hardware / OS information
src/startup.rs   startup entries + toggles
src/util.rs      subprocess helpers (no shell, no console window on Windows)
examples/bench.rs     sampling benchmark
examples/ui_bench.rs  headless rendering benchmark
```
