import { useState } from "react";
import { Card, Btn } from "../components/ui";
import { TOTALS, type Snapshot } from "../sim";

export default function Specs({ d }: { d: Snapshot }) {
  const [t, setT] = useState("CPU");
  const cpu = d.cpu[d.cpu.length - 1];
  const sections: Record<string, [string, string][]> = {
    CPU: [
      ["Name", "AMD Ryzen 7 5800X"], ["Architecture", "x86_64 (Zen 3)"], ["Codename", "Vermeer"], ["Package", "Socket AM4"], ["Technology", "7 nm"],
      ["Cores / Threads", "8 / 16"], ["Core speed", `${(3800 + cpu * 9).toFixed(0)} MHz`], ["Base / Boost", "3800 MHz / 4700 MHz"],
      ["L1 cache", "8 × 32 KB + 8 × 32 KB"], ["L2 cache", "8 × 512 KB"], ["L3 cache", "32 MB"],
      ["Instructions", "SSE4.2, AVX2, AES, FMA3, SHA, BMI2"], ["Temperature", `${d.temp.cpu.toFixed(0)} °C`], ["TDP", "105 W"],
    ],
    Mainboard: [["Manufacturer", "ASUSTeK COMPUTER INC."], ["Model", "ROG STRIX B550-F GAMING"], ["Chipset", "AMD B550"], ["BIOS vendor", "American Megatrends"], ["BIOS version", "3607"], ["BIOS date", "03/14/2024"], ["PCIe", "Gen 4.0 x16"], ["Secure boot", "Enabled"]],
    Memory: [["Type", "DDR4"], ["Size", `${TOTALS.ramGB} GB`], ["Channels", "Dual"], ["Frequency", "1800 MHz (3600 MT/s)"], ["Timings", "CL16‑19‑19‑39"], ["Slots used", "2 / 4"], ["Slot 1", "16 GB · Corsair Vengeance"], ["Slot 2", "16 GB · Corsair Vengeance"]],
    GPU: [["Name", "NVIDIA GeForce RTX 3080 Ti"], ["Driver", "555.58"], ["VRAM", `${TOTALS.vramGB} GB GDDR6X`], ["Core clock", "1665 MHz"], ["Memory clock", "9500 MHz"], ["Bus", "PCIe 4.0 x16"], ["Shaders", "10240"], ["Temperature", `${d.temp.gpu.toFixed(0)} °C`], ["Power limit", "350 W"]],
    System: [["Operating system", "Linux / macOS / Windows (detected at run time)"], ["Kernel", "6.8.0"], ["Hostname", "ferrum-pc"], ["Display", "2560 × 1440 @ 144 Hz"], ["Uptime", `${Math.floor(d.uptime / 3600)} h`], ["Architecture", "x86_64 / aarch64"]],
  };
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2">
        {Object.keys(sections).map((k) => <Btn key={k} active={t === k} onClick={() => setT(k)}>{k}</Btn>)}
      </div>
      <Card title={`${t} information`}>
        <dl className="grid gap-x-8 md:grid-cols-2">
          {sections[t].map(([k, v]) => (
            <div key={k} className="flex justify-between gap-4 border-b border-line/60 py-2 text-sm">
              <dt className="text-mute">{k}</dt>
              <dd className="tnum text-right font-medium">{v}</dd>
            </div>
          ))}
        </dl>
      </Card>
      {t === "CPU" && (
        <Card title="Live core clocks">
          <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
            {d.cores.map((c, i) => (
              <div key={i} className="rounded-lg bg-panel2 px-3 py-2 text-sm">
                <div className="text-xs text-mute">Core {i}</div>
                <div className="tnum font-semibold">{(3600 + c * 11).toFixed(0)} MHz</div>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
}
