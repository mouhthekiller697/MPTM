import { useState } from "react";
import { Card, Bar, Btn } from "../components/ui";
import { DISKS } from "./Disks";

type Node = { name: string; gb: number; kids?: Node[] };

const TREES: Record<string, Node> = {
  nvme0n1: {
    name: "/", gb: 589, kids: [
      { name: "home", gb: 301, kids: [
        { name: "Games", gb: 142, kids: [{ name: "SteamLibrary", gb: 120 }, { name: "Emulation", gb: 22 }] },
        { name: "Projects", gb: 58, kids: [{ name: "rust-targets", gb: 31 }, { name: "node_modules (all)", gb: 14 }, { name: "repos", gb: 13 }] },
        { name: "Videos", gb: 47 }, { name: "Pictures", gb: 28 }, { name: ".cache", gb: 17 }, { name: "Downloads", gb: 9 },
      ] },
      { name: "usr", gb: 62, kids: [{ name: "lib", gb: 38 }, { name: "share", gb: 14 }, { name: "bin", gb: 10 }] },
      { name: "var", gb: 48, kids: [{ name: "lib/docker", gb: 36 }, { name: "log", gb: 3 }, { name: "cache", gb: 9 }] },
      { name: "opt", gb: 41 }, { name: "swapfile", gb: 30 }, { name: "tmp", gb: 7 },
    ],
  },
  sda: {
    name: "/mnt", gb: 1230, kids: [
      { name: "media", gb: 1010, kids: [{ name: "Movies", gb: 560 }, { name: "Series", gb: 340 }, { name: "Music", gb: 110 }] },
      { name: "backup", gb: 220, kids: [{ name: "daily", gb: 90 }, { name: "weekly", gb: 130 }] },
    ],
  },
  sdb: { name: "/media/usb", gb: 41, kids: [{ name: "Photos", gb: 24 }, { name: "Documents", gb: 9 }, { name: "ISO", gb: 8 }] },
};

const APPS = [
  { n: "Steam Games", i: "🎮", gb: 120 }, { n: "Docker", i: "🐳", gb: 36 }, { n: "Blender", i: "🧊", gb: 14.2 }, { n: "Rust toolchains", i: "🦀", gb: 12.5 },
  { n: "Firefox", i: "🦊", gb: 6.1 }, { n: "Code Editor", i: "🧩", gb: 4.8 }, { n: "Discord", i: "💬", gb: 3.2 }, { n: "Spotify", i: "🎵", gb: 2.9 }, { n: "Cloud Sync", i: "☁️", gb: 2.1 }, { n: "Video Player", i: "🎬", gb: 1.4 },
];

const palette = ["#38bdf8", "#34d399", "#f472b6", "#fbbf24", "#a78bfa", "#fb7185", "#22d3ee", "#f97316"];

export default function Storage() {
  const [di, setDi] = useState(0);
  const [path, setPath] = useState<Node[]>([]);
  const [view, setView] = useState<"folders" | "apps">("folders");
  const [scan, setScan] = useState(false);
  const disk = DISKS[di];
  const root = TREES[disk.id];
  const cur = path.length ? path[path.length - 1] : root;
  const kids = [...(cur.kids ?? [])].sort((a, b) => b.gb - a.gb);
  const total = kids.reduce((a, k) => a + k.gb, 0) || 1;
  const rescan = () => { setScan(true); setTimeout(() => setScan(false), 1200); };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        {DISKS.map((x, i) => (
          <Btn key={x.id} active={i === di} onClick={() => { setDi(i); setPath([]); }}>{x.id} · {x.sizeGB} GB</Btn>
        ))}
        <div className="flex-1" />
        <Btn active={view === "folders"} onClick={() => setView("folders")}>Folders</Btn>
        <Btn active={view === "apps"} onClick={() => setView("apps")}>Apps</Btn>
        <Btn onClick={rescan}>{scan ? "Scanning…" : "Rescan"}</Btn>
      </div>

      {view === "folders" ? (
        <Card
          title={
            <span className="normal-case tracking-normal">
              <button className="cursor-pointer text-accent hover:underline" onClick={() => setPath([])}>{root.name}</button>
              {path.map((p, i) => (
                <span key={i}> / <button className="cursor-pointer text-accent hover:underline" onClick={() => setPath(path.slice(0, i + 1))}>{p.name}</button></span>
              ))}
            </span>
          }
          right={<span className="tnum text-sm font-semibold">{cur.gb} GB</span>}
        >
          {kids.length === 0 ? (
            <div className="py-10 text-center text-sm text-mute">No sub‑folders – this is a leaf directory.</div>
          ) : (
            <>
              {/* treemap strip */}
              <div className={`mb-4 flex h-16 gap-0.5 overflow-hidden rounded-lg ${scan ? "animate-pulse" : ""}`}>
                {kids.map((k, i) => (
                  <div key={k.name} title={`${k.name} – ${k.gb} GB`} style={{ width: `${(k.gb / total) * 100}%`, background: palette[i % palette.length] }} className="flex items-end overflow-hidden p-1 text-[0.65rem] font-semibold text-black/75">
                    <span className="truncate">{(k.gb / total) > 0.08 ? k.name : ""}</span>
                  </div>
                ))}
              </div>
              <ul className="space-y-1">
                {kids.map((k, i) => (
                  <li key={k.name}>
                    <button disabled={!k.kids} onClick={() => setPath([...path, k])} className={`flex w-full items-center gap-3 rounded-lg px-2 py-2 text-left text-sm ${k.kids ? "cursor-pointer hover:bg-panel2" : "cursor-default"}`}>
                      <span className="h-3 w-3 shrink-0 rounded-sm" style={{ background: palette[i % palette.length] }} />
                      <span className="w-44 shrink-0 truncate font-medium">{k.kids ? "📁" : "📄"} {k.name}</span>
                      <span className="flex-1"><Bar value={k.gb} max={kids[0].gb} color={palette[i % palette.length]} /></span>
                      <span className="tnum w-24 text-right text-mute">{k.gb} GB</span>
                      <span className="tnum w-12 text-right text-xs text-mute">{Math.round((k.gb / total) * 100)}%</span>
                    </button>
                  </li>
                ))}
              </ul>
            </>
          )}
        </Card>
      ) : (
        <Card title="Storage used by applications" right={<span className="tnum text-sm font-semibold">{APPS.reduce((a, b) => a + b.gb, 0).toFixed(1)} GB</span>}>
          <ul className="space-y-3">
            {APPS.map((a) => (
              <li key={a.n} className="flex items-center gap-3 text-sm">
                <span className="w-44 shrink-0">{a.i} {a.n}</span>
                <span className="flex-1"><Bar value={a.gb} max={APPS[0].gb} /></span>
                <span className="tnum w-20 text-right text-mute">{a.gb} GB</span>
              </li>
            ))}
          </ul>
        </Card>
      )}
    </div>
  );
}
