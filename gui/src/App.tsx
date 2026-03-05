import { useEffect, useMemo, useState } from "react";

type RunIndexEntry = {
  run_id: string;
  scenario_id: string;
  started_at_utc: string;
  config_hash: string;
  manifest_path: string;
};

type RunIndex = {
  updated_at_utc: string;
  entries: RunIndexEntry[];
};

type RunManifest = {
  scenario_id: string;
  run_id: string;
  started_at_utc: string;
  code_version: string;
  config_hash: string;
  config_path: string;
  manifest_created_at_utc: string;
  files: Record<string, string | null>;
  summary: {
    settlement_count: number;
    trait_rows: number;
    baseline_metric_rows: number;
    deposition_rows: number;
    network_rows: number;
    sweep_rows: number;
    settlement_snapshot_rows: number;
  };
};

type VisualSettlement = {
  tick: number;
  year: number;
  settlement_id: number;
  grid_q: number;
  grid_r: number;
  population_total: number;
  is_active: boolean;
  status: string;
};

type VisualPoint = {
  year: number;
  population_total: number;
};

type VisualPayload = {
  run_id: string;
  latest_tick: number;
  population_series: VisualPoint[];
  settlements_latest: VisualSettlement[];
};

export function App() {
  const [index, setIndex] = useState<RunIndex | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [selectedManifest, setSelectedManifest] = useState<RunManifest | null>(null);
  const [configPath, setConfigPath] = useState("configs/sweep.toml");
  const [ticksOverride, setTicksOverride] = useState(0);
  const [liveUpdateEveryTicks, setLiveUpdateEveryTicks] = useState(10);
  const [running, setRunning] = useState(false);
  const [runLog, setRunLog] = useState("");
  const [visuals, setVisuals] = useState<VisualPayload | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    refreshIndex();
  }, []);

  useEffect(() => {
    if (!selectedRunId) {
      setSelectedManifest(null);
      return;
    }
    fetch(`/api/runs/${encodeURIComponent(selectedRunId)}`)
      .then((r) => r.json())
      .then((m: RunManifest) => setSelectedManifest(m))
      .catch((e) => setError(String(e)));

    fetch(`/api/runs/${encodeURIComponent(selectedRunId)}/visuals`)
      .then((r) => r.json())
      .then((v: VisualPayload) => setVisuals(v))
      .catch((e) => setError(String(e)));
  }, [selectedRunId]);

  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => {
      fetch("/api/live-progress")
        .then((r) => r.json())
        .then((lp) => {
          if (lp && Array.isArray(lp.settlements_latest)) {
            setVisuals((prev) => {
              const base = prev?.population_series ?? [];
              const year = Number(lp.year ?? 0);
              const pop = Number(lp.population_total ?? 0);
              const nextSeries =
                base.length > 0 && Math.abs(base[base.length - 1].year - year) < 1e-6
                  ? base.map((p, i) =>
                      i === base.length - 1 ? { year, population_total: pop } : p
                    )
                  : [...base, { year, population_total: pop }];
              return {
                run_id: String(lp.run_id ?? prev?.run_id ?? ""),
                latest_tick: Number(lp.tick ?? prev?.latest_tick ?? 0),
                settlements_latest: lp.settlements_latest,
                population_series: nextSeries
              };
            });
          }
        })
        .catch(() => {});
    }, 1000);
    return () => clearInterval(timer);
  }, [running]);

  const entries = useMemo(() => index?.entries ?? [], [index]);

  async function refreshIndex() {
    try {
      const res = await fetch("/api/runs");
      if (!res.ok) throw new Error(await res.text());
      const data: RunIndex = await res.json();
      setIndex(data);
      setError(null);
      if (!selectedRunId && data.entries.length > 0) {
        setSelectedRunId(data.entries[0].run_id);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function triggerRun() {
    setRunning(true);
    setRunLog("");
    try {
      const res = await fetch("/api/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ configPath, ticksOverride, liveUpdateEveryTicks })
      });
      const body = await res.json();
      if (!res.ok) {
        throw new Error(body?.error ?? "run failed");
      }
      setRunLog(body.stdout ?? "");
      await refreshIndex();
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="app">
      <header className="topbar">
        <h1>MABM Run Console</h1>
        <div className="runbox">
          <input
            value={configPath}
            onChange={(e) => setConfigPath(e.target.value)}
            className="cfg"
          />
          <button onClick={triggerRun} disabled={running}>
            {running ? "Running..." : "Run"}
          </button>
          <button onClick={refreshIndex}>Refresh</button>
        </div>
        <div className="runbox">
          <label>
            Ticks:
            <input
              type="number"
              min={1}
              value={ticksOverride || ""}
              onChange={(e) => setTicksOverride(Number(e.target.value || 0))}
              className="small"
              placeholder="use config"
            />
          </label>
          <label>
            GUI update every:
            <input
              type="number"
              min={0}
              value={liveUpdateEveryTicks}
              onChange={(e) => setLiveUpdateEveryTicks(Number(e.target.value || 0))}
              className="small"
            />
          </label>
        </div>
      </header>

      {error ? <div className="error">{error}</div> : null}

      <main className="grid">
        <section className="panel">
          <h2>Runs</h2>
          <table>
            <thead>
              <tr>
                <th>Run</th>
                <th>Scenario</th>
                <th>Started</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => (
                <tr
                  key={e.run_id}
                  onClick={() => setSelectedRunId(e.run_id)}
                  className={selectedRunId === e.run_id ? "active" : ""}
                >
                  <td>{e.run_id}</td>
                  <td>{e.scenario_id}</td>
                  <td>{e.started_at_utc}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>

        <section className="panel">
          <h2>Manifest</h2>
          {selectedManifest ? (
            <div className="details">
              <div>scenario: {selectedManifest.scenario_id}</div>
              <div>run: {selectedManifest.run_id}</div>
              <div>config: {selectedManifest.config_path}</div>
              <div>hash: {selectedManifest.config_hash}</div>
              <div>settlements: {selectedManifest.summary.settlement_count}</div>
              <div>traits rows: {selectedManifest.summary.trait_rows}</div>
              <div>baseline rows: {selectedManifest.summary.baseline_metric_rows}</div>
              <div>sweep rows: {selectedManifest.summary.sweep_rows}</div>
              <div>settlement snapshots: {selectedManifest.summary.settlement_snapshot_rows}</div>
            </div>
          ) : (
            <div>Select a run.</div>
          )}
        </section>

        <section className="panel">
      <h2>Settlement Hex Grid</h2>
          <HexGridMap settlements={visuals?.settlements_latest ?? []} />
        </section>

        <section className="panel">
          <h2>Population Time Series</h2>
          <PopulationGraph points={visuals?.population_series ?? []} />
        </section>

        <section className="panel full">
          <h2>Last Run Output</h2>
          <pre>{runLog || "No run output yet."}</pre>
        </section>
      </main>
    </div>
  );
}

function HexGridMap({ settlements }: { settlements: VisualSettlement[] }) {
  if (settlements.length === 0) {
    return <div>No settlement snapshots available.</div>;
  }
  const size = 18;
  const positioned = settlements.map((s) => {
    const x = size * Math.sqrt(3) * (s.grid_q + s.grid_r / 2);
    const y = size * 1.5 * s.grid_r;
    return { ...s, x, y };
  });
  const minX = Math.min(...positioned.map((p) => p.x));
  const maxX = Math.max(...positioned.map((p) => p.x));
  const minY = Math.min(...positioned.map((p) => p.y));
  const maxY = Math.max(...positioned.map((p) => p.y));
  const pad = 30;
  const w = maxX - minX + pad * 2 + size * 2;
  const h = maxY - minY + pad * 2 + size * 2;

  return (
    <svg viewBox={`0 0 ${w} ${h + 34}`} className="viz">
      {positioned.map((p) => {
        const cx = p.x - minX + pad + size;
        const cy = p.y - minY + pad + size;
        const poly = hexPoints(cx, cy, size).join(" ");
        return (
          <g key={p.settlement_id}>
            <polygon
              points={poly}
              fill={p.is_active ? "var(--hex-active-fill)" : "var(--hex-abandoned-fill)"}
              stroke={p.is_active ? "var(--hex-active-stroke)" : "var(--hex-abandoned-stroke)"}
              strokeWidth={1.1}
            />
            <circle
              cx={cx}
              cy={cy}
              r={Math.max(2, Math.min(9, Math.sqrt(p.population_total) * 0.32))}
              fill={p.is_active ? "var(--hex-active-node)" : "var(--hex-abandoned-node)"}
              opacity={0.9}
            />
          </g>
        );
      })}
      <g transform={`translate(10, ${h + 14})`}>
        <rect x={0} y={-8} width={12} height={12} fill="var(--hex-active-fill)" stroke="var(--hex-active-stroke)" />
        <text x={18} y={2} fontSize="10" fill="var(--viz-text)">Active settlement</text>
        <rect x={120} y={-8} width={12} height={12} fill="var(--hex-abandoned-fill)" stroke="var(--hex-abandoned-stroke)" />
        <text x={138} y={2} fontSize="10" fill="var(--viz-text)">Abandoned settlement</text>
      </g>
    </svg>
  );
}

function PopulationGraph({ points }: { points: VisualPoint[] }) {
  if (points.length < 2) {
    return <div>Not enough baseline metric points.</div>;
  }
  const w = 540;
  const h = 220;
  const pad = 28;
  const minYear = Math.min(...points.map((p) => p.year));
  const maxYear = Math.max(...points.map((p) => p.year));
  const rawMinPop = Math.min(...points.map((p) => p.population_total));
  const rawMaxPop = Math.max(...points.map((p) => p.population_total), 1);
  const popSpan = Math.max(1, rawMaxPop - rawMinPop);
  const minPop = Math.max(0, rawMinPop - popSpan * 0.1);
  const maxPop = rawMaxPop + popSpan * 0.1;
  const sx = (x: number) =>
    pad + ((x - minYear) / Math.max(1e-6, maxYear - minYear)) * (w - pad * 2);
  const sy = (y: number) =>
    h - pad - ((y - minPop) / Math.max(1e-6, maxPop - minPop)) * (h - pad * 2);
  const d = points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${sx(p.year)} ${sy(p.population_total)}`)
    .join(" ");
  const xTicks = 5;
  const yTicks = 5;

  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="viz">
      <line x1={pad} y1={h - pad} x2={w - pad} y2={h - pad} stroke="var(--viz-axis)" />
      <line x1={pad} y1={pad} x2={pad} y2={h - pad} stroke="var(--viz-axis)" />
      {Array.from({ length: xTicks + 1 }).map((_, i) => {
        const t = i / xTicks;
        const x = pad + t * (w - pad * 2);
        const yr = minYear + t * (maxYear - minYear);
        return (
          <g key={`xt-${i}`}>
            <line x1={x} y1={h - pad} x2={x} y2={h - pad + 4} stroke="var(--viz-axis)" />
            <text x={x - 8} y={h - pad + 15} fontSize="9" fill="var(--viz-text)">
              {formatTick(yr)}
            </text>
          </g>
        );
      })}
      {Array.from({ length: yTicks + 1 }).map((_, i) => {
        const t = i / yTicks;
        const y = h - pad - t * (h - pad * 2);
        const pv = minPop + t * (maxPop - minPop);
        return (
          <g key={`yt-${i}`}>
            <line x1={pad - 4} y1={y} x2={pad} y2={y} stroke="var(--viz-axis)" />
            <text x={2} y={y + 3} fontSize="9" fill="var(--viz-text)">
              {formatTick(pv)}
            </text>
          </g>
        );
      })}
      <path d={d} fill="none" stroke="var(--viz-line)" strokeWidth={2} />
      <text x={pad} y={14} fontSize="10" fill="var(--viz-text)">
        Pop max: {Math.round(maxPop)}
      </text>
      <text x={w - 110} y={h - 8} fontSize="10" fill="var(--viz-text)">
        Year {minYear} to {maxYear}
      </text>
    </svg>
  );
}

function formatTick(v: number): string {
  if (Math.abs(v) >= 1000) return Math.round(v).toString();
  if (Math.abs(v) >= 100) return v.toFixed(0);
  if (Math.abs(v) >= 10) return v.toFixed(1);
  return v.toFixed(2);
}

function hexPoints(cx: number, cy: number, size: number): string[] {
  const pts: string[] = [];
  for (let i = 0; i < 6; i++) {
    const a = ((60 * i - 30) * Math.PI) / 180;
    pts.push(`${cx + size * Math.cos(a)},${cy + size * Math.sin(a)}`);
  }
  return pts;
}
