import { startTransition, useEffect, useRef, useState } from "react";

export const HISTORY = 60;

export type AppProc = {
  pid: number;
  name: string;
  icon: string;
  cpu: number;
  ramMB: number;
  gpu: number;
  netKBs: number;
  diskMBs: number;
  startup: boolean;
};

export type Snapshot = {
  cpu: number[];
  gpu: number[];
  ram: number[];
  vram: number[];
  diskUse: number[];
  diskRead: number[];
  diskWrite: number[];
  net: number[];
  cores: number[];
  battery: { level: number; charging: boolean; minutes: number; health: number };
  procs: AppProc[];
  uptime: number;
  temp: { cpu: number; gpu: number };
};

export const TOTALS = {
  ramGB: 32,
  vramGB: 12,
  diskMaxMBs: 3500,
  netMaxKBs: 50000,
};

type Base = Omit<AppProc, "cpu" | "ramMB" | "gpu" | "netKBs" | "diskMBs"> & {
  c: number;
  r: number;
  g: number;
  n: number;
  d: number;
};

const BASES: Base[] = [
  { pid: 1204, name: "Firefox", icon: "🦊", c: 14, r: 2400, g: 9, n: 1800, d: 3, startup: false },
  { pid: 2281, name: "Chromium", icon: "🌐", c: 9, r: 1900, g: 6, n: 900, d: 2, startup: false },
  { pid: 3320, name: "Code Editor", icon: "🧩", c: 6, r: 1300, g: 2, n: 30, d: 4, startup: true },
  { pid: 4410, name: "Steam", icon: "🎮", c: 3, r: 620, g: 4, n: 4200, d: 12, startup: true },
  { pid: 5121, name: "Blender", icon: "🧊", c: 38, r: 4200, g: 54, n: 2, d: 18, startup: false },
  { pid: 6102, name: "Discord", icon: "💬", c: 4, r: 780, g: 3, n: 240, d: 1, startup: true },
  { pid: 7022, name: "Spotify", icon: "🎵", c: 2, r: 420, g: 1, n: 340, d: 1, startup: true },
  { pid: 8081, name: "Docker", icon: "🐳", c: 7, r: 2100, g: 0, n: 150, d: 9, startup: true },
  { pid: 9101, name: "Terminal", icon: "⌨️", c: 1, r: 90, g: 0, n: 0, d: 0, startup: false },
  { pid: 1033, name: "Rust Compiler", icon: "🦀", c: 52, r: 1700, g: 0, n: 12, d: 85, startup: false },
  { pid: 1190, name: "Backup Agent", icon: "💾", c: 5, r: 210, g: 0, n: 2600, d: 45, startup: true },
  { pid: 1244, name: "System Services", icon: "⚙️", c: 3, r: 540, g: 1, n: 60, d: 2, startup: true },
  { pid: 1301, name: "Video Player", icon: "🎬", c: 11, r: 540, g: 22, n: 1200, d: 6, startup: false },
  { pid: 1402, name: "Cloud Sync", icon: "☁️", c: 2, r: 260, g: 0, n: 800, d: 7, startup: true },
];

const clamp = (v: number, a = 0, b = 100) => Math.max(a, Math.min(b, v));
const walk = (prev: number, target: number, vol: number, min = 0, max = 100) =>
  clamp(prev + (target - prev) * 0.15 + (Math.random() - 0.5) * vol, min, max);
const push = (arr: number[], v: number) => [...arr.slice(-(HISTORY - 1)), v];
const seed = (v: number) => Array.from({ length: HISTORY }, () => v);

export function useSim(paused = false): Snapshot {
  const state = useRef<Snapshot>(null as unknown as Snapshot);
  if (!state.current) {
    state.current = {
      cpu: seed(25),
      gpu: seed(18),
      ram: seed(52),
      vram: seed(35),
      diskUse: seed(12),
      diskRead: seed(40),
      diskWrite: seed(20),
      net: seed(900),
      cores: Array.from({ length: 16 }, () => 20),
      battery: { level: 78, charging: false, minutes: 245, health: 91 },
      procs: BASES.map((b) => ({ ...b, cpu: b.c, ramMB: b.r, gpu: b.g, netKBs: b.n, diskMBs: b.d })),
      uptime: 3 * 3600 + 1240,
      temp: { cpu: 52, gpu: 48 },
    };
  }
  const [, force] = useState(0);
  // Don't burn CPU while the window/tab is in the background.
  const [visible, setVisible] = useState(() => typeof document === "undefined" || !document.hidden);

  useEffect(() => {
    const onVisibility = () => setVisible(!document.hidden);
    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, []);

  useEffect(() => {
    if (paused || !visible) return;
    const id = setInterval(() => {
      const s = state.current;
      const procs = s.procs.map((p, i) => {
        const b = BASES[i];
        const burst = Math.random() < 0.04 ? 2.2 : 1;
        return {
          ...p,
          cpu: walk(p.cpu, b.c * burst, b.c * 0.5 + 1, 0, 100),
          ramMB: walk(p.ramMB, b.r, b.r * 0.01, 10, 20000),
          gpu: walk(p.gpu, b.g * burst, b.g * 0.4 + 0.3, 0, 100),
          netKBs: walk(p.netKBs, b.n * burst, b.n * 0.6 + 1, 0, 60000),
          diskMBs: walk(p.diskMBs, b.d * burst, b.d * 0.6 + 0.2, 0, 3500),
        };
      });
      const cpu = clamp(procs.reduce((a, p) => a + p.cpu, 0) / 3.2 + 4);
      const gpu = clamp(procs.reduce((a, p) => a + p.gpu, 0) * 0.8 + 3);
      const ramGB = procs.reduce((a, p) => a + p.ramMB, 0) / 1024 + 6.5;
      const read = clamp(procs.reduce((a, p) => a + p.diskMBs, 0) * 0.6, 0, TOTALS.diskMaxMBs);
      const write = clamp(procs.reduce((a, p) => a + p.diskMBs, 0) * 0.3, 0, TOTALS.diskMaxMBs);
      const net = procs.reduce((a, p) => a + p.netKBs, 0);
      state.current = {
        ...s,
        procs,
        cpu: push(s.cpu, cpu),
        gpu: push(s.gpu, gpu),
        ram: push(s.ram, clamp((ramGB / TOTALS.ramGB) * 100)),
        vram: push(s.vram, clamp(gpu * 0.55 + 20)),
        diskUse: push(s.diskUse, clamp(((read + write) / 600) * 100)),
        diskRead: push(s.diskRead, read),
        diskWrite: push(s.diskWrite, write),
        net: push(s.net, net),
        cores: s.cores.map((c) => walk(c, cpu, 18)),
        battery: {
          ...s.battery,
          level: clamp(s.battery.level - 0.01),
          minutes: Math.max(0, s.battery.minutes - 0.01),
        },
        uptime: s.uptime + 1,
        temp: { cpu: clamp(35 + cpu * 0.5, 0, 110), gpu: clamp(33 + gpu * 0.45, 0, 110) },
      };
      // Mark the tick as non-urgent so typing/filtering stays responsive.
      startTransition(() => force((n) => n + 1));
    }, 1000);
    return () => clearInterval(id);
  }, [paused, visible]);

  return state.current;
}

export const fmtBytes = (b: number) => {
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (b >= 1024 && i < u.length - 1) {
    b /= 1024;
    i++;
  }
  return `${b.toFixed(b >= 100 || i === 0 ? 0 : 1)} ${u[i]}`;
};

export const fmtDur = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return `${h}h ${m.toString().padStart(2, "0")}m`;
};
