# Decision Evaluation Seasonal Reciprocity Hex Costs

## Decision 1: Seasonal Time Model

Assessment: Strong choice for long-horizon simulation speed and calibration stability.

Benefits:

- Major runtime reduction for 1,000-year scenarios.
- Natural fit for planting/harvest and mobility cycles.
- Better signal-to-noise when comparing broad settlement and population dynamics.

Risks:

- Short shocks can be over-smoothed.
- Need clear within-season ordering of events.

Mitigation:

- Use seasonal sub-phases (e.g., early/late season) without switching to daily ticks.

## Decision 2: Exchange as Balanced Reciprocity

Assessment: Good MVP simplification and historically plausible in many low-complexity contexts.

Benefits:

- Reduced rule complexity and fewer free parameters.
- Easier interpretation of network emergence.
- Cleaner baseline before introducing hierarchy/market mechanisms.

Risks:

- May under-represent prestige exchange, redistribution, or coercive extraction.
- Can suppress inequality patterns that affect conflict and aggregation.

Mitigation:

- Track reciprocity imbalance and unmet obligations as latent pressure variables.

## Decision 3: Hex Grid + Surface Travel Cost Heuristic

Assessment: Strong spatial design choice for isotropic neighborhood structure and path modeling.

Benefits:

- Hexes reduce directional bias compared with square grids.
- Supports local movement, settlement suitability, and trade corridor emergence.
- Clear framework for terrain-dependent friction.

Risks:

- Heuristic costs can hide important slope/river chokepoints.
- Resolution choice (hex size) strongly affects movement and settlement outcomes.

Mitigation:

- Validate that route patterns remain stable across at least two hex resolutions.

## Decision 4: Symmetrical Travel Costs

Assessment: Good first-pass assumption for computational simplicity and determinism.

Benefits:

- Faster path computation and easier caching.
- Fewer confounds in early sensitivity analysis.

Risks:

- Misses uphill/downhill asymmetry and downstream river effects.
- Can distort conflict/trade directionality in rugged terrain.

Mitigation:

- Add optional directional asymmetry later as a feature flag.

## Recommended Immediate Next Rules

1. Define per-hex movement cost formula with bounded ranges.
2. Separate static terrain cost from seasonal weather multiplier.
3. Precompute neighbor transition costs at initialization.
4. Log route-length and effective-distance metrics for calibration.
