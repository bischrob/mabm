# Labor Time Constraint Implementation

Purpose:

- Prevent super-agent behavior by enforcing zero-sum seasonal labor budgets.
- Implement labor/time competition among farming, foraging, fuel/water hauling, construction, and trade.

## Evaluation of Proposed Approach

Strong:

1. Labor/time must be explicitly constrained or carrying capacity is inflated.
2. Seasonal bottlenecks are essential in Southwest-style farming/foraging systems.
3. Children and elders should contribute fractional labor, not be modeled as pure dependents.
4. Opportunity-cost allocation is the right mechanism for task switching under scarcity.

Key refinement for this ABM:

- Because the simulation already uses seasonal ticks, implement allocation per season (`Spring`, `Summer`, `Autumn`, `Winter`) rather than annual waterfall only.

## Ideal Solution Under Current Constraints

Use a seasonal labor ledger with task windows and priority tiers.

### A) Seasonal Labor Budget

Per household `h`, season `s`:

- `L[h,s] = daylight_hours[s] * sum_i alpha(age_i) * attendance_factor[h,s]`

Recommended age coefficients (configurable priors):

1. `0-5`: `alpha = 0.0`
2. `6-12`: `alpha = 0.3-0.5`
3. `13-60`: `alpha = 1.0`
4. `60+`: `alpha = 0.4-0.7`

### B) Task Windows (Seasonal Feasibility)

Each task has active seasons and minimum/target hours:

1. Farming:
   - Spring prep/plant
   - Summer weeding/maintenance
   - Autumn harvest
2. Foraging:
   - season-specific high-return windows (e.g., autumn mast years)
3. Fuel/water hauling:
   - all seasons, variable by distance and scarcity
4. Construction/major trade:
   - primarily off-peak agricultural windows (winter/early spring)

If task window is closed, assign zero productive return.

### C) Allocation Algorithm (Fast)

Per household-season:

1. Reserve Tier 1 survival hours:
   - minimum water + minimum fuel.
2. Allocate Tier 2 subsistence hours:
   - farming to required seasonal target.
3. Allocate Tier 3 resilience/maintenance:
   - storage upkeep, local foraging, repairs.
4. Allocate Tier 4 surplus:
   - craft/trade/travel.

If `L[h,s]` exhausted, lower tiers receive zero.

### D) Opportunity Cost and Yield Penalty

For each constrained task `t`:

- `Output_t = Output_base_t * Phi(h_alloc_t / h_req_t)`

Where `Phi` is a saturating penalty curve (piecewise or power):

- example: `Phi(x) = min(1, x^beta_t)` with `0 < beta_t <= 1`

For harvest-critical tasks, use steeper penalty near shortfall threshold.

### E) Coupling to Existing Subsystems

1. Water/fuel depletion increases Tier 1 hours, crowding out Tier 4 trade.
2. Missed farm labor reduces crop output, increasing migration/fission pressure.
3. Trade networks decay organically when labor surplus disappears.
4. Conflict expedition feasibility requires remaining seasonal labor surplus.

## Performance Design

1. Do not simulate hourly calendars.
2. Use per-household seasonal scalar budgets and vectorized task arrays.
3. Precompute:
   - task hour coefficients by season,
   - distance-based haul hours by settlement.
4. Run allocation in one pass per household per season.

## Minimal Calibration Outputs

1. `labor_used_tier1..tier4`
2. `unmet_task_hours` by task and season
3. `farm_hours_ratio` (`allocated/required`)
4. `trade_hours_surplus`
5. `yield_penalty_from_labor`
6. `water_fuel_time_crowdout_index`

## Sources

ABM and Southwest seasonal labor framing:

1. Kintigh et al. (2013), Village Ecodynamics ABM context:
   - https://www.jasss.org/16/4/7.html
2. Bocinsky et al. (2024), labor allocation and maize farming context:
   - https://www.cambridge.org/core/journals/journal-of-computer-applications-in-archaeology/article/how-farmers-dedicated-labor-to-mesatop-farming-in-new-mexico-around-ad-1300/AFEA95A691F1FC3288C8A0E9A4AF5B0E
3. Bocinsky & Kohler (2014), climate-driven productivity/constraints:
   - https://www.pnas.org/doi/10.1073/pnas.1404367111

Age-specific labor contribution context:

4. Kramer, K. L. (2005), *Maya Children: Helpers at the Farm*.
   - https://www.radcliffe.harvard.edu/people/karen-kramer

Timing-sensitive agronomic penalty context:

5. Evans et al. (2012), early weed control timing and maize yield impacts:
   - https://bioone.org/journals/Weed-Science/volume-60/issue-3/WS-D-11-00183.1/Why-Early-Season-Weed-Control-Is-Important-in-Maize/10.1614/WS-D-11-00183.1.full

Modeling note:

- Treat all age coefficients and task-hour requirements as sweepable priors.
- Validate that trade/craft retracts first under stress before total subsistence collapse.
