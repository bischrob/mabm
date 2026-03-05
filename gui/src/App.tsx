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
        body: JSON.stringify({ configPath })
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
    <svg viewBox={`0 0 ${w} ${h}`} className="viz">
      {positioned.map((p) => {
        const cx = p.x - minX + pad + size;
        const cy = p.y - minY + pad + size;
        const poly = hexPoints(cx, cy, size).join(" ");
        return (
          <g key={p.settlement_id}>
            <polygon
              points={poly}
              fill={p.is_active ? "#d8f0c9" : "#f0e4d8"}
              stroke={p.is_active ? "#4e8d4e" : "#9a6b47"}
              strokeWidth={1.1}
            />
            <circle
              cx={cx}
              cy={cy}
              r={Math.max(2, Math.min(9, Math.sqrt(p.population_total) * 0.32))}
              fill={p.is_active ? "#2d6e2d" : "#934c2a"}
              opacity={0.9}
            />
          </g>
        );
      })}
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
  const minPop = 0;
  const maxPop = Math.max(...points.map((p) => p.population_total), 1);
  const sx = (x: number) => pad + ((x - minYear) / Math.max(1e-6, maxYear - minYear)) * (w - pad * 2);
  const sy = (y: number) => h - pad - ((y - minPop) / Math.max(1e-6, maxPop - minPop)) * (h - pad * 2);
  const d = points.map((p, i) => `${i === 0 ? "M" : "L"} ${sx(p.year)} ${sy(p.population_total)}`).join(" ");

  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="viz">
      <line x1={pad} y1={h - pad} x2={w - pad} y2={h - pad} stroke="#7c8a7c" />
      <line x1={pad} y1={pad} x2={pad} y2={h - pad} stroke="#7c8a7c" />
      <path d={d} fill="none" stroke="#245a9b" strokeWidth={2} />
      <text x={pad} y={14} fontSize="10" fill="#4f5c4f">
        Pop max: {Math.round(maxPop)}
      </text>
      <text x={w - 110} y={h - 8} fontSize="10" fill="#4f5c4f">
        Year {minYear} to {maxYear}
      </text>
    </svg>
  );
}

function hexPoints(cx: number, cy: number, size: number): string[] {
  const pts: string[] = [];
  for (let i = 0; i < 6; i++) {
    const a = ((60 * i - 30) * Math.PI) / 180;
    pts.push(`${cx + size * Math.cos(a)},${cy + size * Math.sin(a)}`);
  }
  return pts;
}
