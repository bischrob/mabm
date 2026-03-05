# Settlement Fission Abandonment Push Pull

Purpose:

- Evaluate and operationalize multi-cause settlement fission/abandonment dynamics for the seasonal Southwest-focused ABM.

## Evaluation

Strong:

1. Push-pull framing is correct and preferable to single-trigger abandonment.
2. Scalar stress + environmental returns + disease shocks form a robust core.
3. Destination choice using kin ties + terrain-weighted opportunity is appropriate.

Refinements:

1. Do not trigger on hard single thresholds alone; use weighted hazard/score models.
2. Keep modularity/faction checks periodic (not every tick) for performance.
3. Keep hierarchy as a continuous mitigation parameter, not binary unlock only.

## Recommended Settlement-Level Trigger Model

For settlement `s` at tick `t`, define:

1. Social push:
   - `stress_scalar = HH_s * (HH_s - 1) / 2`
   - `stress_norm = stress_scalar / stress_ref`
2. Network factionalism push:
   - `faction_push = max(0, Q_s - Q0)` where `Q_s` is modularity.
3. Environmental push:
   - `env_push = max(0, (yield_regional_s - yield_local_s) / max(eps, yield_regional_s))`
4. Catastrophic disease push:
   - `epi_push = max(0, mortality_tick_s - mortality_threshold)`

Composite push score:

- `Push_s = w1*stress_norm + w2*faction_push + w3*env_push + w4*epi_push`

Pull score for candidate destination `d`:

- `Pull_sd = a1*kin_tie_sd + a2*resource_potential_d - a3*travel_cost_sd - a4*defense_risk_d`

Decisions:

1. If `Push_s < T_stay`: remain.
2. If `T_stay <= Push_s < T_split`: partial emigration (household-level moves).
3. If `Push_s >= T_split`: fission event (cohesive subgroup leaves).
4. If viable labor falls below subsistence minimum after shock: forced abandonment and absorption attempt.

## Cohesive Fission Rule

1. Build household interaction graph inside settlement.
2. Run community detection every `K` ticks (or when `Push_s` exceeds pre-threshold).
3. Select emigrating community by highest internal cohesion and positive destination pull.
4. Move entire community block to preserve social structure.

Performance fallback:

- If graph algorithms are expensive, use lightweight proxy clusters from tie-strength labels and kin groups.

## Destination Selection

Candidate set:

1. Existing settlements with positive kin/trade ties.
2. Unoccupied hexes within migration radius and acceptable travel cost.

Choose destination by max `Pull_sd` subject to:

1. minimum resource score,
2. water access constraint,
3. conflict risk ceiling.

## Minimal Outputs Needed for Calibration

1. `fission_event_count`
2. `abandonment_event_count`
3. `mean_households_per_fission`
4. `push_component_scores` (`stress`, `faction`, `env`, `epi`)
5. `destination_type` (`kin_settlement`, `new_hex`, `merge_settlement`)

## Source Notes to Track

1. Johnson (1982) scalar stress and decision bottlenecks in growing groups.
2. Charnov MVT logic for patch abandonment under declining marginal returns.
3. Network modularity/community structure methods for faction-like splits.
4. Archaeological Southwest ABM literature for calibration targets of aggregation/dispersion cycles.
