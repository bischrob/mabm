# Research Audit Full Coverage

Date:

- 2026-03-05

Purpose:

- Full evaluation of current research coverage across the ABM.
- Identify remaining gaps that are not yet explicit in the current checklist.

## Coverage Summary

Well covered (architecture and core mechanisms defined):

1. Temporal scale (seasonal ticks).
2. Travel friction and hex movement.
3. Demography (mortality, fertility, IBI).
4. Disease (hex-mixing SEIR with movement import).
5. Environmental carrying capacity (crop/wild/game streams).
6. Settlement fission/abandonment push-pull logic.
7. Conflict subsystem structure.
8. Trade-network dynamics.
9. Water subsystem.
10. Fuel subsystem.
11. Labor/time constraints.
12. Output schema and streaming/parallelization design.

Partially covered (mechanisms present, calibration evidence still thin):

1. Storage technology variation and spoilage priors.
2. IDD tenure inequality strength and institutional dampening.
3. Conflict coefficient priors and retaliation-memory empirical bounds.
4. Trade gravity/decay coefficients by ecology and social context.
5. Water quality decay and lag-kernel parameterization.
6. Fuel substitution efficiencies and craft-quality thresholds.
7. Age-task labor coefficients by subsistence regime.

## Newly Identified Gaps (Not Explicit Enough Yet)

1. Initialization and spin-up protocol:
   - Initial settlement distribution, household composition, and stock states.
   - Burn-in period rules before collecting analysis outputs.
2. Boundary-condition policy:
   - Edge effects for migration, trade, conflict, and resource spillover.
   - Whether map edges are hard boundaries, buffers, or permeable exogenous regions.
3. Information/perception model:
   - What agents know (local vs regional yields, water reliability, conflict risk).
   - Information delay/noise and its effect on decisions.
4. Observation model for archaeological comparison:
   - Mapping latent simulation states to observable proxies (site counts, ceramic distributions, conflict markers).
   - Taphonomic/sampling uncertainty handling.
5. Institutional adaptation over centuries:
   - Rules for how coordination capacity, norms, or leadership changes through time.
   - Interaction with scalar stress and network persistence.
6. Multi-hazard coupling:
   - Compound shocks (drought + epidemic + conflict) and nonlinear cascading effects.
7. Unit-consistency and dimensional QA:
   - Formal checks across kcal, liters, labor-hours, distance/time, and biomass conversions.

## Evaluation Addendum: Four-Point Framework Review

1. Seed tax in storage:
   - Valid and accepted.
   - Added as explicit storage requirement (`sigma_seed`) with desperation branch.
2. Systemic vs deposited culture:
   - Valid and accepted.
   - Added optional deposition output to support archaeological assemblage comparison.
3. Defensibility caloric trap:
   - Valid and accepted.
   - Added explicit coupling of high defensibility to higher water/fuel/field-access costs.
4. Edge-ledger output:
   - Valid with caveat.
   - Kept trait-frequency as minimal default; added optional network snapshot ledger for topology inference.

## Priority Ranking for Next Research Sprint

1. Observation model for archaeological comparison.
2. Initialization and spin-up protocol.
3. Information/perception limits and delay.
4. Boundary-condition policy.
5. Institutional adaptation rules.

## Acceptance Criterion for "Research Complete for MVP"

Research can be considered MVP-complete when:

1. Every subsystem has bounded priors and at least one sensitivity-tested alternative.
2. Initialization/boundary/information assumptions are explicit and versioned.
3. Output metrics are mapped to at least one archaeological proxy family.
4. Calibration and falsification tests are predefined before feature expansion.
