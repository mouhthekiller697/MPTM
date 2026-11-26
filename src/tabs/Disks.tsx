import { useState } from "react";
import { Card, Pill, Btn, Bar } from "../components/ui";

type Part = { name: string; fs: string; mount: string; sizeGB: number; usedGB: number; color: string };
type Disk = {
  id: string; model: string; kind: string; sizeGB: number; table: string; temp: number; hours: number; health: number; written: string; smart: [string, string, "good" | "warn" | "bad"][]; parts: Part[];
};

export const DISKS: Disk[] = [
  {
    id: "nvme0n1", model: "Samsung 980 PRO 1TB", kind: "NVMe SSD", sizeGB: 931, table: "GPT", temp: 41, hours: 6120, health: 96, written: "21.4 TB",
    smart: [["Available spare", "100%", "good"], ["Percentage used", "4%", "good"], ["Media errors", "0", "good"], ["Unsafe shutdowns", "12", "warn"], ["Power cycles", "812", "good"]],
    parts: [
      { name: "EFI System", fs: "FAT32", mount: "/boot/efi", sizeGB: 1, usedGB: 0.2, color: "#fbbf24" },
      { name: "System", fs: "ext4 / NTFS / APFS", mount: "/", sizeGB: 420, usedGB: 288, color: "#38bdf8" },
      { name: "Home / Data", fs: "ext4 / NTFS / APFS", mount: "/home", sizeGB: 480, usedGB: 301, color: "#34d399" },
      { name: "Swap / Recovery", fs: "swap", mount: "—", sizeGB: 30, usedGB: 3, color: "#f472b6" },
    ],
  },
  {
    id: "sda", model: "WD Blue 2TB", kind: "HDD 7200rpm", sizeGB: 1863, table: "GPT", temp: 36, hours: 24210, health: 78, written: "—",
    smart: [["Reallocated sectors", "8", "warn"], ["Pending sectors", "0", "good"], ["Spin retry count", "0", "good"], ["UDMA CRC errors", "0", "good"], ["Power-on hours", "24 210", "warn"]],
    parts: [
      { name: "Media", fs: "ext4 / NTFS / exFAT", mount: "/mnt/media", sizeGB: 1200, usedGB: 1010, color: "#a78bfa" },
      { name: "Backups", fs: "ext4 / NTFS / exFAT", mount: "/mnt/backup", sizeGB: 663, usedGB: 220, color: "#22d3ee" },
    ],
  },
  {
    id: "sdb", model: "SanDisk Ultra 64GB", kind: "USB flash", sizeGB: 59, table: "MBR", temp: 0, hours: 0, health: 100, written: "—",
    smart: [["SMART", "Not supported", "warn"]],
    parts: [{ name: "USB", fs: "exFAT", mount: "/media/usb", sizeGB: 59, usedGB: 41, color: "#fb7185" }],
  },
];

export default function Disks() {
  const [di, setDi] = useState(0);
  const [pi, setPi] = useState(0);
  const [toast, setToast] = useState("");
  const disk = DISKS[di];
  const part = disk.parts[pi] ?? disk.parts[0];
  const act = (a: string) => {
    setToast(`${a} “${part.name}” — demo only. The native Rust app asks for admin rights before changing anything.`);
    setTimeout(() => setToast(""), 3500);
  };
  const tone = disk.health > 85 ? "good" : disk.health > 60 ? "warn" : "bad";

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-3">
        {DISKS.map((x, i) => (
          <button key={x.id} onClick={() => { setDi(i); setPi(0); }} className={`cursor-pointer border p-4 text-left transition ${i === di ? "border-accent bg-accent/10" : "border-line bg-panel hover:border-mute"}`} style={{ borderRadius: "var(--radius)" }}>
            <div className="flex items-center justify-between">
              <span className="font-semibold">{x.model}</span>
              <Pill tone={x.health > 85 ? "good" : x.health > 60 ? "warn" : "bad"}>{x.health}%</Pill>
            </div>
            <div className="mt-1 text-xs text-mute">/dev/{x.id} · {x.kind} · {x.sizeGB} GB</div>
          </button>
        ))}
      </div>

      <Card title="Partition layout" right={<Pill tone="mute">{disk.table} partition table</Pill>}>
        <div className="flex h-12 overflow-hidden rounded-lg border border-line">
          {disk.parts.map((p, i) => (
            <button key={p.name} onClick={() => setPi(i)} title={p.name} className="relative cursor-pointer text-xs font-semibold text-black/80 transition hover:brightness-110" style={{ width: `${Math.max(3, (p.sizeGB / disk.sizeGB) * 100)}%`, background: p.color, outline: i === pi ? "3px solid var(--fg)" : "none", outlineOffset: "-3px" }}>
              <span className="truncate px-1">{p.sizeGB / disk.sizeGB > 0.1 ? p.name : ""}</span>
            </button>
          ))}
        </div>
        <div className="mt-4 overflow-x-auto">
          <table className="w-full min-w-[560px] text-sm">
            <thead className="text-left text-xs uppercase text-mute">
              <tr><th className="py-1">Partition</th><th>Filesystem</th><th>Mount</th><th>Size</th><th className="w-48">Used</th></tr>
            </thead>
            <tbody className="tnum">
              {disk.parts.map((p, i) => (
                <tr key={p.name} onClick={() => setPi(i)} className={`cursor-pointer border-t border-line/60 ${i === pi ? "bg-accent/10" : "hover:bg-panel2"}`}>
                  <td className="py-2"><span className="mr-2 inline-block h-3 w-3 rounded-sm align-middle" style={{ background: p.color }} />{p.name}</td>
                  <td className="text-mute">{p.fs}</td>
                  <td className="text-mute">{p.mount}</td>
                  <td>{p.sizeGB} GB</td>
                  <td><Bar value={p.usedGB} max={p.sizeGB} /><span className="text-xs text-mute">{p.usedGB} GB · {Math.round((p.usedGB / p.sizeGB) * 100)}%</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="mt-4 flex flex-wrap gap-2">
          {["Mount", "Unmount", "Resize", "Format", "Check filesystem", "New partition"].map((a) => (
            <Btn key={a} danger={a === "Format"} onClick={() => act(a)}>{a}</Btn>
          ))}
        </div>
        {toast && <div className="mt-3 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-sm">{toast}</div>}
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        <Card title="Health (S.M.A.R.T.)" right={<Pill tone={tone}>{tone === "good" ? "Healthy" : tone === "warn" ? "Caution" : "Failing"}</Pill>}>
          <ul className="divide-y divide-line/60 text-sm">
            {disk.smart.map(([k, v, t]) => (
              <li key={k} className="flex items-center justify-between py-2"><span className="text-mute">{k}</span><Pill tone={t}>{v}</Pill></li>
            ))}
          </ul>
        </Card>
        <Card title="Drive info">
          <dl className="grid grid-cols-2 gap-y-3 text-sm">
            {[["Model", disk.model], ["Type", disk.kind], ["Capacity", `${disk.sizeGB} GB`], ["Temperature", disk.temp ? `${disk.temp}°C` : "n/a"], ["Power‑on", disk.hours ? `${disk.hours.toLocaleString()} h` : "n/a"], ["Data written", disk.written]].map(([k, v]) => (
              <div key={k}><dt className="text-xs uppercase text-mute">{k}</dt><dd className="tnum font-medium">{v}</dd></div>
            ))}
          </dl>
        </Card>
      </div>
    </div>
  );
}
