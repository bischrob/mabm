import express from "express";
import path from "node:path";
import { fileURLToPath } from "node:url";
import fs from "node:fs/promises";
import { spawn } from "node:child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..", "..");
const outputsDir = path.join(repoRoot, "outputs");
const runIndexPath = path.join(outputsDir, "run_index.json");

const app = express();
app.use(express.json());

app.get("/api/health", (_req, res) => {
  res.json({ ok: true });
});

app.get("/api/runs", async (_req, res) => {
  try {
    const raw = await fs.readFile(runIndexPath, "utf-8");
    res.type("application/json").send(raw);
  } catch {
    res.json({ updated_at_utc: "", entries: [] });
  }
});

app.get("/api/runs/:runId", async (req, res) => {
  try {
    const raw = await fs.readFile(runIndexPath, "utf-8");
    const idx = JSON.parse(raw);
    const entry = (idx.entries ?? []).find((e) => e.run_id === req.params.runId);
    if (!entry) {
      res.status(404).json({ error: "run not found" });
      return;
    }
    const manifestPath = path.join(outputsDir, entry.manifest_path);
    const manifest = await fs.readFile(manifestPath, "utf-8");
    res.type("application/json").send(manifest);
  } catch (e) {
    res.status(500).json({ error: String(e) });
  }
});

app.get("/api/runs/:runId/visuals", async (req, res) => {
  try {
    const manifest = await loadManifestByRunId(req.params.runId);
    if (!manifest) {
      res.status(404).json({ error: "run not found" });
      return;
    }

    const popSeries = manifest.files?.baseline_metrics_csv
      ? await readCsvAsObjects(path.join(outputsDir, manifest.files.baseline_metrics_csv))
      : [];
    const settlementRows = manifest.files?.settlement_snapshot_csv
      ? await readCsvAsObjects(path.join(outputsDir, manifest.files.settlement_snapshot_csv))
      : [];

    const populationSeries = popSeries.map((r) => ({
      year: Number(r.year ?? 0),
      population_total: Number(r.population_total ?? 0)
    }));

    const latestTick = settlementRows.reduce(
      (acc, r) => Math.max(acc, Number(r.tick ?? 0)),
      0
    );
    const latestSettlements = settlementRows
      .filter((r) => Number(r.tick ?? 0) === latestTick)
      .map((r) => ({
        tick: Number(r.tick ?? 0),
        year: Number(r.year ?? 0),
        settlement_id: Number(r.settlement_id ?? 0),
        grid_q: Number(r.grid_q ?? 0),
        grid_r: Number(r.grid_r ?? 0),
        population_total: Number(r.population_total ?? 0),
        is_active: String(r.is_active).toLowerCase() === "true",
        status: String(r.status ?? "")
      }));

    res.json({
      run_id: manifest.run_id,
      latest_tick: latestTick,
      population_series: populationSeries,
      settlements_latest: latestSettlements
    });
  } catch (e) {
    res.status(500).json({ error: String(e) });
  }
});

app.post("/api/run", async (req, res) => {
  const configPath = req.body?.configPath ?? "configs/sweep.toml";
  const args = ["run", "--quiet", "--", configPath];
  const child = spawn("cargo", args, { cwd: repoRoot, shell: true });

  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (d) => (stdout += d.toString()));
  child.stderr.on("data", (d) => (stderr += d.toString()));

  child.on("close", (code) => {
    if (code === 0) {
      res.json({ ok: true, code, stdout, stderr });
    } else {
      res.status(500).json({ ok: false, code, stdout, stderr, error: "run failed" });
    }
  });
});

async function loadManifestByRunId(runId) {
  try {
    const raw = await fs.readFile(runIndexPath, "utf-8");
    const idx = JSON.parse(raw);
    const entry = (idx.entries ?? []).find((e) => e.run_id === runId);
    if (!entry) return null;
    const manifestPath = path.join(outputsDir, entry.manifest_path);
    const manifestRaw = await fs.readFile(manifestPath, "utf-8");
    return JSON.parse(manifestRaw);
  } catch {
    return null;
  }
}

async function readCsvAsObjects(filePath) {
  const raw = await fs.readFile(filePath, "utf-8");
  const lines = raw.split(/\r?\n/).filter((l) => l.length > 0);
  if (lines.length <= 1) return [];
  const headers = splitCsvLine(lines[0]);
  return lines.slice(1).map((line) => {
    const cols = splitCsvLine(line);
    const obj = {};
    for (let i = 0; i < headers.length; i++) {
      obj[headers[i]] = cols[i] ?? "";
    }
    return obj;
  });
}

function splitCsvLine(line) {
  const out = [];
  let cur = "";
  let inQuotes = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      if (inQuotes && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
      continue;
    }
    if (ch === "," && !inQuotes) {
      out.push(cur);
      cur = "";
    } else {
      cur += ch;
    }
  }
  out.push(cur);
  return out;
}

const port = Number(process.env.MABM_GUI_API_PORT ?? 8787);
app.listen(port, () => {
  console.log(`mabm gui api listening on http://localhost:${port}`);
});
