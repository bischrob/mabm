# Code Architecture Conventions

Purpose:

- Keep ABM codebase maintainable as subsystem complexity increases.

Conventions implemented:

1. Date-time run versioning:
   - `RunVersion` includes `code_version`, `started_at_utc` (RFC3339 UTC), and `run_id`.
2. Modular structure:
   - `model.rs`: shared state and domain types.
   - `engine.rs`: deterministic seasonal execution order and coupling logic.
   - `utils.rs`: reusable normalization/helpers.
   - `versioning.rs`: run/version metadata.
   - `lib.rs`: public API and integration tests.
3. Documentation style:
   - Comments explain why a section exists in the ABM framework (causal clarity, reproducibility, calibration support), not trivial line-by-line behavior.

Why this matters:

1. Long-horizon ABMs require reproducibility across sweeps and scenario runs.
2. Subsystem isolation reduces integration bugs during iterative expansion.
3. Causality-focused docs make calibration and interpretation faster.
