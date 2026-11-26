import { memo, useState, type ReactNode } from "react";
import { SettingsProvider, useSettings } from "./settings";
import { useSim, fmtDur } from "./sim";
import Overview from "./tabs/Overview";
import Apps from "./tabs/Apps";
import Disks from "./tabs/Disks";
import Storage from "./tabs/Storage";
import Specs from "./tabs/Specs";
import Startup from "./tabs/Startup";
import SettingsTab from "./tabs/SettingsTab";

const I = (d: string): ReactNode => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"><path d={d} /></svg>
);

// These tabs don't consume live samples, so memoisation stops them re-rendering
// every second while the user is interacting with them.
const StaticDisks = memo(Disks);
const StaticStorage = memo(Storage);
const StaticStartup = memo(Startup);
const StaticSettings = memo(SettingsTab);

const TABS = [
  { id: "general", label: "General", icon: I("M3 12h4l3-8 4 16 3-8h4") },
  { id: "apps", label: "Applications", icon: I("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z") },
  { id: "disks", label: "Disk Manager", icon: I("M3 7c0-1.7 4-3 9-3s9 1.3 9 3-4 3-9 3-9-1.3-9-3zM3 7v10c0 1.7 4 3 9 3s9-1.3 9-3V7") },
  { id: "storage", label: "Storage Analyser", icon: I("M21 12A9 9 0 1 1 12 3v9zM15 3.5A9 9 0 0 1 20.5 9H15z") },
  { id: "specs", label: "Specs", icon: I("M6 6h12v12H6zM9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4") },
  { id: "startup", label: "Startup", icon: I("M12 2v10M6.3 6.3a8 8 0 1 0 11.4 0") },
  { id: "settings", label: "Appearance", icon: I("M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z") },
] as const;

function Shell() {
  const [tab, setTab] = useState<(typeof TABS)[number]["id"]>("general");
  const [paused, setPaused] = useState(false);
  const d = useSim(paused);
  const { s } = useSettings();
  const cur = TABS.find((t) => t.id === tab)!;
  const pad = s.compact ? "p-3 md:p-4" : "p-4 md:p-6";

  return (
    <div className="flex min-h-screen flex-col md:flex-row">
      <aside className="sticky top-0 z-10 flex shrink-0 gap-1 overflow-x-auto border-b border-line bg-panel p-2 md:h-screen md:w-60 md:flex-col md:border-b-0 md:border-r md:p-3">
        <div className="hidden items-center gap-2 px-2 pb-4 pt-2 md:flex">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-accent font-black text-black">Fe</div>
          <div>
            <div className="font-bold leading-tight">Ferrum Monitor</div>
            <div className="text-xs text-mute">System task manager</div>
          </div>
        </div>
        {TABS.map((t) => (
          <button key={t.id} onClick={() => setTab(t.id)} className={`flex shrink-0 cursor-pointer items-center gap-3 px-3 py-2 text-sm font-medium transition ${tab === t.id ? "bg-accent text-black" : "text-mute hover:bg-panel2 hover:text-fg"}`} style={{ borderRadius: "calc(var(--radius) * 0.65)" }}>
            {t.icon}
            <span className="whitespace-nowrap">{t.label}</span>
          </button>
        ))}
        <div className="mt-auto hidden space-y-1 px-2 pt-4 text-xs text-mute md:block">
          <div className="tnum">Uptime {fmtDur(d.uptime)}</div>
          <div className="tnum">CPU {d.cpu.at(-1)!.toFixed(0)}% · RAM {d.ram.at(-1)!.toFixed(0)}%</div>
        </div>
      </aside>

      <main className={`min-w-0 flex-1 ${pad}`}>
        <header className="mb-4 flex flex-wrap items-center gap-3">
          <h1 className="text-2xl font-bold">{cur.label}</h1>
          <div className="flex-1" />
          <button onClick={() => setPaused(!paused)} className="flex cursor-pointer items-center gap-2 border border-line bg-panel px-3 py-1.5 text-sm hover:border-accent" style={{ borderRadius: "calc(var(--radius) * 0.6)" }}>
            <span className={`h-2 w-2 rounded-full ${paused ? "bg-amber-500" : "animate-pulse bg-emerald-500"}`} />
            {paused ? "Paused" : "Live · 1 s"}
          </button>
        </header>

        <div className="mb-4 border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-500" style={{ borderRadius: "calc(var(--radius) * 0.6)" }}>
          Interactive preview with <b>simulated data</b> — a browser can't read your hardware. The real, native Rust app (egui + sysinfo) is in the <code>rust-app/</code> folder.
        </div>

        {tab === "general" && <Overview d={d} />}
        {tab === "apps" && <Apps d={d} />}
        {tab === "disks" && <StaticDisks />}
        {tab === "storage" && <StaticStorage />}
        {tab === "specs" && <Specs d={d} />}
        {tab === "startup" && <StaticStartup />}
        {tab === "settings" && <StaticSettings />}
      </main>
    </div>
  );
}

export default function App() {
  return (
    <SettingsProvider>
      <Shell />
    </SettingsProvider>
  );
}
