# Deterministic Growth Calibration Sweep 2026-03-05

## Why this sweep was rerun

Earlier calibration trials were unstable across runs because settlement iteration order came from `HashMap`.  
Settlement storage is now ordered (`BTreeMap`), so repeated runs with the same config are reproducible.

## Sweep dimensions

Using `configs/sweep_long_transition.toml` as template, with `sweep.enabled = false` per trial:

1. `annual_birth_rate_override`: `0.056`, `0.060`, `0.064`, `0.068`
2. `annual_death_rate_override`: `0.024`, `0.027`, `0.030`
3. `yield_multiplier`: `1.20`, `1.40`, `1.60`
4. `shock_chance_per_year`: `0.01`, `0.02`

Output artifact:

1. `outputs/_growth_sweep_deterministic_results.json`

## Selected regime

Chosen to target slow positive growth without explosive dynamics:

1. `annual_birth_rate_override = 0.056`
2. `annual_death_rate_override = 0.030`
3. `yield_multiplier = 1.20`
4. `shock_chance_per_year = 0.02`

Observed from full run (`configs/sweep_long_transition.toml`):

1. Start population: `1000`
2. End population (500 years): `1203`
3. CAGR: `+0.03697%` per year

This is a conservative positive-growth baseline suitable for continued stress/capacity calibration.
