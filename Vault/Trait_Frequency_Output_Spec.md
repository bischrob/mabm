# Trait Frequency Output Spec

Goal:

- Output only trait frequencies per settlement.

## Single Required Table

Name:

- `settlement_trait_frequency`

Grain:

- one row per `run_id`, `tick`, `settlement_id`, `trait_id` with non-zero count

Required fields:

1. `run_id`
2. `tick`
3. `year`
4. `settlement_id`
5. `trait_id` (0..63 if `u64` bitset index)
6. `trait_count`
7. `trait_frequency` (0.0-1.0)
8. `population_total` (for normalization checks)

Computation:

1. `trait_count = number of households in settlement with bit trait_id = 1`
2. `trait_frequency = trait_count / households_in_settlement`

Write cadence:

1. default every seasonal tick.
2. optional downsample every 2-4 ticks for long runs.

Performance rules:

1. Emit sparse rows only (`trait_count > 0`).
2. Do not write full household trait vectors.
3. Use Parquet with partition keys: `run_id`, coarse `tick_bucket`.

Minimum companion metadata:

1. `run_id`
2. `seed`
3. `scenario_id`
4. `config_hash`

This is sufficient for:

- within-settlement trait composition,
- between-settlement style-distance calculations,
- temporal convergence/divergence analysis.

## Optional Archaeological Deposition Table

If comparing to excavated assemblages, add:

- `settlement_trait_deposition`

Fields:

1. `run_id`
2. `tick`
3. `settlement_id`
4. `trait_id`
5. `deposited_count`
6. `cumulative_deposited_count`

Why:

- living trait frequency (systemic inventory) and deposited assemblage are different observables.

## Optional Network Edge Snapshot Ledger

If reconstructing topology, add:

- `network_interaction_snapshot`

Fields:

1. `run_id`
2. `tick`
3. `source_settlement_id`
4. `target_settlement_id`
5. `edge_type` (`kin`, `trade`, `alliance`)
6. `weight`
7. `goods_exchanged_kcal`

Performance:

- emit snapshots at interval (e.g., every 40 ticks = 10 years), not every interaction event.
