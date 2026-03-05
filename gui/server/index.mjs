import express from "express";
import path from "node:path";
import { fileURLToPath } from "node:url";
import fs from "node:fs/promises";
import { existsSync } from "node:fs";
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

app.get("/api/live-progress", async (_req, res) => {
  try {
    const p = path.join(outputsDir, "live_progress.json");
    const raw = await fs.readFile(p, "utf-8");
    res.type("application/json").send(raw);
  } catch {
    res.json({ running: false });
  }
});

app.get("/api/runs", async (_req, res) => {
  try {
    const raw = await fs.readFile(runIndexPath, "utf-8");
    res.type("application/json").send(raw);
  } catch {
    res.json({ updated_at_utc: "", entries: [] });
  }
});

app.get("/api/configs", async (_req, res) => {
  try {
    const configsDir = path.join(repoRoot, "configs");
    const entries = await fs.readdir(configsDir, { withFileTypes: true });
    const files = entries
      .filter((e) => e.isFile() && e.name.toLowerCase().endsWith(".toml"))
      .map((e) => e.name)
      .sort((a, b) => a.localeCompare(b));

    const configs = [];
    for (const name of files) {
      const relPath = path.posix.join("configs", name);
      const absPath = path.join(configsDir, name);
      const raw = await fs.readFile(absPath, "utf-8");
      configs.push(parseConfigMetadata(raw, relPath, repoRoot));
    }

    const defaultPath =
      configs.find((c) => c.gui_load?.default)?.path ??
      configs.find((c) => c.path === "configs/phoenix_basin.toml")?.path ??
      configs[0]?.path ??
      "configs/sweep_long_transition.toml";

    res.json({ default_path: defaultPath, configs });
  } catch (e) {
    res.status(500).json({ error: String(e) });
  }
});

app.post("/api/configs/save", async (req, res) => {
  try {
    const baseConfigPath = String(req.body?.baseConfigPath ?? "").trim();
    const saveAs = String(req.body?.saveAs ?? "").trim();
    const updates = req.body?.updates ?? {};

    if (!baseConfigPath.startsWith("configs/") || !baseConfigPath.endsWith(".toml")) {
      res.status(400).json({ error: "baseConfigPath must be a configs/*.toml path" });
      return;
    }
    if (!saveAs) {
      res.status(400).json({ error: "saveAs is required" });
      return;
    }

    const targetName = normalizeTomlFileName(saveAs);
    const targetRelPath = path.posix.join("configs", targetName);
    const baseAbs = path.join(repoRoot, baseConfigPath);
    const targetAbs = path.join(repoRoot, targetRelPath);
    const rawBase = await fs.readFile(baseAbs, "utf-8");

    let raw = rawBase;
    raw = replaceTopLevelString(raw, "scenario_id", String(updates.scenario_id ?? "").trim());
    raw = replaceTableNumber(raw, "mvp", "ticks", toIntOrNull(updates.ticks));
    raw = replaceTableNumber(
      raw,
      "mvp",
      "settlement_count",
      toIntOrNull(updates.settlement_count)
    );
    raw = replaceTableNumber(raw, "mvp", "base_population", toIntOrNull(updates.base_population));
    raw = replaceTableNumber(raw, "mvp", "seed", toIntOrNull(updates.seed));
    raw = replaceTableNumber(
      raw,
      "mvp.gui",
      "live_update_every_ticks",
      toIntOrNull(updates.live_update_every_ticks)
    );
    raw = replaceTableNumber(
      raw,
      "mvp.spatial",
      "hex_diameter_km",
      toFloatOrNull(updates.hex_diameter_km)
    );
    raw = replaceTableNumber(
      raw,
      "mvp.spatial",
      "flat_travel_km_per_day",
      toFloatOrNull(updates.flat_travel_km_per_day)
    );
    raw = replaceTableBool(
      raw,
      "mvp.spatial",
      "use_gis_hex_inputs",
      toBoolOrNull(updates.use_gis_hex_inputs)
    );
    raw = replaceTableString(
      raw,
      "mvp.spatial",
      "gis_hex_csv_path",
      toStringOrNull(updates.gis_hex_csv_path)
    );

    await fs.writeFile(targetAbs, raw, "utf-8");
    res.json({ ok: true, path: targetRelPath });
  } catch (e) {
    res.status(500).json({ error: String(e) });
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

    const baselinePopulationSeries = popSeries.map((r) => ({
      year: Number(r.year ?? 0),
      population_total: Number(r.population_total ?? 0)
    }));
    const fallbackPopulationSeries = aggregatePopulationFromSettlements(settlementRows);
    const populationSeries =
      baselinePopulationSeries.length >= 2 ? baselinePopulationSeries : fallbackPopulationSeries;

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
        hex_id: Number(r.hex_id ?? 0),
        grid_q: Number(r.grid_q ?? 0),
        grid_r: Number(r.grid_r ?? 0),
        population_total: Number(r.population_total ?? 0),
        households: Number(r.households ?? 0),
        climate_pdsi: Number(r.climate_pdsi ?? 0),
        drought_index_5y: Number(r.drought_index_5y ?? 0),
        water_reliability: Number(r.water_reliability ?? 0),
        water_quality: Number(r.water_quality ?? 0),
        fuel_stock: Number(r.fuel_stock ?? 0),
        food_yield_kcal: Number(r.food_yield_kcal ?? 0),
        food_stores_kcal: Number(r.food_stores_kcal ?? 0),
        food_deficit_kcal: Number(r.food_deficit_kcal ?? 0),
        food_capacity_persons: Number(r.food_capacity_persons ?? 0),
        hex_quality: Number(r.hex_quality ?? 0),
        stress_composite: Number(r.stress_composite ?? 0),
        defensibility: Number(r.defensibility ?? 0),
        burden_multiplier: Number(r.burden_multiplier ?? 0),
        disease_infected_share: Number(r.disease_infected_share ?? 0),
        is_active: String(r.is_active).toLowerCase() === "true",
        status: String(r.status ?? "")
      }));

    res.json({
      run_id: manifest.run_id,
      hex_count: Number(manifest.summary?.hex_count ?? manifest.summary?.settlement_count ?? 0),
      latest_tick: latestTick,
      population_series: populationSeries,
      settlements_latest: latestSettlements
    });
  } catch (e) {
    res.status(500).json({ error: String(e) });
  }
});

app.post("/api/run", async (req, res) => {
  const configPath = req.body?.configPath ?? "configs/phoenix_basin.toml";
  const ticksOverride = Number(req.body?.ticksOverride ?? 0);
  const liveUpdateEveryTicks = Number(req.body?.liveUpdateEveryTicks ?? 0);
  const seedOverride = Number(req.body?.seedOverride ?? 0);
  let runConfigPath = configPath;
  let tempConfigPath = null;
  if (ticksOverride > 0 || liveUpdateEveryTicks > 0 || seedOverride > 0) {
    try {
      const srcPath = path.isAbsolute(configPath) ? configPath : path.join(repoRoot, configPath);
      const raw = await fs.readFile(srcPath, "utf-8");
      let patched = raw;
      if (ticksOverride > 0) {
        patched = patched.replace(/^ticks\s*=\s*\d+/m, `ticks = ${Math.floor(ticksOverride)}`);
      }
      if (seedOverride > 0) {
        patched = patched.replace(/^seed\s*=\s*\d+/m, `seed = ${Math.floor(seedOverride)}`);
      }
      if (/^\[mvp\.gui\]/m.test(patched)) {
        patched = patched.replace(
          /^live_update_every_ticks\s*=\s*\d+/m,
          `live_update_every_ticks = ${Math.max(0, Math.floor(liveUpdateEveryTicks))}`
        );
      } else {
        patched += `\n[mvp.gui]\nlive_update_every_ticks = ${Math.max(0, Math.floor(liveUpdateEveryTicks))}\n`;
      }
      tempConfigPath = path.join(repoRoot, "outputs", `_tmp_gui_run_${Date.now()}.toml`);
      await fs.writeFile(tempConfigPath, patched, "utf-8");
      runConfigPath = tempConfigPath;
    } catch (e) {
      res.status(400).json({ error: `failed to patch config: ${String(e)}` });
      return;
    }
  }

  const args = ["run", "--quiet", "--", runConfigPath];
  const child = spawn("cargo", args, { cwd: repoRoot, shell: true });

  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (d) => (stdout += d.toString()));
  child.stderr.on("data", (d) => (stderr += d.toString()));

  child.on("close", (code) => {
    if (tempConfigPath) {
      fs.unlink(tempConfigPath).catch(() => {});
    }
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

function parseConfigMetadata(raw, relPath, rootDir) {
  const scenarioId = firstGroup(raw, /^scenario_id\s*=\s*"([^"]+)"/m) ?? "";
  const mvp = parseTableBody(raw, "mvp");
  const spatial = parseTableBody(raw, "mvp.spatial");
  const gui = parseTableBody(raw, "mvp.gui");
  const guiLoad = parseTableBody(raw, "gui_load");
  const useGisHexInputs = parseTomlBool(spatial, "use_gis_hex_inputs") ?? false;
  const gisHexCsvPath = parseTomlString(spatial, "gis_hex_csv_path");
  const description = parseTomlString(guiLoad, "description") ?? "";
  const label =
    parseTomlString(guiLoad, "label") ??
    (scenarioId ? `${scenarioId} (${relPath})` : relPath);
  const isDefault = parseTomlBool(guiLoad, "default") ?? false;
  const requiredFilesFromGui = parseTomlStringArray(guiLoad, "required_files");
  const requiredFilePaths =
    requiredFilesFromGui.length > 0
      ? requiredFilesFromGui
      : useGisHexInputs && gisHexCsvPath
        ? [gisHexCsvPath]
        : [];
  const required_files = requiredFilePaths.map((p) => {
    const abs = path.isAbsolute(p) ? p : path.join(rootDir, p);
    return { path: p, exists: fileExistsSync(abs) };
  });

  const run_defaults = {
    scenario_id: scenarioId,
    ticks: parseTomlNumber(mvp, "ticks") ?? 0,
    settlement_count: parseTomlNumber(mvp, "settlement_count") ?? 0,
    base_population: parseTomlNumber(mvp, "base_population") ?? 0,
    seed: parseTomlNumber(mvp, "seed") ?? 0,
    hex_diameter_km: parseTomlNumber(spatial, "hex_diameter_km") ?? 0,
    flat_travel_km_per_day: parseTomlNumber(spatial, "flat_travel_km_per_day") ?? 0,
    live_update_every_ticks: parseTomlNumber(gui, "live_update_every_ticks") ?? 0,
    use_gis_hex_inputs: useGisHexInputs,
    gis_hex_csv_path: gisHexCsvPath ?? ""
  };

  return {
    path: relPath,
    scenario_id: scenarioId,
    label,
    use_gis_hex_inputs: useGisHexInputs,
    gis_hex_csv_path: gisHexCsvPath ?? "",
    gui_load: {
      description,
      default: isDefault,
      required_files
    },
    run_defaults
  };
}

function parseTableBody(raw, tableName) {
  const esc = tableName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(
    `^\\[${esc}\\]\\s*\\r?\\n([\\s\\S]*?)(?=^\\[[^\\]]+\\]\\s*$|(?![\\s\\S]))`,
    "m"
  );
  const m = raw.match(re);
  return m ? m[1] : "";
}

function parseTomlString(sectionBody, key) {
  const re = new RegExp(`^\\s*${escapeRegExp(key)}\\s*=\\s*"([^"]*)"\\s*$`, "m");
  const m = sectionBody.match(re);
  return m ? m[1] : null;
}

function parseTomlBool(sectionBody, key) {
  const re = new RegExp(`^\\s*${escapeRegExp(key)}\\s*=\\s*(true|false)\\s*$`, "mi");
  const m = sectionBody.match(re);
  if (!m) return null;
  return m[1].toLowerCase() === "true";
}

function parseTomlNumber(sectionBody, key) {
  const re = new RegExp(`^\\s*${escapeRegExp(key)}\\s*=\\s*(-?\\d+(?:\\.\\d+)?)\\s*$`, "m");
  const m = sectionBody.match(re);
  if (!m) return null;
  const n = Number(m[1]);
  return Number.isFinite(n) ? n : null;
}

function parseTomlStringArray(sectionBody, key) {
  const re = new RegExp(`^\\s*${escapeRegExp(key)}\\s*=\\s*\\[(.*)\\]\\s*$`, "m");
  const m = sectionBody.match(re);
  if (!m) return [];
  const body = m[1].trim();
  if (!body) return [];
  return body
    .split(",")
    .map((s) => s.trim())
    .map((s) => s.replace(/^"(.*)"$/, "$1"))
    .filter((s) => s.length > 0);
}

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function firstGroup(raw, re) {
  const m = raw.match(re);
  return m ? m[1] : null;
}

function fileExistsSync(p) {
  return existsSync(p);
}

function normalizeTomlFileName(name) {
  const base = name.toLowerCase().endsWith(".toml") ? name.slice(0, -5) : name;
  const safe = base.replace(/[^a-zA-Z0-9._-]+/g, "_").replace(/^_+|_+$/g, "");
  return `${safe || "config_custom"}.toml`;
}

function toIntOrNull(v) {
  const n = Number(v);
  if (!Number.isFinite(n)) return null;
  return Math.floor(n);
}

function toFloatOrNull(v) {
  const n = Number(v);
  if (!Number.isFinite(n)) return null;
  return n;
}

function toBoolOrNull(v) {
  if (typeof v === "boolean") return v;
  if (typeof v === "string") {
    if (v.toLowerCase() === "true") return true;
    if (v.toLowerCase() === "false") return false;
  }
  return null;
}

function toStringOrNull(v) {
  if (v === undefined || v === null) return null;
  const s = String(v).trim();
  return s.length > 0 ? s : null;
}

function replaceTopLevelString(raw, key, value) {
  if (!value) return raw;
  const re = new RegExp(`^${escapeRegExp(key)}\\s*=\\s*"[^"]*"\\s*$`, "m");
  const line = `${key} = "${escapeTomlString(value)}"`;
  return re.test(raw) ? raw.replace(re, line) : `${line}\n${raw}`;
}

function replaceTableString(raw, table, key, value) {
  if (value === null) return raw;
  return replaceTableValue(raw, table, key, `"${escapeTomlString(value)}"`);
}

function replaceTableBool(raw, table, key, value) {
  if (value === null) return raw;
  return replaceTableValue(raw, table, key, value ? "true" : "false");
}

function replaceTableNumber(raw, table, key, value) {
  if (value === null) return raw;
  const rendered = Number.isInteger(value) ? String(value) : String(value);
  return replaceTableValue(raw, table, key, rendered);
}

function replaceTableValue(raw, table, key, valueLiteral) {
  const body = parseTableBody(raw, table);
  const headerRe = new RegExp(`^\\[${escapeRegExp(table)}\\]\\s*\\r?\\n`, "m");
  if (!headerRe.test(raw)) {
    return `${raw}\n[${table}]\n${key} = ${valueLiteral}\n`;
  }
  const keyRe = new RegExp(`^\\s*${escapeRegExp(key)}\\s*=\\s*.*$`, "m");
  const nextBody = keyRe.test(body)
    ? body.replace(keyRe, `${key} = ${valueLiteral}`)
    : `${body}${body.endsWith("\n") ? "" : "\n"}${key} = ${valueLiteral}\n`;
  const tableRe = new RegExp(
    `(^\\[${escapeRegExp(table)}\\]\\s*\\r?\\n)([\\s\\S]*?)(?=^\\[[^\\]]+\\]\\s*$|(?![\\s\\S]))`,
    "m"
  );
  return raw.replace(tableRe, `$1${nextBody}`);
}

function escapeTomlString(s) {
  return s.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function aggregatePopulationFromSettlements(settlementRows) {
  const byTick = new Map();
  for (const r of settlementRows) {
    const tick = Number(r.tick ?? 0);
    const year = Number(r.year ?? 0);
    const pop = Number(r.population_total ?? 0);
    const cur = byTick.get(tick);
    if (!cur) {
      byTick.set(tick, { year, population_total: pop });
    } else {
      cur.population_total += pop;
      if (year > cur.year) cur.year = year;
    }
  }
  return [...byTick.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([, v]) => v);
}

const port = Number(process.env.MABM_GUI_API_PORT ?? 8787);
app.listen(port, () => {
  console.log(`mabm gui api listening on http://localhost:${port}`);
});
