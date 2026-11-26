import { useMemo, useState } from "react";
import { Card, Bar, Btn } from "../components/ui";
import type { AppProc, Snapshot } from "../sim";

type Key = "name" | "cpu" | "ramMB" | "gpu" | "netKBs" | "diskMBs";

const heat = (v: number, max: number) => ({ background: `color-mix(in srgb, var(--accent) ${Math.min(55, (v / max) * 100 * 0.55)}%, transparent)` });

export default function Apps({ d }: { d: Snapshot }) {
  const [key, setKey] = useState<Key>("cpu");
  const [asc, setAsc] = useState(false);
  const [q, setQ] = useState("");
  const [ended, setEnded] = useState<number[]>([]);
  const [sel, setSel] = useState<number | null>(null);

  const rows = useMemo(() => {
    const r = d.procs.filter((p) => !ended.includes(p.pid) && p.name.toLowerCase().includes(q.toLowerCase()));
    return r.sort((a: AppProc, b: AppProc) => {
      const x = key === "name" ? a.name.localeCompare(b.name) : (a[key] as number) - (b[key] as number);
      return asc ? x : -x;
    });
  }, [d.procs, key, asc, q, ended]);

  const tot = (k: keyof AppProc) => rows.reduce((a, p) => a + (p[k] as number), 0);

  const H = ({ k, children, right = true }: { k: Key; children: string; right?: boolean }) => (
    <th className={`cursor-pointer select-none px-3 py-2 font-semibold hover:text-fg ${right ? "text-right" : "text-left"}`} onClick={() => (key === k ? setAsc(!asc) : (setKey(k), setAsc(false)))}>
      {children} {key === k ? (asc ? "▲" : "▼") : ""}
    </th>
  );

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search applications…" className="w-64 border border-line bg-panel px-3 py-2 text-sm outline-none focus:border-accent" style={{ borderRadius: "calc(var(--radius) * 0.6)" }} />
        <div className="flex-1" />
        <Btn danger onClick={() => { if (sel) { setEnded([...ended, sel]); setSel(null); } }}>End task</Btn>
      </div>
      <Card className="overflow-x-auto !p-0">
        <table className="w-full min-w-[760px] text-sm">
          <thead className="border-b border-line text-xs uppercase tracking-wide text-mute">
            <tr>
              <H k="name" right={false}>Application</H>
              <H k="cpu">CPU</H>
              <H k="ramMB">Memory</H>
              <H k="gpu">GPU</H>
              <H k="netKBs">Network</H>
              <H k="diskMBs">Disk</H>
            </tr>
          </thead>
          <tbody className="tnum">
            {rows.map((p) => (
              <tr key={p.pid} onClick={() => setSel(p.pid)} className={`cursor-pointer border-b border-line/60 ${sel === p.pid ? "bg-accent/10" : "hover:bg-panel2"}`}>
                <td className="px-3 py-2">
                  <span className="mr-2">{p.icon}</span>
                  <span className="font-medium">{p.name}</span>
                  <span className="ml-2 text-xs text-mute">PID {p.pid}</span>
                </td>
                <td className="px-3 py-2 text-right" style={heat(p.cpu, 60)}>{p.cpu.toFixed(1)}%</td>
                <td className="px-3 py-2 text-right" style={heat(p.ramMB, 4000)}>{p.ramMB >= 1024 ? `${(p.ramMB / 1024).toFixed(2)} GB` : `${p.ramMB.toFixed(0)} MB`}</td>
                <td className="px-3 py-2 text-right" style={heat(p.gpu, 60)}>{p.gpu.toFixed(1)}%</td>
                <td className="px-3 py-2 text-right" style={heat(p.netKBs, 4000)}>{p.netKBs >= 1024 ? `${(p.netKBs / 1024).toFixed(2)} MB/s` : `${p.netKBs.toFixed(0)} KB/s`}</td>
                <td className="px-3 py-2 text-right" style={heat(p.diskMBs, 80)}>{p.diskMBs.toFixed(1)} MB/s</td>
              </tr>
            ))}
          </tbody>
          <tfoot className="tnum text-xs font-semibold text-mute">
            <tr>
              <td className="px-3 py-2">{rows.length} applications</td>
              <td className="px-3 py-2 text-right">{tot("cpu").toFixed(0)}%</td>
              <td className="px-3 py-2 text-right">{(tot("ramMB") / 1024).toFixed(1)} GB</td>
              <td className="px-3 py-2 text-right">{tot("gpu").toFixed(0)}%</td>
              <td className="px-3 py-2 text-right">{(tot("netKBs") / 1024).toFixed(1)} MB/s</td>
              <td className="px-3 py-2 text-right">{tot("diskMBs").toFixed(0)} MB/s</td>
            </tr>
          </tfoot>
        </table>
      </Card>
      <Card title="Top consumers">
        <div className="grid gap-4 md:grid-cols-3">
          {(["cpu", "ramMB", "netKBs"] as const).map((k) => {
            const top = [...rows].sort((a, b) => b[k] - a[k]).slice(0, 4);
            const mx = top[0]?.[k] || 1;
            return (
              <div key={k} className="space-y-2">
                <div className="text-xs uppercase text-mute">{k === "cpu" ? "CPU" : k === "ramMB" ? "Memory" : "Network"}</div>
                {top.map((p) => (
                  <div key={p.pid}>
                    <div className="mb-1 flex justify-between text-xs"><span>{p.icon} {p.name}</span></div>
                    <Bar value={p[k]} max={mx} />
                  </div>
                ))}
              </div>
            );
          })}
        </div>
      </Card>
    </div>
  );
}
