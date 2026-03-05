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
  };
};

export function App() {
  const [index, setIndex] = useState<RunIndex | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [selectedManifest, setSelectedManifest] = useState<RunManifest | null>(null);
  const [configPath, setConfigPath] = useState("configs/sweep.toml");
  const [running, setRunning] = useState(false);
  const [runLog, setRunLog] = useState("");
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
