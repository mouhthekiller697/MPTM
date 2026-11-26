import type { ReactNode } from "react";

export function Card({ title, right, children, className = "" }: { title?: ReactNode; right?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <section className={`border border-line bg-panel p-4 ${className}`} style={{ borderRadius: "var(--radius)" }}>
      {(title || right) && (
        <header className="mb-3 flex items-center justify-between gap-2">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-mute">{title}</h3>
          {right}
        </header>
      )}
      {children}
    </section>
  );
}

export function Spark({ data, max = 100, color = "var(--accent)", height = 56, data2, color2 = "#f472b6" }: { data: number[]; max?: number; color?: string; height?: number; data2?: number[]; color2?: string }) {
  const w = 300;
  const h = height;
  const path = (d: number[]) => d.map((v, i) => `${i === 0 ? "M" : "L"}${((i / (d.length - 1)) * w).toFixed(1)},${(h - (Math.min(v, max) / max) * (h - 4) - 2).toFixed(1)}`).join(" ");
  const id = `g${color.replace(/\W/g, "")}${height}`;
  return (
    <svg viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" className="w-full" style={{ height }}>
      <defs>
        <linearGradient id={id} x1="0" x2="0" y1="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.35" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      {[0.25, 0.5, 0.75].map((g) => (
        <line key={g} x1="0" x2={w} y1={h * g} y2={h * g} stroke="var(--line)" strokeWidth="1" strokeDasharray="3 4" vectorEffect="non-scaling-stroke" />
      ))}
      <path d={`${path(data)} L${w},${h} L0,${h} Z`} fill={`url(#${id})`} />
      <path d={path(data)} fill="none" stroke={color} strokeWidth="2" vectorEffect="non-scaling-stroke" strokeLinejoin="round" />
      {data2 && <path d={path(data2)} fill="none" stroke={color2} strokeWidth="2" vectorEffect="non-scaling-stroke" strokeLinejoin="round" />}
    </svg>
  );
}

export function Ring({ value, size = 64, color = "var(--accent)", label }: { value: number; size?: number; color?: string; label?: string }) {
  const r = size / 2 - 6;
  const c = 2 * Math.PI * r;
  return (
    <svg width={size} height={size} className="shrink-0 -rotate-90">
      <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--line)" strokeWidth="6" />
      <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke={color} strokeWidth="6" strokeLinecap="round" strokeDasharray={c} strokeDashoffset={c * (1 - Math.min(100, value) / 100)} style={{ transition: "stroke-dashoffset .6s" }} />
      <text x="50%" y="50%" dominantBaseline="central" textAnchor="middle" className="rotate-90 origin-center fill-fg tnum" style={{ fontSize: size * 0.24, fontWeight: 600, transformBox: "fill-box" }}>
        {label ?? `${Math.round(value)}%`}
      </text>
    </svg>
  );
}

export function Bar({ value, max = 100, color = "var(--accent)", className = "" }: { value: number; max?: number; color?: string; className?: string }) {
  const pct = Math.min(100, (value / max) * 100);
  const col = pct > 90 ? "#f43f5e" : pct > 75 ? "#f59e0b" : color;
  return (
    <div className={`h-2 w-full overflow-hidden rounded-full bg-panel2 ${className}`}>
      <div className="h-full rounded-full" style={{ width: `${pct}%`, background: col, transition: "width .5s" }} />
    </div>
  );
}

export function Stat({ label, value, sub }: { label: string; value: ReactNode; sub?: ReactNode }) {
  return (
    <div>
      <div className="text-[0.7rem] uppercase tracking-wide text-mute">{label}</div>
      <div className="tnum text-xl font-semibold">{value}</div>
      {sub && <div className="tnum text-xs text-mute">{sub}</div>}
    </div>
  );
}

export function Pill({ children, tone = "accent" }: { children: ReactNode; tone?: "accent" | "good" | "warn" | "bad" | "mute" }) {
  const tones = {
    accent: "bg-accent/15 text-accent",
    good: "bg-emerald-500/15 text-emerald-500",
    warn: "bg-amber-500/15 text-amber-500",
    bad: "bg-rose-500/15 text-rose-500",
    mute: "bg-panel2 text-mute",
  };
  return <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${tones[tone]}`}>{children}</span>;
}

export function Btn({ children, onClick, active, danger }: { children: ReactNode; onClick?: () => void; active?: boolean; danger?: boolean }) {
  return (
    <button
      onClick={onClick}
      className={`cursor-pointer border px-3 py-1.5 text-sm font-medium transition ${
        active ? "border-accent bg-accent text-black" : danger ? "border-rose-500/40 text-rose-500 hover:bg-rose-500/10" : "border-line bg-panel2 text-fg hover:border-accent"
      }`}
      style={{ borderRadius: "calc(var(--radius) * 0.6)" }}
    >
      {children}
    </button>
  );
}
