# Sweep Runner Implementation

Purpose:

- Add reproducible parameter-sweep execution and summary outputs for robustness analysis.

Implemented:

1. Config-driven sweep module:
   - [sweep.rs](C:\Users\rjbischo\Documents\mabm\src\sweep.rs)
2. Seed policies:
   - `fixed`
   - `incremental`
   - `list`
3. Parameter ranges:
   - `sigma_seed_values`
   - `defensibility_cost_values`
   - `prestige_rate_values`
4. Summary metrics per sweep run:
   - final population total
   - mean stress composite
   - settlement count
   - output row counts
5. CSV output writer for sweep summaries.

Runtime integration:

1. [main.rs](C:\Users\rjbischo\Documents\mabm\src\main.rs)
   - if `sweep.enabled=true`, runs sweep and writes summary CSV.
2. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
   - validation for sweep config completeness.
3. [lib.rs](C:\Users\rjbischo\Documents\mabm\src\lib.rs)
   - sweep APIs exported; tests added.

Configs:

1. default sweep section in [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml) (disabled)
2. runnable example in [sweep.toml](C:\Users\rjbischo\Documents\mabm\configs\sweep.toml) (enabled)

Validation:

1. Unit test verifies sweep row production.
2. Full test suite passes.
3. Runtime test with `configs/sweep.toml` generates sweep summary CSV.
