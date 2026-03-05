# Synthetic Climate Generator Implementation

Purpose:

- Provide deterministic seasonal climate forcing for synthetic-data MVP runs before GIS/paleoclimate ingestion.

Implemented:

1. New climate module with configurable generator:
   - [climate.rs](C:\Users\rjbischo\Documents\mabm\src\climate.rs)
2. Climate forcing stored in simulation state:
   - [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
3. Engine climate update now consumes forcing series each tick:
   - [engine.rs](C:\Users\rjbischo\Documents\mabm\src\engine.rs)
4. MVP builder now generates forcing from config and seed:
   - [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
5. Configurable climate section added to default TOML:
   - [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)

Generator characteristics:

1. Long and medium cycles (sinusoidal terms).
2. AR(1)-style interannual persistence.
3. Random shocks with configurable probability and duration.
4. Global regional forcing with per-settlement local multiplier/offset.

Validation:

1. Determinism test added:
   - same seed + config -> identical series.
2. Full test suite passes.
3. `cargo run` produces normal output artifact with climate forcing active.
