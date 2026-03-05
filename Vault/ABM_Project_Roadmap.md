# ABM Project Roadmap

Goal: Build a high-scale historical agent-based model in Rust with strong ABM validity and performance engineering discipline.

## 1. Background Research

1. Define research scope and target phenomena.
2. Identify candidate ABM paradigms:
   - Discrete-time vs event-driven.
   - Spatial vs network interaction models.
3. Build a variable inventory from literature:
   - Agent state variables.
   - Environment/state transition variables.
   - Exogenous shock variables.
4. Define calibration and validation targets:
   - Stylized facts.
   - Historical benchmarks/time series.
5. Specify modeling assumptions explicitly:
   - Behavioral rules.
   - Information availability constraints.
6. Define performance requirements:
   - Agent count targets.
   - Tick throughput targets.
   - Runtime/memory limits per experiment.
7. Create a data governance plan:
   - Source provenance.
   - Data cleaning/reproducibility.
8. Produce a model specification document (living spec):
   - ODD-inspired structure (Overview, Design concepts, Details).

## 2. MVP Simulation at Scale

1. Implement minimal simulation kernel in Rust:
   - ECS-like data layout or SoA storage for cache locality.
   - Deterministic tick loop with fixed seed handling.
2. Define minimal agent rule set:
   - Few core states and transitions only.
3. Add scalable execution architecture:
   - Partitioned agent updates.
   - Parallel processing with clear synchronization boundaries.
4. Build high-throughput I/O strategy:
   - Buffered logging.
   - Sparse checkpointing.
   - Columnar-friendly output where useful.
5. Add observability and profiling hooks:
   - Tick time breakdown.
   - Memory usage snapshots.
   - Hotspot tracing.
6. Build correctness tests early:
   - Invariant/property tests.
   - Determinism regression tests.
7. Run scale trials:
   - 10k, 100k, 1M agents (or staged targets).
   - Document bottlenecks and optimize.
8. Establish MVP acceptance gate:
   - Meets performance baseline.
   - Produces plausible macro dynamics.

## 3. Add Necessary Variables

1. Prioritize variable additions by marginal explanatory value.
2. Add variables in small batches with feature flags.
3. For each variable batch:
   - Implement state representation.
   - Implement transition effects.
   - Add unit and invariance tests.
   - Add metrics to assess impact.
4. Recalibrate after each batch:
   - Keep baseline comparability.
   - Track fit improvement vs complexity increase.
5. Control model complexity:
   - Remove redundant variables/rules.
   - Avoid overfitting to one historical episode.
6. Maintain schema/versioning for saved runs.
7. Update model documentation continuously:
   - Assumptions.
   - New equations/rules.
   - Expected emergent behavior.

## 4. Parameter Sweeps

1. Define experiment matrix:
   - Core parameters.
   - Bounds/priors.
   - Resolution strategy (grid, Latin hypercube, Bayesian search).
2. Build reproducible experiment runner:
   - Seed policy.
   - Run metadata and config hashes.
3. Parallelize sweeps safely:
   - Multi-process or distributed workers.
   - Isolate runs and deterministic replay.
4. Implement result storage and indexing:
   - Structured outputs with run IDs.
   - Fast post-hoc aggregation.
5. Define analysis metrics:
   - Fit-to-history metrics.
   - Stability/sensitivity metrics.
   - Runtime/cost metrics.
6. Identify robust parameter regions:
   - Not single-point best fits.
   - Stability under perturbation.
7. Validate out-of-sample scenarios.
8. Package findings:
   - Ranked candidate parameter sets.
   - Confidence intervals and caveats.

## Cross-Cutting Best Practices

1. Keep simulation core deterministic and side-effect disciplined.
2. Prefer data-oriented design and minimize allocations in hot loops.
3. Use profiling-guided optimization only after correctness.
4. Separate model logic, execution engine, and analysis pipeline.
5. Use strict reproducibility:
   - Versioned configs.
   - Fixed seeds.
   - Environment capture.
6. Treat validation as continuous, not end-stage.
7. Automate CI checks for performance regressions and deterministic drift.
