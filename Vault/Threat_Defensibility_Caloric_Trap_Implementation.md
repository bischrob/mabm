# Threat Defensibility Caloric Trap Implementation

Purpose:

- Encode defensive-site tradeoffs so high-defensibility locations become costly to sustain and only favored under elevated regional threat.

Implemented behavior:

1. Regional threat index:
   - computed each tick from weighted drought, conflict memory, and food stress.
2. Settlement burden multiplier:
   - `burden = 1 + k * threat * defensibility`, clamped to `[1.0, 3.0]`.
3. Burden effects:
   - increases Tier 1 survival labor requirement,
   - increases Tier 2 subsistence labor requirement,
   - increases seasonal caloric requirement in storage/consumption accounting.

Config surface (`mvp.threat`):

1. `drought_weight`
2. `conflict_weight`
3. `food_weight`
4. `defensibility_cost_k`

Code changes:

1. [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
   - `ThreatPolicy` and regional threat state
   - per-settlement `defensibility` and `burden_multiplier`
2. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
   - threat config struct and policy wiring
   - synthetic defensibility initialization
3. [engine.rs](C:\Users\rjbischo\Documents\mabm\src\engine.rs)
   - regional threat computation stage
   - burden application in labor and food-storage updates
4. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
   - threat parameter validation
5. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)
   - default threat parameters

Validation:

1. Added unit test:
   - high-defensibility settlement gets higher burden under positive threat.
2. Full test suite passes.
3. `cargo run` succeeds with updated config and output generation.
