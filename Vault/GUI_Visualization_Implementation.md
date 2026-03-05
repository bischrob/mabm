# GUI Visualization Implementation

Date: 2026-03-05

## What was added

1. Settlement snapshot output artifact:
   - `*_settlement_snapshot.csv`
   - Includes `tick`, `year`, `settlement_id`, `grid_q`, `grid_r`, `population_total`, and active/abandoned status.

2. Manifest/index integration:
   - Manifests now include `settlement_snapshot_csv`.
   - Manifest summaries include `settlement_snapshot_rows`.

3. API endpoint for visualization data:
   - `GET /api/runs/:runId/visuals`
   - Returns:
     - population time-series from baseline metrics
     - latest settlement snapshot for map rendering

4. Frontend visualization panels:
   - Hex-grid settlement map (active vs abandoned colors, population marker size).
   - Population line chart across simulation years.

## Why this exists

This gives the GUI a direct visual readout of settlement spatial structure and demographic trend dynamics without changing simulation core logic.
