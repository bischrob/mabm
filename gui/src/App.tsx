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
    hex_count: number;
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
  hex_id: number;
  grid_q: number;
  grid_r: number;
  population_total: number;
  households: number;
  climate_pdsi: number;
  drought_index_5y: number;
  water_reliability: number;
  water_quality: number;
  fuel_stock: number;
  food_yield_kcal: number;
  food_stores_kcal: number;
  food_deficit_kcal: number;
  food_capacity_persons: number;
  hex_quality: number;
  stress_composite: number;
  defensibility: number;
  burden_multiplier: number;
  disease_infected_share: number;
  is_active: boolean;
  status: string;
};

type HexMetricKey =
  | "hex_quality"
  | "food_capacity_persons"
  | "population_total"
  | "households"
  | "food_yield_kcal"
  | "food_stores_kcal"
  | "food_deficit_kcal"
  | "water_reliability"
  | "water_quality"
  | "fuel_stock"
  | "stress_composite"
  | "defensibility"
  | "burden_multiplier"
  | "disease_infected_share"
  | "climate_pdsi"
  | "drought_index_5y";

type MetricOption = {
  key: HexMetricKey;
  label: string;
};

const HEX_METRIC_OPTIONS: MetricOption[] = [
  { key: "hex_quality", label: "Hex Quality" },
  { key: "food_capacity_persons", label: "Food Capacity (persons)" },
  { key: "population_total", label: "Population" },
  { key: "households", label: "Households" },
  { key: "food_yield_kcal", label: "Food Yield (kcal)" },
  { key: "food_stores_kcal", label: "Food Stores (kcal)" },
  { key: "food_deficit_kcal", label: "Food Deficit (kcal)" },
  { key: "water_reliability", label: "Water Reliability" },
  { key: "water_quality", label: "Water Quality" },
  { key: "fuel_stock", label: "Fuel Stock" },
  { key: "stress_composite", label: "Stress Composite" },
  { key: "defensibility", label: "Defensibility" },
  { key: "burden_multiplier", label: "Burden Multiplier" },
  { key: "disease_infected_share", label: "Disease Infected Share" },
  { key: "climate_pdsi", label: "Climate PDSI" },
  { key: "drought_index_5y", label: "Drought Index 5y" }
];

type VisualPoint = {
  year: number;
  population_total: number;
};

type VisualPayload = {
  run_id: string;
  hex_count: number;
  latest_tick: number;
  population_series: VisualPoint[];
  settlements_latest: VisualSettlement[];
};

type ConfigRequirementFile = {
  path: string;
  exists: boolean;
};

type ConfigCatalogEntry = {
  path: string;
  scenario_id: string;
  label: string;
  use_gis_hex_inputs: boolean;
  gis_hex_csv_path: string;
  gui_load?: {
    description?: string;
    default?: boolean;
    required_files?: ConfigRequirementFile[];
  };
};

type ConfigCatalogResponse = {
  default_path: string;
  configs: ConfigCatalogEntry[];
};

export function App() {
  const [index, setIndex] = useState<RunIndex | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [selectedManifest, setSelectedManifest] = useState<RunManifest | null>(null);
  const [configCatalog, setConfigCatalog] = useState<ConfigCatalogEntry[]>([]);
  const [configPath, setConfigPath] = useState("configs/phoenix_basin.toml");
  const [ticksOverride, setTicksOverride] = useState(0);
  const [liveUpdateEveryTicks, setLiveUpdateEveryTicks] = useState(10);
  const [seedValue, setSeedValue] = useState<number>(() => randomSeed());
  const [lockSeed, setLockSeed] = useState(false);
  const [running, setRunning] = useState(false);
  const [runLog, setRunLog] = useState("");
  const [visuals, setVisuals] = useState<VisualPayload | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [hexMetricKey, setHexMetricKey] = useState<HexMetricKey>("hex_quality");

  useEffect(() => {
    refreshConfigs();
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
      .then((v: VisualPayload) =>
        setVisuals({
          ...v,
          settlements_latest: (v.settlements_latest ?? []).map(normalizeSettlement)
        })
      )
      .catch((e) => setError(String(e)));
  }, [selectedRunId]);

  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => {
      fetch("/api/live-progress")
        .then((r) => r.json())
        .then((lp) => {
          if (lp && Array.isArray(lp.settlements_latest)) {
            const lpRunId = String(lp.run_id ?? "");
            setVisuals((prev) => {
              const isSameRun = prev?.run_id && prev.run_id === lpRunId;
              const base = prev?.population_series ?? [];
              const year = Number(lp.year ?? 0);
              const pop = Number(lp.population_total ?? 0);
              const merged = isSameRun ? [...base, { year, population_total: pop }] : [{ year, population_total: pop }];
              const nextSeries = normalizeSeries(merged);
              return {
                run_id: lpRunId || prev?.run_id || "",
                hex_count:
                  prev?.hex_count ?? inferHexCountFromSettlements(lp.settlements_latest ?? []),
                latest_tick: Number(lp.tick ?? prev?.latest_tick ?? 0),
                settlements_latest: (lp.settlements_latest ?? []).map(normalizeSettlement),
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
  const selectedConfigMeta = useMemo(
    () => configCatalog.find((c) => c.path === configPath) ?? null,
    [configCatalog, configPath]
  );
  const missingConfigRequirements = useMemo(
    () => (selectedConfigMeta?.gui_load?.required_files ?? []).filter((f) => !f.exists),
    [selectedConfigMeta]
  );
  const currentTick = visuals?.latest_tick ?? 0;
  const initialHexCount =
    visuals?.hex_count ??
    selectedManifest?.summary.hex_count ??
    selectedManifest?.summary.settlement_count ??
    0;
  const initialHexes = useMemo(
    () => buildInitialHexPlaceholders(initialHexCount),
    [initialHexCount]
  );
  const mapSettlements = useMemo(
    () => mergeHexSnapshots(initialHexes, visuals?.settlements_latest ?? []),
    [initialHexes, visuals]
  );

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

  async function refreshConfigs() {
    try {
      const res = await fetch("/api/configs");
      if (!res.ok) throw new Error(await res.text());
      const data: ConfigCatalogResponse = await res.json();
      const list = data.configs ?? [];
      setConfigCatalog(list);
      setError(null);
      if (list.length === 0) return;
      const hasCurrent = list.some((c) => c.path === configPath);
      if (!hasCurrent) {
        setConfigPath(data.default_path || list[0].path);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function triggerRun() {
    if (missingConfigRequirements.length > 0) {
      setError(
        `Cannot run. Missing required files: ${missingConfigRequirements
          .map((r) => r.path)
          .join(", ")}`
      );
      return;
    }
    setRunning(true);
    setRunLog("");
    try {
      setVisuals(null);
      const runSeed = lockSeed ? seedValue : randomSeed();
      setSeedValue(runSeed);
      const res = await fetch("/api/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          configPath,
          ticksOverride,
          liveUpdateEveryTicks,
          seedOverride: runSeed
        })
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
          <label>
            Config:
            <select
              value={configPath}
              onChange={(e) => setConfigPath(e.target.value)}
              className="cfg"
            >
              {configCatalog.length === 0 ? (
                <option value={configPath}>{configPath}</option>
              ) : (
                configCatalog.map((c) => (
                  <option key={c.path} value={c.path}>
                    {c.label}
                  </option>
                ))
              )}
            </select>
          </label>
          <button onClick={triggerRun} disabled={running || missingConfigRequirements.length > 0}>
            {running ? "Running..." : "Run"}
          </button>
          <button onClick={refreshIndex}>Refresh</button>
          <button onClick={refreshConfigs}>Refresh Configs</button>
        </div>
        <div className="runbox">
          <div>Tick: {currentTick}</div>
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
          <label>
            Seed:
            <input
              type="number"
              min={1}
              value={seedValue}
              onChange={(e) => setSeedValue(Number(e.target.value || 1))}
              className="small"
            />
          </label>
          <label>
            <input
              type="checkbox"
              checked={lockSeed}
              onChange={(e) => setLockSeed(e.target.checked)}
            />
            Keep seed fixed
          </label>
        </div>
        {selectedConfigMeta ? (
          <div className="runbox">
            <div>
              <strong>Scenario:</strong> {selectedConfigMeta.scenario_id || "-"}
            </div>
            <div>
              <strong>GIS mode:</strong>{" "}
              {selectedConfigMeta.use_gis_hex_inputs ? "enabled" : "disabled"}
            </div>
            {selectedConfigMeta.gui_load?.description ? (
              <div>
                <strong>Load:</strong> {selectedConfigMeta.gui_load.description}
              </div>
            ) : null}
            {(selectedConfigMeta.gui_load?.required_files?.length ?? 0) > 0 ? (
              <div>
                <strong>Required files:</strong>{" "}
                {(selectedConfigMeta.gui_load?.required_files ?? [])
                  .map((f) => `${f.exists ? "OK" : "Missing"} ${f.path}`)
                  .join(" | ")}
              </div>
            ) : null}
          </div>
        ) : null}
      </header>

      {error ? <div className="error">{error}</div> : null}

      <main className="grid">
        <section className="panel">
          <h2>Settlement Hex Grid</h2>
          <label>
            Hex metric:
            <select
              value={hexMetricKey}
              onChange={(e) => setHexMetricKey(e.target.value as HexMetricKey)}
              className="small"
            >
              {HEX_METRIC_OPTIONS.map((o) => (
                <option key={o.key} value={o.key}>
                  {o.label}
                </option>
              ))}
            </select>
          </label>
          <HexGridMap settlements={mapSettlements} metricKey={hexMetricKey} />
        </section>

        <section className="panel">
          <h2>Population Time Series</h2>
          <PopulationGraph points={visuals?.population_series ?? []} />
        </section>

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
              <div>hexes: {selectedManifest.summary.hex_count}</div>
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

        <section className="panel full">
          <h2>Last Run Output</h2>
          <pre>{runLog || "No run output yet."}</pre>
        </section>
      </main>
    </div>
  );
}

function HexGridMap({
  settlements,
  metricKey
}: {
  settlements: VisualSettlement[];
  metricKey: HexMetricKey;
}) {
  if (settlements.length === 0) {
    return <div>No settlement snapshots available.</div>;
  }
  const metricLabel =
    HEX_METRIC_OPTIONS.find((m) => m.key === metricKey)?.label ?? metricKey;
  const metricValues = settlements.map((s) => metricValue(s, metricKey));
  const minMetric = Math.min(...metricValues);
  const maxMetric = Math.max(...metricValues);
  const size = 18;
  const positioned = settlements.map((s) => {
    const x = size * Math.sqrt(3) * (s.grid_q + s.grid_r / 2);
    const y = size * 1.5 * s.grid_r;
    return { ...s, x, y, metric: metricValue(s, metricKey) };
  });
  const minX = Math.min(...positioned.map((p) => p.x));
  const maxX = Math.max(...positioned.map((p) => p.x));
  const minY = Math.min(...positioned.map((p) => p.y));
  const maxY = Math.max(...positioned.map((p) => p.y));
  const pad = 30;
  const w = maxX - minX + pad * 2 + size * 2;
  const h = maxY - minY + pad * 2 + size * 2;
  const legendBarW = Math.max(64, Math.min(110, w * 0.18));
  const legendBarH = 8;
  const legendX = 10;
  const legendY = h + 14;
  const gradientId = `metricGradient-${metricKey}`;
  const legendStops = [0, 0.25, 0.5, 0.75, 1];

  return (
    <svg viewBox={`0 0 ${w} ${h + 54}`} className="viz">
      {positioned.map((p) => {
        const cx = p.x - minX + pad + size;
        const cy = p.y - minY + pad + size;
        const poly = hexPoints(cx, cy, size).join(" ");
        return (
          <g key={`hex-${p.hex_id}`}>
            <title>{hexTooltipText(p)}</title>
            <polygon
              points={poly}
              fill={colorForMetric(p.metric, minMetric, maxMetric, metricKey)}
              stroke={p.is_active ? "var(--hex-active-stroke)" : "var(--hex-abandoned-stroke)"}
              strokeWidth={1.1}
              opacity={1}
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
      <g transform={`translate(${legendX}, ${legendY})`}>
        <defs>
          <linearGradient id={gradientId} x1="0%" y1="0%" x2="100%" y2="0%">
            {legendStops.map((t) => {
              const v = minMetric + (maxMetric - minMetric) * t;
              return (
                <stop
                  key={`legend-stop-${t}`}
                  offset={`${Math.round(t * 100)}%`}
                  stopColor={colorForMetric(v, minMetric, maxMetric, metricKey)}
                />
              );
            })}
          </linearGradient>
        </defs>
        <text x={0} y={-6} fontSize="9" fill="var(--viz-text)">
          {metricLabel}
        </text>
        <rect
          x={0}
          y={0}
          width={legendBarW}
          height={legendBarH}
          fill={`url(#${gradientId})`}
          stroke="var(--viz-axis)"
        />
        <text x={legendBarW + 6} y={legendBarH - 1} fontSize="9" fill="var(--viz-text)">
          {formatMetric(minMetric)} - {formatMetric(maxMetric)}
        </text>
        <circle cx={0} cy={18} r={4} fill="none" stroke="var(--hex-active-stroke)" />
        <text x={10} y={21} fontSize="9" fill="var(--viz-text)">
          Active
        </text>
        <circle cx={58} cy={18} r={4} fill="none" stroke="var(--hex-abandoned-stroke)" />
        <text x={68} y={21} fontSize="9" fill="var(--viz-text)">
          Abandoned/empty
        </text>
      </g>
    </svg>
  );
}

function PopulationGraph({ points }: { points: VisualPoint[] }) {
  const normalized = normalizeSeries(points);
  if (normalized.length === 0) {
    return <div>No population points available yet.</div>;
  }
  if (normalized.length === 1) {
    return <div>Only one population point available so far. Run longer or lower GUI update interval.</div>;
  }
  const w = 540;
  const h = 220;
  const pad = 28;
  const minYearObserved = Math.min(...normalized.map((p) => p.year));
  const minYear = Math.min(0, minYearObserved);
  const maxYear = Math.max(...normalized.map((p) => p.year));
  const rawMinPop = Math.min(...normalized.map((p) => p.population_total));
  const rawMaxPop = Math.max(...normalized.map((p) => p.population_total), 1);
  const popSpan = Math.max(1, rawMaxPop - rawMinPop);
  const minPop = Math.max(0, rawMinPop - popSpan * 0.1);
  const maxPop = rawMaxPop + popSpan * 0.1;
  const sx = (x: number) =>
    pad + ((x - minYear) / Math.max(1e-6, maxYear - minYear)) * (w - pad * 2);
  const sy = (y: number) =>
    h - pad - ((y - minPop) / Math.max(1e-6, maxPop - minPop)) * (h - pad * 2);
  const dNorm = normalized
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
      <path d={dNorm} fill="none" stroke="var(--viz-line)" strokeWidth={2} />
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

function normalizeSeries(points: VisualPoint[]): VisualPoint[] {
  const sorted = [...points].sort((a, b) => a.year - b.year);
  const byYear = new Map<number, number>();
  for (const p of sorted) {
    byYear.set(Number(p.year.toFixed(6)), p.population_total);
  }
  return [...byYear.entries()]
    .map(([year, population_total]) => ({ year, population_total }))
    .sort((a, b) => a.year - b.year);
}

function hexPoints(cx: number, cy: number, size: number): string[] {
  const pts: string[] = [];
  for (let i = 0; i < 6; i++) {
    const a = ((60 * i - 30) * Math.PI) / 180;
    pts.push(`${cx + size * Math.cos(a)},${cy + size * Math.sin(a)}`);
  }
  return pts;
}

function buildInitialHexPlaceholders(count: number): VisualSettlement[] {
  if (!Number.isFinite(count) || count <= 0) return [];
  const out: VisualSettlement[] = [];
  const cols = Math.ceil(Math.sqrt(count));
  for (let i = 0; i < count; i++) {
    const col = i % cols;
    const r = Math.floor(i / cols);
    const q = col - Math.floor(r / 2);
    out.push({
      tick: 0,
      year: 0,
      settlement_id: 0,
      hex_id: i + 1,
      grid_q: q,
      grid_r: r,
      population_total: 0,
      households: 0,
      climate_pdsi: 0,
      drought_index_5y: 0,
      water_reliability: 0,
      water_quality: 0,
      fuel_stock: 0,
      food_yield_kcal: 0,
      food_stores_kcal: 0,
      food_deficit_kcal: 0,
      food_capacity_persons: 0,
      hex_quality: 0,
      stress_composite: 0,
      defensibility: 0,
      burden_multiplier: 1,
      disease_infected_share: 0,
      is_active: false,
      status: "initialized"
    });
  }
  return out;
}

function n(v: unknown): number {
  const x = Number(v ?? 0);
  return Number.isFinite(x) ? x : 0;
}

function normalizeSettlement(s: Partial<VisualSettlement> & Record<string, unknown>): VisualSettlement {
  return {
    tick: n(s.tick),
    year: n(s.year),
    settlement_id: n(s.settlement_id),
    hex_id: n(s.hex_id),
    grid_q: n(s.grid_q),
    grid_r: n(s.grid_r),
    population_total: n(s.population_total),
    households: n(s.households),
    climate_pdsi: n(s.climate_pdsi),
    drought_index_5y: n(s.drought_index_5y),
    water_reliability: n(s.water_reliability),
    water_quality: n(s.water_quality),
    fuel_stock: n(s.fuel_stock),
    food_yield_kcal: n(s.food_yield_kcal),
    food_stores_kcal: n(s.food_stores_kcal),
    food_deficit_kcal: n(s.food_deficit_kcal),
    food_capacity_persons: n(s.food_capacity_persons),
    hex_quality: n(s.hex_quality),
    stress_composite: n(s.stress_composite),
    defensibility: n(s.defensibility),
    burden_multiplier: n(s.burden_multiplier) || 1,
    disease_infected_share: n(s.disease_infected_share),
    is_active: Boolean(s.is_active),
    status: String(s.status ?? "unknown")
  };
}

function mergeHexSnapshots(
  base: VisualSettlement[],
  latest: VisualSettlement[]
): VisualSettlement[] {
  if (base.length === 0) return latest;
  if (latest.length === 0) return base;
  const byHex = new Map<number, VisualSettlement>();
  for (const s of latest) {
    if (s.hex_id > 0) {
      byHex.set(s.hex_id, s);
    }
  }
  return base.map((hex) => byHex.get(hex.hex_id) ?? hex);
}

function inferHexCountFromSettlements(settlements: VisualSettlement[]): number {
  if (settlements.length === 0) return 0;
  let maxHex = 0;
  for (const s of settlements) {
    if (s.hex_id > maxHex) maxHex = s.hex_id;
  }
  return maxHex > 0 ? maxHex : settlements.length;
}

function metricValue(s: VisualSettlement, key: HexMetricKey): number {
  const v = s[key] as number;
  return Number.isFinite(v) ? v : 0;
}

function colorForMetric(v: number, min: number, max: number, key: HexMetricKey): string {
  if (!Number.isFinite(v) || v <= 0) {
    return "hsl(0 0% 100%)";
  }
  if (!Number.isFinite(min) || !Number.isFinite(max) || max <= min) {
    return "hsl(0 0% 100%)";
  }
  const span = Math.max(1e-9, max - min);
  let t = Math.max(0, Math.min(1, (v - min) / span));

  const scale = metricScaleType(key);
  if (scale === "good_low") {
    t = 1 - t;
  }

  if (scale === "population_households") {
    // White (zero) -> blue (mid) -> red (high).
    if (t <= 0.5) {
      const u = t / 0.5;
      return hslLerp(0, 0, 100, 217, 79, 53, u);
    }
    const u = (t - 0.5) / 0.5;
    return hslLerp(217, 79, 53, 0, 70, 50, u);
  }

  // Default: red (bad) -> orange (mid) -> green (good).
  if (t <= 0.5) {
    const u = t / 0.5;
    return hslLerp(0, 70, 50, 36, 92, 50, u);
  }
  const u = (t - 0.5) / 0.5;
  return hslLerp(36, 92, 50, 142, 72, 40, u);
}

function metricScaleType(
  key: HexMetricKey
): "default_good_high" | "good_low" | "population_households" {
  if (key === "population_total" || key === "households") {
    return "population_households";
  }
  if (
    key === "food_deficit_kcal" ||
    key === "stress_composite" ||
    key === "disease_infected_share" ||
    key === "drought_index_5y"
  ) {
    return "good_low";
  }
  return "default_good_high";
}

function hslLerp(
  h1: number,
  s1: number,
  l1: number,
  h2: number,
  s2: number,
  l2: number,
  t: number
): string {
  const u = Math.max(0, Math.min(1, t));
  const h = h1 + (h2 - h1) * u;
  const s = s1 + (s2 - s1) * u;
  const l = l1 + (l2 - l1) * u;
  return `hsl(${h.toFixed(1)} ${s.toFixed(1)}% ${l.toFixed(1)}%)`;
}

function formatMetric(v: number): string {
  if (!Number.isFinite(v)) return "0";
  if (Math.abs(v) >= 1_000_000) return `${(v / 1_000_000).toFixed(2)}M`;
  if (Math.abs(v) >= 1_000) return `${(v / 1_000).toFixed(1)}k`;
  if (Math.abs(v) >= 100) return v.toFixed(0);
  if (Math.abs(v) >= 1) return v.toFixed(2);
  return v.toFixed(3);
}

function randomSeed(): number {
  return Math.floor(Math.random() * 2_147_483_646) + 1;
}

function hexTooltipText(h: VisualSettlement): string {
  const lines = [
    `Hex ${h.hex_id} (${h.status})`,
    `Settlement ID: ${h.settlement_id || "-"}`,
    `Grid: q=${h.grid_q}, r=${h.grid_r}`,
    `Tick: ${h.tick}, Year: ${h.year.toFixed(2)}`,
    `Population: ${h.population_total}`,
    `Households: ${h.households}`,
    `Food capacity: ${formatMetric(h.food_capacity_persons)}`,
    `Food yield: ${formatMetric(h.food_yield_kcal)} kcal`,
    `Food stores: ${formatMetric(h.food_stores_kcal)} kcal`,
    `Food deficit: ${formatMetric(h.food_deficit_kcal)} kcal`,
    `Hex quality: ${n(h.hex_quality).toFixed(3)}`,
    `Water reliability: ${h.water_reliability.toFixed(3)}`,
    `Water quality: ${h.water_quality.toFixed(3)}`,
    `Fuel stock: ${formatMetric(h.fuel_stock)}`,
    `Stress composite: ${h.stress_composite.toFixed(3)}`,
    `Defensibility: ${h.defensibility.toFixed(3)}`,
    `Burden multiplier: ${h.burden_multiplier.toFixed(3)}`,
    `Disease infected share: ${h.disease_infected_share.toFixed(3)}`,
    `Climate PDSI: ${h.climate_pdsi.toFixed(3)}`,
    `Drought index 5y: ${h.drought_index_5y.toFixed(3)}`
  ];
  return lines.join("\n");
}
