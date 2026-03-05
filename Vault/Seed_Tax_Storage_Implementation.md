# Seed Tax Storage Implementation

Purpose:

- Enforce agricultural persistence constraints by reserving a seed fraction from each seasonal yield before consumption.

Implemented behavior:

1. Seed reservation:
   - `seed_reserve_kcal = gross_yield * sigma_seed`
   - `usable_yield = gross_yield - seed_reserve_kcal`
2. Storage update includes spoilage from config.
3. Desperation branch:
   - if `usable_yield + stores` is insufficient, optionally draw seed reserve.
4. Carryover penalty:
   - seed draw reduces `next_yield_multiplier` for future productivity.
5. Emergency flag:
   - if deficit remains after seed draw, `emergency_reciprocity_last_tick = true`.

Config surface:

1. `mvp.storage.sigma_seed`
2. `mvp.storage.spoilage_rate`
3. `mvp.storage.allow_seed_draw`
4. `mvp.storage.enable_emergency_reciprocity`

Code changes:

1. [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
   - `FoodState` diagnostics and carryover field
   - `StoragePolicy` in `SimulationState`
2. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
   - `StorageConfig` and policy wiring
3. [engine.rs](C:\Users\rjbischo\Documents\mabm\src\engine.rs)
   - seed-tax logic in `update_food_and_storage`
4. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
   - validation for storage parameters
5. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)
   - default storage settings

Validation:

1. Unit test verifies desperation branch, emergency flag, and carryover penalty.
2. Full `cargo test` passes.
3. `cargo run` succeeds with updated config and output generation.
