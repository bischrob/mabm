# Parallelization Audit and Implementation

Date: 2026-03-04

## Why this note exists

This note records the parallel execution strategy for the MVP so large parameter sweeps can run quickly without sacrificing reproducibility, and it documents which systems remain intentionally serial until post-MVP data-layout refactors.

## Implemented now

1. Parallel sweep execution:
   - `run_sweep` now builds explicit run jobs and executes them in parallel using Rayon.
   - Each job uses its own deterministic seed and config clone, so runs are independent and thread-safe.
   - Output rows are sorted by `run_index` after execution to keep deterministic ordering.

2. Config controls:
   - `sweep.parallel_enabled` toggles threaded sweep execution.
   - `sweep.max_parallel_workers` optionally caps thread count via a local Rayon thread pool.

3. Parallel output collection hotspots:
   - Trait-frequency row collection is parallelized across settlements.
   - Trait-deposition row collection is parallelized across settlements.
   - Network snapshot edge construction is parallelized across pair-index bands.

## Reviewed and intentionally serial for MVP

1. Tick engine subsystem updates (`engine.rs`) remain serial.
   - Reason: updates have strict causal ordering and currently mutate `HashMap` settlement state directly.
   - Parallel mutation here would require either locking (bad for performance) or a data-layout refactor.

2. In-tick deposition accumulation remains serial.
   - Reason: mutable iteration over shared settlement map and limited runtime share versus sweep-level cost.

3. Synthetic state initialization remains serial.
   - Reason: one-time startup cost; not a dominant bottleneck versus full simulation ticks and sweeps.

## Performance-oriented next refactor (post-MVP)

1. Replace settlement `HashMap` with stable vector-backed storage + id-index mapping.
2. Convert subsystems to data-oriented passes over contiguous arrays.
3. Parallelize per-subsystem settlement updates with `par_iter_mut` where no cross-settlement writes occur.
4. Keep deterministic reduction rules for global/regional aggregates.

## Reproducibility safeguards

1. Sweep job seeds are pre-assigned before execution.
2. Each run creates independent simulation state.
3. Final sweep summaries are sorted by run index before optional snapshot filtering.
