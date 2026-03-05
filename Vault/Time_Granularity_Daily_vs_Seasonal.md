# Time Granularity Daily vs Seasonal

## Daily Ticks

Pros:

- Captures short shocks (storms, raids, acute food stress).
- Better movement/logistics realism.
- Better for event-driven conflict/trade timing.

Cons:

- Very high compute and storage cost over centuries.
- More parameters needed (daily behavior rules).
- Higher risk of overfitting noisy short-term dynamics.

## Seasonal Ticks

Pros:

- Much faster for long-horizon runs.
- Aligns with planting/harvest/mobility cycles in many preindustrial settings.
- Easier calibration with sparse historical/archaeological data.

Cons:

- Misses short-duration events unless approximated.
- Can smooth out thresholds that trigger conflict or migration.
- Requires careful aggregation rules for within-season processes.

## Runtime Back-of-the-Envelope (100,000 agents, 1,000 years)

- Daily ticks: about 365,000 ticks -> 36.5 billion agent-updates.
- Seasonal ticks (4/year): 4,000 ticks -> 400 million agent-updates.

Runtime depends on effective agent-updates/second:

- 0.5 million updates/s:
  - Daily: about 20.3 hours
  - Seasonal: about 13.3 minutes
- 5 million updates/s:
  - Daily: about 2.0 hours
  - Seasonal: about 1.3 minutes
- 50 million updates/s:
  - Daily: about 12.2 minutes
  - Seasonal: about 8 seconds

These are core-loop estimates only and exclude heavy I/O, checkpointing, GUI rendering, and expensive interaction queries.
