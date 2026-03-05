# GUI Vite Backend Manifest Implementation

Date: 2026-03-05

## Why this exists

The GUI needs a stable discovery contract for run outputs and a clean execution boundary where Rust remains the simulation engine. This implementation adds both.

## Implemented

1. Rust run manifest/index:
   - Per-run JSON manifest written to `outputs/<scenario>_<run_id>_manifest.json`
   - Global index written to `outputs/run_index.json`
   - Includes output file paths and summary row counts

2. Rust stays backend engine:
   - CLI remains the single execution path (`cargo run -- <config>`)
   - No simulation logic moved into JavaScript

3. Vite GUI + thin API wrapper:
   - `gui/` contains React/Vite frontend for run list + manifest details
   - `gui/server/index.mjs` exposes:
     - `GET /api/runs`
     - `GET /api/runs/:runId`
     - `POST /api/run` (triggers Rust CLI run)

## Verified artifacts

- `outputs/run_index.json` now updates after runs.
- `outputs/*_manifest.json` generated and contains output paths.

## How to run

From `gui/`:

```bash
npm install
npm run api
```

In another terminal:

```bash
npm run dev
```
