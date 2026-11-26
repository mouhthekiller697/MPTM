import { useState } from "react";
import { Card, Pill } from "../components/ui";

type Item = { name: string; icon: string; publisher: string; where: string; impact: "High" | "Medium" | "Low"; ms: number; on: boolean };

const INIT: Item[] = [
  { name: "Steam", icon: "🎮", publisher: "Valve", where: "~/.config/autostart · HKCU\\Run · LaunchAgents", impact: "High", ms: 2400, on: true },
  { name: "Docker Desktop", icon: "🐳", publisher: "Docker Inc.", where: "systemd user service", impact: "High", ms: 3100, on: true },
  { name: "Discord", icon: "💬", publisher: "Discord Inc.", where: "~/.config/autostart", impact: "Medium", ms: 1300, on: true },
  { name: "Spotify", icon: "🎵", publisher: "Spotify AB", where: "~/.config/autostart", impact: "Medium", ms: 900, on: true },
  { name: "Cloud Sync", icon: "☁️", publisher: "Nextcloud GmbH", where: "/etc/xdg/autostart", impact: "Medium", ms: 700, on: true },
  { name: "Backup Agent", icon: "💾", publisher: "Restic", where: "cron @reboot", impact: "Low", ms: 300, on: true },
  { name: "Bluetooth Applet", icon: "📶", publisher: "BlueZ", where: "/etc/xdg/autostart", impact: "Low", ms: 120, on: true },
  { name: "Code Editor", icon: "🧩", publisher: "Microsoft", where: "~/.config/autostart", impact: "Low", ms: 200, on: false },
  { name: "Screenshot Tool", icon: "📸", publisher: "Flameshot", where: "~/.config/autostart", impact: "Low", ms: 150, on: false },
];

const tone = { High: "bad", Medium: "warn", Low: "good" } as const;

export default function Startup() {
  const [items, setItems] = useState(INIT);
  const toggle = (n: string) => setItems(items.map((i) => (i.name === n ? { ...i, on: !i.on } : i)));
  const enabled = items.filter((i) => i.on);
  const boot = enabled.reduce((a, i) => a + i.ms, 0);
  return (
    <div className="space-y-4">
      <div className="grid gap-4 sm:grid-cols-3">
        <Card><div className="text-xs uppercase text-mute">Enabled</div><div className="tnum text-3xl font-semibold">{enabled.length}<span className="text-base text-mute"> / {items.length}</span></div></Card>
        <Card><div className="text-xs uppercase text-mute">Added boot time</div><div className="tnum text-3xl font-semibold">{(boot / 1000).toFixed(1)} s</div></Card>
        <Card><div className="text-xs uppercase text-mute">High impact</div><div className="tnum text-3xl font-semibold text-rose-500">{enabled.filter((i) => i.impact === "High").length}</div></Card>
      </div>
      <Card title="Startup applications" className="!p-0">
        <ul>
          {items.map((i) => (
            <li key={i.name} className="flex items-center gap-4 border-b border-line/60 px-4 py-3 last:border-0">
              <span className="text-2xl">{i.icon}</span>
              <div className="min-w-0 flex-1">
                <div className="font-medium">{i.name}</div>
                <div className="truncate text-xs text-mute">{i.publisher} · {i.where}</div>
              </div>
              <span className="tnum hidden text-sm text-mute sm:block">{i.ms} ms</span>
              <Pill tone={tone[i.impact]}>{i.impact}</Pill>
              <button role="switch" aria-checked={i.on} onClick={() => toggle(i.name)} className="relative h-6 w-11 shrink-0 cursor-pointer rounded-full transition" style={{ background: i.on ? "var(--accent)" : "var(--line)" }}>
                <span className="absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-all" style={{ left: i.on ? "22px" : "2px" }} />
              </button>
            </li>
          ))}
        </ul>
      </Card>
    </div>
  );
}
