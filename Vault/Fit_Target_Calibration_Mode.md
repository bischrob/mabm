# Fit Target Calibration Mode

Date: 2026-03-04

## Why this exists

Fixed fit targets can be unreasonable during early model development and can saturate all error components. Calibration mode derives empirically grounded target and scale suggestions from pilot sweep outputs.

## Implementation

1. Sweep summary rows now include observed metrics used in scoring:
   - `observed_population_total`
   - `observed_aggregation_count`
   - `observed_network_density`
   - `observed_mean_stress`

2. Added `sweep.fit_scoring.calibration` config block:
   - `enabled`
   - `target_quantile`
   - `low_quantile`
   - `high_quantile`
   - per-metric minimum scale floors

3. Added recommendation builder:
   - Computes quantile-based suggested targets.
   - Computes suggested scales from half-spread between low/high quantiles.
   - Applies minimum scale floors for numerical stability.

4. Added output artifact:
   - `*_fit_calibration.csv` written automatically after sweep when calibration is enabled.

## Validation

- Unit test added for recommendation builder.
- Config validation enforces quantile bounds/order and positive scale floors.

## Current pilot recommendation (synthetic-sweep)

From `outputs/synthetic-sweep_run-20260305T035444.233Z_fit_calibration.csv`:

- Suggested targets:
  - population_total: 519
  - aggregation_count: 0
  - network_density: 1.0
  - mean_stress: 0.1988
- Suggested scales:
  - population: 188.5
  - aggregation: 0.5
  - network_density: 0.05
  - stress: 0.05

These values describe the current model regime and are a better starting point for iterative calibration than aspirational targets.
