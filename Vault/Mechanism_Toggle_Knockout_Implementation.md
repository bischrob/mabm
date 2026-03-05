# Mechanism Toggle Knockout Implementation

Purpose:

- Enable controlled knockout experiments by switching specific mechanisms on/off per run and within sweep scenarios.

Implemented:

1. Mechanism toggle policy in simulation state:
   - `seed_tax_storage`
   - `threat_defensibility`
   - `cultural_transmission`
   - `water_quality_disease_coupling`
2. Toggle-aware engine behavior:
   - disabled threat-defensibility -> burden multiplier fixed at 1.0
   - disabled seed-tax -> no seed reserve/draw branch
   - disabled cultural transmission -> no trait update stage
   - disabled water-quality coupling -> disease multiplier fixed at 1.0
3. Sweep knockout variants:
   - baseline and knockout variants run side-by-side
   - summary includes `knockout` label for attribution

Code:

1. [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
2. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
3. [engine.rs](C:\Users\rjbischo\Documents\mabm\src\engine.rs)
4. [sweep.rs](C:\Users\rjbischo\Documents\mabm\src\sweep.rs)
5. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
6. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)
7. [sweep.toml](C:\Users\rjbischo\Documents\mabm\configs\sweep.toml)

Validation:

1. Unit test verifies cultural transmission knockout keeps trait counts unchanged.
2. Sweep run confirms knockout-labeled summary rows are produced.
