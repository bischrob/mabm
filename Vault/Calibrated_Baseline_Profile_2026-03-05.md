# Calibrated Baseline Profile 2026-03-05

## Why this note exists

This note locks a reproducible calibrated fit profile for the synthetic sweep regime so subsequent model changes can be evaluated against a stable baseline.

## Source runs

- Pre-calibration long sweep summary:
  - `outputs/synthetic-sweep_run-20260305T034726.501Z_sweep_summary.csv`
- Calibration recommendation:
  - `outputs/synthetic-sweep_run-20260305T035444.233Z_fit_calibration.csv`
- Post-calibration validation sweep:
  - `outputs/synthetic-sweep_run-20260305T035658.620Z_sweep_summary.csv`

## Locked fit profile (applied to `configs/sweep.toml`)

- Targets:
  - `population_total = 519.0`
  - `aggregation_count = 0.0`
  - `network_density = 1.0`
  - `mean_stress = 0.1988`
- Scales:
  - `population = 188.5`
  - `aggregation = 0.5`
  - `network_density = 0.05`
  - `stress = 0.05`

## Post-calibration performance (36-run sweep)

- Mean fit score: `0.6797`
- Best fit score: `0.9451`
- Worst fit score: `0.4886`

## Interpretation

- The fit profile now reflects the current model regime under the 400-tick sweep setup.
- This is a model-state baseline, not an empirical archaeology target baseline.
- Future mechanism changes should be compared against this checkpoint first; empirical target migration should happen after additional calibration steps.

