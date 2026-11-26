import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

export type Theme = {
  id: string;
  name: string;
  vars: Record<string, string>;
};

export const THEMES: Theme[] = [
  { id: "midnight", name: "Midnight", vars: { "--bg": "#0b0f17", "--panel": "#121826", "--panel2": "#182032", "--line": "#243049", "--fg": "#e6ebf5", "--mute": "#8a97b2" } },
  { id: "graphite", name: "Graphite", vars: { "--bg": "#121212", "--panel": "#1b1b1d", "--panel2": "#242427", "--line": "#333338", "--fg": "#eeeeee", "--mute": "#9a9aa2" } },
  { id: "light", name: "Daylight", vars: { "--bg": "#eef1f6", "--panel": "#ffffff", "--panel2": "#f3f5f9", "--line": "#dbe0ea", "--fg": "#1a2233", "--mute": "#667189" } },
  { id: "paper", name: "Paper", vars: { "--bg": "#f6f1e7", "--panel": "#fffaf0", "--panel2": "#f1ead9", "--line": "#e1d8c2", "--fg": "#3a3226", "--mute": "#8b7f69" } },
  { id: "forest", name: "Forest", vars: { "--bg": "#0c1411", "--panel": "#121d18", "--panel2": "#18271f", "--line": "#243a2f", "--fg": "#e3f1e9", "--mute": "#85a393" } },
  { id: "dracula", name: "Dusk", vars: { "--bg": "#1d1b2b", "--panel": "#262337", "--panel2": "#2f2b44", "--line": "#403b5e", "--fg": "#efeaff", "--mute": "#a79fc9" } },
];

export const FONTS = [
  { id: "sans", name: "System Sans", css: 'ui-sans-serif, system-ui, "Segoe UI", Roboto, sans-serif' },
  { id: "mono", name: "Monospace", css: 'ui-monospace, "JetBrains Mono", "Cascadia Code", Menlo, Consolas, monospace' },
  { id: "serif", name: "Serif", css: 'ui-serif, Georgia, "Times New Roman", serif' },
  { id: "rounded", name: "Rounded", css: 'ui-rounded, "SF Pro Rounded", "Nunito", "Varela Round", system-ui, sans-serif' },
  { id: "condensed", name: "Condensed", css: '"Arial Narrow", "Roboto Condensed", "Helvetica Neue", sans-serif' },
];

export const ACCENTS = ["#38bdf8", "#34d399", "#f472b6", "#fbbf24", "#a78bfa", "#fb7185", "#f97316", "#22d3ee"];

export type Settings = {
  theme: string;
  accent: string;
  font: string;
  size: number; // px root font size
  radius: number;
  compact: boolean;
};

const DEFAULTS: Settings = { theme: "midnight", accent: "#38bdf8", font: "sans", size: 16, radius: 14, compact: false };

type Ctx = { s: Settings; set: (p: Partial<Settings>) => void; reset: () => void };
const C = createContext<Ctx>(null as unknown as Ctx);
export const useSettings = () => useContext(C);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [s, setS] = useState<Settings>(() => {
    try {
      return { ...DEFAULTS, ...JSON.parse(localStorage.getItem("ferrum-settings") || "{}") };
    } catch {
      return DEFAULTS;
    }
  });

  useEffect(() => {
    const root = document.documentElement;
    const theme = THEMES.find((t) => t.id === s.theme) ?? THEMES[0];
    Object.entries(theme.vars).forEach(([k, v]) => root.style.setProperty(k, v));
    root.style.setProperty("--accent", s.accent);
    root.style.setProperty("--ui-size", `${s.size}px`);
    root.style.setProperty("--radius", `${s.radius}px`);
    root.style.setProperty("--font", (FONTS.find((f) => f.id === s.font) ?? FONTS[0]).css);
    localStorage.setItem("ferrum-settings", JSON.stringify(s));
  }, [s]);

  return (
    <C.Provider value={{ s, set: (p) => setS((o) => ({ ...o, ...p })), reset: () => setS(DEFAULTS) }}>
      {children}
    </C.Provider>
  );
}
