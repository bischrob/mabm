# Rust Tick Pipeline Skeleton

Implemented code:

1. [Cargo.toml](C:\Users\rjbischo\Documents\mabm\Cargo.toml)
2. [lib.rs](C:\Users\rjbischo\Documents\mabm\src\lib.rs)
3. [main.rs](C:\Users\rjbischo\Documents\mabm\src\main.rs)

## What Is Implemented

1. Deterministic seasonal scheduler:
   - `Season::{Spring, Summer, Autumn, Winter}` via `tick % 4`.
2. Tick engine with explicit update order:
   - climate -> water/fuel -> labor -> food/storage -> disease -> conflict -> migration/fission -> trade -> demography.
3. Multi-hazard coupling:
   - per-settlement stress components,
   - bounded composite stress with selected pairwise interaction terms.
4. Core state structs:
   - climate, water, fuel, food, disease, conflict, labor, settlement, simulation.
5. Basic tests:
   - seasonal cycle correctness,
   - composite stress bounded in `[0,1]`.

## Why This Matches Current ABM Constraints

1. Seasonal time step is first-class.
2. Coupling is settlement-scale for performance.
3. Deterministic stage order supports reproducibility and parallel-safe staging.

## Next Code Steps

1. Replace placeholders with real subsystem equations from Vault notes.
2. Add parallel stage execution:
   - thread-local deltas + deterministic merge.
3. Add output emitter for `settlement_trait_frequency`.
4. Add config loading and scenario profiles.
