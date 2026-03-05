# Sweep Fit Scoring Implementation

Purpose:

- Rank sweep runs by closeness to target pattern metrics during synthetic-data phase.

Implemented:

1. Configurable fit scoring block in sweep config:
   - `targets`
   - `weights`
   - `scales`
2. Normalized error components:
   - population error
   - aggregation error
   - network density error
   - stress error
3. Composite fit score:
   - weighted error aggregated then transformed to `fit_score` in `[0,1]`.

Output:

- Sweep summary rows now include:
  - `fit_score`
  - `fit_error_population`
  - `fit_error_aggregation`
  - `fit_error_network_density`
  - `fit_error_stress`

Code:

1. [sweep.rs](C:\Users\rjbischo\Documents\mabm\src\sweep.rs)
2. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs) (validation)
3. [lib.rs](C:\Users\rjbischo\Documents\mabm\src\lib.rs) (tests)
4. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml) (defaults)
5. [sweep.toml](C:\Users\rjbischo\Documents\mabm\configs\sweep.toml) (example)

Validation:

1. Unit tests pass.
2. Sweep run emits summary CSV with fit scoring columns.
