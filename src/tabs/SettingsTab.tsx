import { Card, Btn } from "../components/ui";
import { ACCENTS, FONTS, THEMES, useSettings } from "../settings";

export default function SettingsTab() {
  const { s, set, reset } = useSettings();
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <Card title="Theme">
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
          {THEMES.map((t) => (
            <button key={t.id} onClick={() => set({ theme: t.id })} className={`cursor-pointer border p-2 text-left text-sm transition ${s.theme === t.id ? "border-accent" : "border-line hover:border-mute"}`} style={{ borderRadius: "calc(var(--radius) * 0.7)", background: t.vars["--bg"], color: t.vars["--fg"] }}>
              <div className="mb-2 flex gap-1">
                {["--panel", "--panel2", "--line"].map((k) => <span key={k} className="h-4 flex-1 rounded" style={{ background: t.vars[k] }} />)}
                <span className="h-4 flex-1 rounded" style={{ background: s.accent }} />
              </div>
              {t.name}
            </button>
          ))}
        </div>
      </Card>

      <Card title="Accent colour">
        <div className="flex flex-wrap items-center gap-3">
          {ACCENTS.map((c) => (
            <button key={c} aria-label={c} onClick={() => set({ accent: c })} className="h-9 w-9 cursor-pointer rounded-full transition hover:scale-110" style={{ background: c, outline: s.accent === c ? "3px solid var(--fg)" : "none", outlineOffset: 2 }} />
          ))}
          <label className="flex cursor-pointer items-center gap-2 text-sm text-mute">
            Custom
            <input type="color" value={s.accent} onChange={(e) => set({ accent: e.target.value })} className="h-9 w-12 cursor-pointer rounded border border-line bg-transparent" />
          </label>
        </div>
        <div className="tnum mt-3 text-xs text-mute">{s.accent.toUpperCase()}</div>
      </Card>

      <Card title="Font">
        <div className="space-y-2">
          {FONTS.map((f) => (
            <button key={f.id} onClick={() => set({ font: f.id })} className={`flex w-full cursor-pointer items-center justify-between border px-3 py-2 text-left transition ${s.font === f.id ? "border-accent bg-accent/10" : "border-line hover:border-mute"}`} style={{ fontFamily: f.css, borderRadius: "calc(var(--radius) * 0.6)" }}>
              <span>{f.name}</span>
              <span className="text-sm text-mute">CPU 42% · 16.0 GB</span>
            </button>
          ))}
        </div>
      </Card>

      <Card title="Interface size & shape">
        <div className="space-y-5 text-sm">
          <label className="block">
            <div className="mb-1 flex justify-between"><span>UI size</span><span className="tnum text-mute">{Math.round((s.size / 16) * 100)}%</span></div>
            <input type="range" min={12} max={22} step={1} value={s.size} onChange={(e) => set({ size: +e.target.value })} className="w-full" />
          </label>
          <label className="block">
            <div className="mb-1 flex justify-between"><span>Corner roundness</span><span className="tnum text-mute">{s.radius}px</span></div>
            <input type="range" min={0} max={28} step={2} value={s.radius} onChange={(e) => set({ radius: +e.target.value })} className="w-full" />
          </label>
          <label className="flex cursor-pointer items-center justify-between">
            <span>Compact layout</span>
            <input type="checkbox" checked={s.compact} onChange={(e) => set({ compact: e.target.checked })} className="h-5 w-5" />
          </label>
          <Btn danger onClick={reset}>Reset to defaults</Btn>
        </div>
      </Card>
    </div>
  );
}
