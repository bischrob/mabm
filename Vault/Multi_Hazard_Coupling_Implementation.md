# Multi Hazard Coupling Implementation

Goal:

- Model compound effects of drought, disease, conflict, water stress, and fuel stress without exploding complexity.

## Recommended Approach

Use a hybrid coupling strategy:

1. Shared latent stress state at settlement level.
2. Hazard-specific pathways that act on different subsystems.
3. Deterministic seasonal update order.

This captures nonlinear cascades while remaining fast and interpretable.

## Core State

Per settlement `s`, season `t`:

1. `stress_food`
2. `stress_water`
3. `stress_fuel`
4. `stress_disease`
5. `stress_conflict`
6. `StressComposite_s,t`

Composite:

- `StressComposite = sum(w_i * stress_i) + sum(w_ij * stress_i * stress_j)`

Include pairwise interaction terms only for known strong couplings:

1. `drought x disease`
2. `drought x conflict`
3. `water x disease`
4. `fuel x labor`

## Seasonal Update Order (Deterministic)

1. Climate forcing update.
2. Water and fuel availability update.
3. Labor allocation update (water/fuel first).
4. Food production/storage update.
5. Disease update (with water-quality modifier).
6. Conflict hazard and outcomes.
7. Migration/fission/abandonment decisions.
8. Trade/network updates.
9. Demography (birth/death/migration accounting).

This order avoids circular write conflicts and keeps causality explicit.

## Coupling Rules (Minimal but Effective)

1. Drought -> lower yields + lower water reliability.
2. Water stress -> higher disease transmission (waterborne term only).
3. Fuel/water hauling burden -> less labor for farming/trade.
4. Food shortfall + conflict memory -> higher conflict onset hazard.
5. Conflict losses -> lower labor pool + weaker defense next season.
6. Disease mortality -> lower labor pool + potential settlement collapse.

## Stability Controls

1. Clamp all stress components to `[0,1]`.
2. Smooth shocks with short moving average where needed.
3. Use hysteresis for abandonment triggers (require persistence across N seasons).
4. Separate transient spike effects from chronic stress effects.

## Calibration Strategy

1. Calibrate each hazard in isolation first.
2. Add pairwise couplings next.
3. Add full multi-hazard mode last.
4. Compare:
   - single-hazard vs coupled outcomes,
   - event timing,
   - collapse frequency and recovery time.

## Output Metrics for Coupling Diagnostics

1. `stress_component_*` by settlement and season.
2. `stress_composite`.
3. `compound_event_flag` (>=2 stressors above threshold).
4. `compound_event_duration`.
5. `recovery_time_seasons`.

## Performance Notes

1. Keep hazard states at settlement/hex level, not per-agent.
2. Reuse existing subsystem outputs as inputs to coupling step.
3. Avoid dynamic graph-wide recomputation unless thresholds crossed.
