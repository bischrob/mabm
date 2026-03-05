# GUI Live Update Controls Implementation

Date: 2026-03-05

## Features added

1. GUI update frequency control (ticks):
   - New runtime knob: `live_update_every_ticks`
   - Config path: `[mvp.gui]`
   - `0` disables live progress writes.

2. Live progress channel:
   - Rust writes `outputs/live_progress.json` every `live_update_every_ticks`.
   - API endpoint: `GET /api/live-progress`.
   - Frontend polls while a run is active and updates visuals in place.

3. GUI run-length control:
   - New UI numeric control for ticks override.
   - `POST /api/run` patches a temporary config and runs with overridden `ticks`.

4. Visualization improvements:
   - Population chart now has x-axis and y-axis tick labels.
   - Hex map now includes active/abandoned legend.

## Notes

- Live updates are coarse-grained by design to preserve simulation performance.
- The override patching is non-destructive: original config files are unchanged.
