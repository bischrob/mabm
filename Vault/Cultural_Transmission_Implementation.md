# Cultural Transmission Implementation

Purpose:

- Add bounded, scalable cultural dynamics to settlement traits during seasonal simulation.

Implemented behavior:

Per settlement and trait each tick:

1. Neutral drift term:
   - pulls frequency toward midpoint at `neutral_drift_rate`.
2. Conformist term:
   - reinforces majority traits locally with `conformist_strength`.
3. Prestige term:
   - pulls toward prestige-weighted regional trait frequency at `prestige_rate`.
4. Deterministic jitter:
   - seeded pseudo-noise (`simulation_seed`, `tick`, `settlement_id`, `trait_id`) scaled by `jitter_scale`.
5. Bounded change:
   - per-tick frequency change clamped by `max_trait_step_per_tick`.
6. Safety bound:
   - trait counts clamped to `[0, households]`.

Config surface (`mvp.culture`):

1. `neutral_drift_rate`
2. `conformist_strength`
3. `prestige_rate`
4. `jitter_scale`
5. `max_trait_step_per_tick`

Code changes:

1. [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
   - `CulturalPolicy` and `simulation_seed` on state
2. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
   - `CultureConfig` + policy wiring
3. [engine.rs](C:\Users\rjbischo\Documents\mabm\src\engine.rs)
   - cultural update stage in tick loop
4. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
   - validation rules for culture parameters
5. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)
   - default culture settings

Validation:

1. Added test to ensure cultural updates keep trait counts bounded.
2. Full test suite passes.
3. Runtime output generation remains stable.
