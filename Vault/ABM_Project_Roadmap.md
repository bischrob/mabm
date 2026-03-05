# ABM Project Roadmap

Goal:

- Build a high-scale seasonal ABM in Rust to study long-run demographics, settlement dynamics, and interaction networks in a semi-arid macro-region.

## Status Key

- `[x]` completed
- `[~]` in progress
- `[ ]` not started

## Completed

1. `[x]` Core research architecture documented:
   - seasonal time model
   - hex travel friction
   - carrying capacity streams
   - water/fuel/labor constraints
   - conflict/trade/fission frameworks
2. `[x]` Validation framework documented:
   - generative sufficiency
   - pattern-oriented modeling
   - sensitivity and knockout strategy
3. `[x]` Rust base framework implemented:
   - modular crate layout (`model`, `engine`, `mvp`, `output`, `utils`, `versioning`)
   - deterministic seasonal tick pipeline
   - multi-hazard composite stress hook
4. `[x]` Date-time run versioning implemented:
   - UTC run id and code version in state metadata
5. `[x]` Simulated-data MVP runner implemented:
   - synthetic initialization
   - seasonal simulation loop
6. `[x]` Minimal cultural output path implemented:
   - `settlement_trait_frequency` collection
   - CSV writer + tests
7. `[x]` Configuration system for scenarios and parameters implemented:
   - file-based TOML loading
   - config validation
   - SHA-256 config hash for run metadata
8. `[x]` Synthetic climate generator implemented:
   - deterministic seeded regional PDSI forcing
   - cyclical variability + AR(1) interannual noise
   - multi-year shock events (drought-like pulses)
9. `[x]` Seed-tax storage logic implemented:
   - configurable `sigma_seed` and spoilage in scenario config
   - desperation seed draw branch
   - emergency reciprocity flag when residual shortfall persists
10. `[x]` Threat-defensibility caloric trap implemented:
   - regional threat index from drought/conflict/food stress
   - defensibility-linked burden multiplier
   - burden applied to survival/subsistence labor and caloric requirement
11. `[x]` Cultural transmission update implemented:
   - neutral drift + conformist + prestige pull at settlement level
   - deterministic jitter and bounded per-tick trait-step control
   - trait counts clamped to household bounds
12. `[x]` Optional validation outputs implemented:
   - deposited trait accumulator snapshots
   - network interaction snapshot ledger
   - config-gated emission for validation runs only
13. `[x]` Experiment runner for sweeps implemented:
   - config-driven batch parameter runs
   - seed policy support (fixed/incremental/list)
   - sweep summary metric CSV output

## In Progress (Current MVP Build)

1. `[~]` Replace placeholder subsystem math in engine with configured equations:
   - storage with seed reserve
   - settlement stress terms
   - threat-defensibility cost coupling
2. `[~]` Keep outputs minimal-by-default:
   - trait-frequency as standard run artifact
   - optional validation outputs gated by config

## Next (Highest Priority)

1. `[ ]` Add mechanism toggle system for knockout experiments.

## Next After That

1. `[ ]` Add baseline metric pack:
   - population trend
   - aggregation/abandonment counts
   - network structure metrics

## MVP Acceptance Gate

MVP is done when all are true:

1. `[ ]` Simulated-data runs complete reproducibly from config.
2. `[ ]` Core mechanisms run without placeholder math for:
   - food/storage
   - water/fuel/labor stress
   - migration/fission pressure
   - cultural trait dynamics
3. `[ ]` Trait-frequency CSV output is stable and analysis-ready.
4. `[ ]` At least one 1,000-year synthetic run completes at target scale without crash.
5. `[ ]` At least one sensitivity mini-sweep and one knockout run complete.

## Deferred Until Post-MVP

1. `[ ]` GIS ingestion pipeline and high-res-to-hex aggregation.
2. `[ ]` Archaeological empirical calibration datasets.
3. `[ ]` Distributed execution and large sweep orchestration.
4. `[ ]` Rich GUI exploration and interactive scenario controls.

## Working Rule

- Default to synthetic-data-first, reproducible runs until MVP acceptance gate is met, then expand to GIS and empirical calibration.
