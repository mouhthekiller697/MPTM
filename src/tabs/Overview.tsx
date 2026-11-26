import { Card, Spark, Ring, Bar, Pill } from "../components/ui";
import { TOTALS, fmtDur, type Snapshot } from "../sim";

const last = (a: number[]) => a[a.length - 1];

function Metric({ title, value, sub, data, color, ring, max = 100, extra }: { title: string; value: number; sub: string; data: number[]; color: string; ring?: string; max?: number; extra?: React.ReactNode }) {
  return (
    <Card title={title} right={extra}>
      <div className="flex items-center gap-4">
        <Ring value={(value / max) * 100} color={color} label={ring} size={76} />
        <div className="min-w-0 flex-1">
          <div className="tnum text-sm text-mute">{sub}</div>
          <Spark data={data} max={max} color={color} height={52} />
        </div>
      </div>
    </Card>
  );
}

export default function Overview({ d }: { d: Snapshot }) {
  const cpu = last(d.cpu), gpu = last(d.gpu), ram = last(d.ram), vram = last(d.vram);
  const rd = last(d.diskRead), wr = last(d.diskWrite);
  const net = last(d.net);
  const b = d.battery;
  return (
    <div className="space-y-4">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        <Metric title="CPU" value={cpu} color="#38bdf8" sub={`${d.cores.length} threads · ${(3.2 + cpu / 60).toFixed(2)} GHz · ${d.temp.cpu.toFixed(0)}°C`} data={d.cpu} />
        <Metric title="GPU" value={gpu} color="#a78bfa" sub={`RTX-class · ${d.temp.gpu.toFixed(0)}°C · ${(60 + gpu * 1.2).toFixed(0)} W`} data={d.gpu} />
        <Metric title="Memory (RAM)" value={ram} color="#34d399" sub={`${((ram / 100) * TOTALS.ramGB).toFixed(1)} / ${TOTALS.ramGB} GB in use`} data={d.ram} />
        <Metric title="GPU Memory (VRAM)" value={vram} color="#f472b6" sub={`${((vram / 100) * TOTALS.vramGB).toFixed(1)} / ${TOTALS.vramGB} GB in use`} data={d.vram} />
        <Metric title="Disk activity" value={last(d.diskUse)} color="#fbbf24" sub="NVMe · active time" data={d.diskUse} />
        <Card title="Disk speed">
          <div className="grid grid-cols-2 gap-3">
            <div>
              <div className="text-xs text-mute">Read</div>
              <div className="tnum text-2xl font-semibold text-[#38bdf8]">{rd.toFixed(0)} <span className="text-sm text-mute">MB/s</span></div>
            </div>
            <div>
              <div className="text-xs text-mute">Write</div>
              <div className="tnum text-2xl font-semibold text-[#f472b6]">{wr.toFixed(0)} <span className="text-sm text-mute">MB/s</span></div>
            </div>
          </div>
          <Spark data={d.diskRead} data2={d.diskWrite} max={Math.max(200, ...d.diskRead, ...d.diskWrite)} color="#38bdf8" color2="#f472b6" height={52} />
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card title="Network" right={<Pill tone="mute">Wi‑Fi / Ethernet</Pill>}>
          <div className="tnum mb-1 text-2xl font-semibold">{(net / 1024).toFixed(2)} <span className="text-sm text-mute">MB/s total</span></div>
          <Spark data={d.net} max={Math.max(2000, ...d.net)} color="#22d3ee" height={60} />
        </Card>
        <Card title="Battery" right={<Pill tone={b.charging ? "good" : b.level < 20 ? "bad" : "mute"}>{b.charging ? "Charging" : "On battery"}</Pill>}>
          <div className="flex items-center gap-5">
            <Ring value={b.level} color={b.level < 20 ? "#f43f5e" : "#34d399"} size={84} />
            <div className="flex-1 space-y-2 text-sm">
              <Bar value={b.level} max={100} color="#34d399" />
              <div className="flex justify-between text-mute"><span>Time remaining</span><span className="tnum text-fg">{fmtDur(b.minutes * 60)}</span></div>
              <div className="flex justify-between text-mute"><span>Health</span><span className="tnum text-fg">{b.health}%</span></div>
              <div className="flex justify-between text-mute"><span>Uptime</span><span className="tnum text-fg">{fmtDur(d.uptime)}</span></div>
            </div>
          </div>
        </Card>
      </div>

      <Card title="Per‑thread CPU load">
        <div className="grid grid-cols-4 gap-3 sm:grid-cols-8">
          {d.cores.map((c, i) => (
            <div key={i}>
              <div className="flex h-14 items-end overflow-hidden rounded-md bg-panel2">
                <div className="w-full bg-accent/80" style={{ height: `${c}%`, transition: "height .6s" }} />
              </div>
              <div className="tnum mt-1 text-center text-[0.7rem] text-mute">T{i}</div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
