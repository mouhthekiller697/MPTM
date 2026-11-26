# Ferrum Monitor

A cross-platform system monitor / task manager with seven dedicated sections:

**General** (CPU, per-thread load, GPU, RAM, VRAM, disk activity + read/write speed, network, battery) ·
**Applications** (per-app CPU, RAM, GPU, network, disk speed) ·
**Disk Manager** (disks, partitions, volumes, S.M.A.R.T. health) ·
**Storage Analyser** (folder sizes with drill-down, space per installed app) ·
**Specs** (CPU-Z style CPU / board / memory / GPU / OS viewer) ·
**Startup** (startup apps, toggleable where the OS allows it) ·
**Appearance** (6 themes, any accent colour, font + custom font file, UI size/ratio, corner radius, compact mode).

## Two implementations in this repository

| Path | Language | Purpose |
| --- | --- | --- |
| `rust-app/` | **100 % Rust** (eframe/egui + sysinfo) | The real program: native, cross-platform (Linux · macOS · Windows). |
| `src/`, `index.html` | React + Vite (browser preview only) | A fully interactive preview of the same UI, using simulated data. A browser cannot read hardware, and this sandbox only serves a web build. |

The native Rust program is the deliverable for "written purely in Rust"; the browser app is a visual preview with identical tabs, controls and appearance options (including live theme/font/UI-size customisation).

## Compile: native Rust program

**Prerequisites:** a recent stable Rust toolchain (tested with **rustc 1.100.0**) and the platform toolchain:

| OS | Install |
| --- | --- |
| Windows | [rustup-init.exe](https://rustup.rs) + **Visual Studio Build Tools** with “Desktop development with C++” (MSVC linker). Use the default `stable-x86_64-pc-windows-msvc` toolchain. |
| macOS | `xcode-select --install` + rustup (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh`) |
| Linux | rustup, plus a working GL driver and X11/Wayland session. If the build complains about missing X11/Wayland/GL headers:<br>`sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev libx11-dev libgl1-mesa-dev` (Debian/Ubuntu)<br>`sudo dnf install gcc pkgconf-pkg-config libxkbcommon-devel wayland-devel libX11-devel mesa-libGL-devel` (Fedora)<br>`sudo pacman -S base-devel pkgconf libxkbcommon wayland libx11 mesa` (Arch) |

**Build and run (recommended: release mode):**

```bash
cd rust-app
cargo run --release
```

From the repository root without `cd`:

```bash
cargo run --release --manifest-path rust-app/Cargo.toml
```

**Build only, then launch the binary:**

```bash
cd rust-app
cargo build --release
./target/release/ferrum-monitor          # Linux / macOS
# .\target\release\ferrum-monitor.exe    # Windows (PowerShell)
```

To put it on your `PATH`:

```bash
cargo install --path rust-app   # installs `ferrum-monitor` into ~/.cargo/bin
```

**Quick compile check / benchmarks:**

```bash
cd rust-app
cargo check                       # fast type-check
cargo run --release --example bench      # sampling-thread benchmark
cargo run --release --example ui_bench   # headless egui render benchmark
```

### Verification status

The native app was compiled in this workspace with Rust 1.100.0:

* `cargo check` → **0 errors, 0 warnings**
* `cargo build --release` → **success**, stripped `ferrum-monitor` binary ≈ **9.8 MB** (incl. 2.7 MB bundled fonts)
* launched under a virtual X display: every tab was opened, a real folder scan ran, no panics
* sampler benchmark → **≈1.8 ms median** per pass (2-vCPU sandbox, 73–75 applications; 1.2–2.3 ms across runs; excludes the 1 s sleep)
* headless UI benchmark → General **0.08 ms**, Applications **0.36 ms** median per frame (`p95` 0.51 ms)

Measured in a container without a GPU/battery; real machines will differ, and GPU painting is extra.

## Compile / run the browser preview

Requires Node.js 20+.

```bash
npm install
npm run dev       # http://localhost:5173
npm run build     # single-file production build in dist/index.html
npm run preview   # serve the production build
```

## Permissions

* The app itself runs as your normal user. It never writes to raw disks.
* **S.M.A.R.T. health** needs `smartmontools` and elevated I/O rights. Prefer granting capabilities to `smartctl` instead of running the GUI as root:
  `sudo setcap cap_sys_rawio,cap_sys_admin+ep "$(command -v smartctl)"`
* **Partition editing** opens the OS partition editor (GParted / GNOME Disks, Disk Utility, `diskmgmt.msc`), which handles privilege elevation and safety prompts.
* **Startup toggles** write only to your user's XDG autostart directory (Linux) or call `launchctl` (macOS). Windows registry entries are read-only.

## Known limits

* Per-application **network** throughput has no portable OS API; the column is wired but shows “—”. A platform back-end (eBPF/nethogs on Linux, ETW on Windows, `nettop` on macOS) is future work.
* **GPU telemetry** needs `nvidia-smi` (NVIDIA) or amdgpu sysfs (AMD on Linux). Intel/Apple GPUs show “not available”.
* **Disk speed** is the sum of per-process I/O deltas reported by the OS; kernel-level I/O outside processes is not included.
* Battery data is absent on desktops without a battery.
* Windows startup entries are listed but not toggled.

## Performance work (no visual redesign)

* Sampling runs on a background thread; the UI only reads an immutable snapshot and never blocks on hardware calls.
* `sysinfo` disk/component/network containers are created once and refreshed in place; full re-enumeration only every 60 s, temperatures/disks every 5 s, battery every 10 s, GPU per-process usage every 6 s.
* Process names are borrowed while aggregating, so one `String` per application is allocated instead of one per process per tick; I/O is tracked as per-PID deltas (also fixes a rate-calculation bug).
* Rendering: no-case-conversion sorting/filtering, one mesh for sparkline area fill instead of ~90 polygons per chart, and appearance/font application only when settings change.
* Storage analyser publishes `Arc` snapshots in batches of 16 and sorts once per batch, eliminating per-frame deep copies and O(n²) sort behaviour on huge folders.
* macOS startup scan calls `launchctl` once instead of once per LaunchAgent.
* Release profile: `opt-level = 3`, thin LTO, `codegen-units = 1`, `panic = "abort"`, stripped binary.

## Cross-platform binaries with GitHub Actions

`.github/workflows/build.yml` builds a release binary on Ubuntu, Windows and macOS and uploads them as artifacts. Push the repository to GitHub and check **Actions → build-native → Artifacts**. Building natively on each OS is strongly recommended over cross-compiling GUI apps.

Bundled fonts and licences: [`rust-app/assets/LICENSE.md`](rust-app/assets/LICENSE.md).

Detailed documentation: [`rust-app/README.md`](rust-app/README.md).
