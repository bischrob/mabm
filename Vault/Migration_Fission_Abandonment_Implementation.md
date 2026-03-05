# Migration Fission Abandonment Implementation

Date: 2026-03-04

## Why this exists in the ABM

Settlement geography should respond to stress gradients, not only demographic growth/decline. This subsystem creates explicit push-pull relocation pressure so aggregation, fission, and abandonment can emerge from stress dynamics.

## Implemented logic

1. Stress computation pass retained:
   - Each settlement computes food/water/fuel/disease/conflict stress and composite stress.

2. Deterministic migration outflow rules:
   - Moderate-high stress (`stress > 0.55`) produces proportional outflow.
   - Large stressed settlements (`population > 260`, `stress > 0.45`) emit extra fission outflow.
   - Catastrophic stress in very small settlements (`stress > 0.90`, `population < 40`) triggers full abandonment outflow.

3. Destination selection (pull factors):
   - Candidate settlements must have lower stress and remaining headroom.
   - Selection score combines lower stress, better water reliability, and lower burden.
   - Best-scoring candidate receives migrants.

4. State updates:
   - Population deltas are applied after a read-only snapshot (avoids in-loop mutation hazards).
   - Household counts and labor budgets are recalculated after migration.
   - Supports true zero-population settlements.

## Demography integration adjustments

Demography now handles unoccupied settlements explicitly:
- If `population == 0`, households/labor/disease/trait counts remain zero.
- Avoids forced repopulation from previous `max(1)` behavior.

## Tests added

- `migration_reallocates_population_toward_lower_stress_settlement` verifies push-pull migration behavior.
- Existing tests continue to pass with migration/fission active.

## Scope note

This is MVP-level deterministic migration pressure (no explicit kin-edge routing or route-cost pathfinding yet). It establishes non-placeholder migration/fission mechanics needed for acceptance criteria and future extension.
