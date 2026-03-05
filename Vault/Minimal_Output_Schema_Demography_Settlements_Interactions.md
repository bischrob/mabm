# Minimal Output Schema Demography Settlements Interactions

Goal:

- Keep outputs minimal but sufficient to analyze demographics, settlement rates, and between-settlement interaction.

## 1. Run Metadata (one row per run)

Required fields:

1. `run_id`
2. `seed`
3. `scenario_id`
4. `tick_per_year` (for seasonal model, `4`)
5. `start_year`
6. `end_year`
7. `config_hash`

## 2. Demographic Time Series (one row per tick per settlement)

Required fields:

1. `run_id`
2. `tick`
3. `year`
4. `settlement_id`
5. `population_total`
6. `households`
7. `births`
8. `deaths_total`
9. `deaths_disease`
10. `in_migrants`
11. `out_migrants`

Minimum derived metrics enabled:

- growth rate, crude birth/death rates, net migration rate.

## 3. Settlement State Time Series (one row per tick per settlement)

Required fields:

1. `run_id`
2. `tick`
3. `year`
4. `settlement_id`
5. `hex_id`
6. `is_occupied` (0/1)
7. `new_settlement_event` (0/1)
8. `abandonment_event` (0/1)
9. `food_store_kcal`
10. `seasonal_food_deficit_kcal`

Minimum derived metrics enabled:

- settlement formation rate, abandonment rate, occupancy duration.

## 4. Inter-Settlement Interaction Edges (one row per active edge per tick)

Required fields:

1. `run_id`
2. `tick`
3. `year`
4. `from_settlement_id`
5. `to_settlement_id`
6. `interaction_type` (`trade`, `conflict`, `alliance`, `migration_link`)
7. `interaction_count`
8. `flow_people`
9. `flow_goods_kcal`
10. `travel_cost_days`

Minimum derived metrics enabled:

- network density, mean degree, reciprocity, conflict intensity, corridor persistence.

## 5. Optional Minimal Spatial Snapshot (one row per settlement every N ticks)

Required fields:

1. `run_id`
2. `tick`
3. `settlement_id`
4. `x`
5. `y`

Purpose:

- map-level validation and animation without logging full agent trajectories.

## 6. Optional Cultural Table: Trait Frequency by Settlement

If you only want cultural output:

1. `run_id`
2. `tick`
3. `year`
4. `settlement_id`
5. `trait_id`
6. `trait_count`
7. `trait_frequency`

Write sparse rows only (`trait_count > 0`).

## Logging Cadence and Performance

1. Write settlement and demographic tables every tick.
2. Write interaction edges every tick but only for non-zero interactions.
3. Write spatial snapshots every 4-8 ticks to reduce I/O.
4. Use columnar format (Parquet) for analysis speed and storage efficiency.

## Absolute Minimum If You Need To Cut Further

Keep only:

1. `Demographic Time Series`
2. `Settlement State Time Series`
3. `Inter-Settlement Interaction Edges`

This is enough to answer:

- How population changes over time,
- where settlements emerge/persist/fail,
- and how settlements connect via trade/conflict/migration.

For culture-only output mode, keep only:

1. `Trait Frequency by Settlement`.
