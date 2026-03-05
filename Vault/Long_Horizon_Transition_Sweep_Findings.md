# Long Horizon Transition Sweep Findings

Date: 2026-03-05

## Objective

- Run several long-horizon parameter sweeps with many more starting hexes and lower initial population.
- Diagnose the early population drop and low plateau behavior.
- Tune toward stable neolithic-transition-like growth (slow positive growth, no rapid collapse).

## Experimental setup (high scale)

1. Long horizon:
   - `ticks = 2000` (500 simulated years at seasonal resolution).
2. Many hexes:
   - `settlement_count = 90`.
3. Lower starting population:
   - `base_population = 20`.
4. New long sweep config:
   - `configs/sweep_long_transition.toml`.

## Diagnosis

1. Primary driver of the early drop:
   - Stress-coupled mortality pressure (especially threat/burden effects), not space exhaustion.
2. Evidence against space limitation:
   - Aggregation remained near zero in diagnostics despite many available hexes.
   - Increasing hex count did not eliminate decline under the original stress-demography coupling.
3. Resource/stress coupling effect:
   - Lower threat burden and slightly improved resource baseline reduced early declines.

## Implemented tuning changes

1. Demography stress response softened in engine:
   - Reduced birth suppression sensitivity to stress/disease/burden.
   - Reduced mortality add-on sensitivity to stress/disease/burden.
   - Reduced emergency reciprocity mortality penalty.
2. Added configurable synthetic resource multipliers:
   - `mvp.resources.yield_multiplier`
   - `mvp.resources.stores_multiplier`
3. Added long-run sweep growth diagnostics:
   - `observed_start_population_total`
   - `observed_cagr_percent`
   in sweep summary rows.

## Final long-sweep result

From `synthetic-transition-long` sweep:

- Mean CAGR: `+0.0007%` per year
- Range: `-0.0008%` to `+0.0016%` per year
- Interpretation:
  - Early sharp collapse behavior is resolved.
  - System is near-stable with slight positive drift in best regimes.

This is an acceptable stable baseline regime for continued neolithic transition calibration.
