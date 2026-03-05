# Ideal Despotic Distribution Implementation

Purpose:

- Evaluate whether IDD should be added to the current ABM.
- Specify a performant implementation under seasonal, 100k+ agent constraints.

## Evaluation

Conclusion:

- IDD is a strong fit for your research goals because it introduces within-hex inequality in access to productive land, which should accelerate early out-migration and fission relative to Ideal Free Distribution logic.

Why it fits this project:

1. You already model push-pull migration and MVT-like yield comparison.
2. IDD adds a realistic social-institutional mechanism (tenure priority) without requiring parcel-level micro-simulation.
3. It is computationally cheap if implemented at settlement/household-rank level.

## Recommended Model for Your Constraints

Use tenure-ranked access shares per settlement, not per-cell parcel ownership.

Per settlement `s` at tick `t`:

1. `Y_s,t` = total cultivable kcal available in the settlement catchment.
2. Households have `arrival_tick` and derived tenure rank `r` (older = lower `r`).
3. Each household gets a land-access weight:
   - `w_r = exp(-alpha * r)` with `alpha > 0`.
4. Normalize weights:
   - `p_r = w_r / sum_j(w_j)`
5. Household crop allocation:
   - `alloc_r = Y_s,t * p_r`

Interpretation:

- Early arrivals capture disproportionate productive share.
- Later arrivals are effectively pushed to marginal returns even before absolute scarcity.

## Migration Coupling (Critical)

For household `h`:

1. Compute expected local return:
   - `R_local_h = alloc_h - travel_cost_local_h - labor_cost_h`
2. Compute best reachable outside option using existing pull model:
   - `R_out_h = max_d(Pull_hd_adjusted_return)`
3. Tenure-sensitive migration trigger:
   - if `R_local_h < R_out_h * (1 - tau_stickiness_h)`, set migration intent.

This directly implements your suggestion that new households should migrate earlier.

## Optional Extensions (Feature Flags)

1. Inheritance persistence:
   - tenure privilege transfers partially to descendants.
2. Political integration mitigation:
   - hierarchy/cooperative institutions reduce `alpha` over time.
3. Rotational access:
   - occasional redistribution shocks to simulate social renegotiation.

## Calibration Targets

1. Household-level variance in food access within same settlement.
2. Age-of-settlement effect on newcomer out-migration rates.
3. Frequency/timing of fission events under equal climate forcing.
4. Correlation of tenure inequality with conflict incidents.

## Performance Notes

1. Sort households by `arrival_tick` once per major demographic change, not every tick.
2. Cache normalized tenure weights by settlement size buckets.
3. Apply allocation with vectorized loops over households.

## Sources

Foundations:

1. Fretwell, S. D., & Lucas, H. L. (1970). On territorial behavior and habitat distribution in birds.
   - https://link.springer.com/article/10.1007/BF00347876
2. Fretwell, S. D. (1972). *Populations in a Seasonal Environment*.
   - https://www.jstor.org/stable/10.2307/j.ctvx5w9k1

IDD development and unequal competitors:

3. Sutherland, W. J., & Parker, G. A. (1985). Distribution of unequal competitors.
   - https://www.sciencedirect.com/science/article/abs/pii/0003347285901411

Human/archaeological application context:

4. Winterhalder, B., Kennett, D. J., Grote, M. N., & Bartruff, J. (2010). Ideal free settlement of California's Northern Channel Islands.
   - https://www.pnas.org/doi/10.1073/pnas.0909611107
5. Codding, B. F., & Jones, T. L. (2013). Environmental productivity predicts migration and settlement patterns.
   - https://www.pnas.org/doi/10.1073/pnas.1302008110

Note:

- IFD and IDD are complementary bounding models. For this project, use IDD as default and keep IFD as sensitivity baseline.
