# Baseline Metric Pack Implementation

Purpose:

- Produce standard validation metrics at snapshot cadence for every run.

Implemented metric outputs:

1. Population trend:
   - total population
   - occupied settlement count
   - mean population per occupied settlement
2. Aggregation/abandonment:
   - aggregation count above threshold
   - abandonment events since previous snapshot
3. Network structure:
   - edge count
   - network density
   - mean edge weight

Code:

1. [metrics.rs](C:\Users\rjbischo\Documents\mabm\src\metrics.rs)
   - `BaselineMetricRow`
   - `MetricTracker`
   - CSV writer
2. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
   - metrics config and row collection in run loop
3. [main.rs](C:\Users\rjbischo\Documents\mabm\src\main.rs)
   - baseline metrics CSV emission
4. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
   - metrics parameter validation
5. [lib.rs](C:\Users\rjbischo\Documents\mabm\src\lib.rs)
   - exports + baseline metrics tests

Config (`mvp.metrics`):

1. `enable_baseline_metrics`
2. `aggregation_threshold`
3. `network_min_weight`

Validation:

1. Unit tests confirm baseline metric rows are emitted and network density is bounded.
2. Full test suite passes.
3. Runtime emits `*_baseline_metrics.csv` when enabled.
