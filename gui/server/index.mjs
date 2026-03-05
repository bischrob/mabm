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

const port = Number(process.env.MABM_GUI_API_PORT ?? 8787);
app.listen(port, () => {
  console.log(`mabm gui api listening on http://localhost:${port}`);
});
