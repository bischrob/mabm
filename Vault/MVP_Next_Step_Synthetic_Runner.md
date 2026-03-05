# MVP Next Step Synthetic Runner

Decision:

- For simulated-data-only MVP, the highest-priority step is an end-to-end executable path:
  - synthetic initialization -> seasonal tick execution -> trait-frequency output snapshots.

Why this is the right next step:

1. It validates architecture now, before GIS integration.
2. It produces analyzable output (`settlement_trait_frequency`) aligned with current output strategy.
3. It exposes performance and schema issues early.

Implemented code:

1. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
   - `MvpRunConfig`
   - deterministic synthetic state generator
   - run loop with snapshot cadence
2. [output.rs](C:\Users\rjbischo\Documents\mabm\src\output.rs)
   - `SettlementTraitFrequencyRow`
   - row collector for sparse trait frequencies
3. [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
   - added household count and per-settlement trait counts
4. [lib.rs](C:\Users\rjbischo\Documents\mabm\src\lib.rs)
   - module exports + new runner test
5. [main.rs](C:\Users\rjbischo\Documents\mabm\src\main.rs)
   - demonstrates default MVP run execution.

Validation:

- `cargo test` passing with `mvp_runner_emits_trait_rows`.

Immediate next implementation targets:

1. add config file loading for run/scenario parameters,
2. replace synthetic climate/resource placeholders with synthetic generators that mimic drought cycles,
3. add Parquet writer path for larger calibration runs.

Progress update:

- Implemented CSV writer path for `SettlementTraitFrequencyRow` snapshots and validated with automated test.
