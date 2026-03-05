# Fuelwood Dynamics Implementation

Purpose:

- Evaluate and implement firewood/fuel dynamics for long-horizon, semi-arid settlement ABM.

## Evaluation of Proposed Claims

What is strong:

1. Fuel is a hard constraint and should be modeled explicitly.
2. Hauling bulk wood over distance creates a strong nonlinear labor/energy penalty.
3. Fuel scarcity can drive relocation/fission even when food is acceptable.
4. Pottery production should be fuel-constrained.

Important correction:

- For Chaco and similar cases, fuel depletion is a major hypothesis, but abandonment is multi-causal (hydrology, social-political change, risk, mobility, and resource stress interact). Model fuel as a major driver with uncertainty, not a single deterministic cause.

## Ideal Solution Under Current Constraints

Use a single hex-level fuel stock + settlement-level demand and hauling cost.

### A) Fuel Stock (Hex Level)

Per hex `h`, tick `t`, maintain:

1. `fuel_stock[h,t]` (aggregate thermal-equivalent biomass)

Regeneration:

- `Fuel_{t+1} = Fuel_t + r * Fuel_t * (1 - Fuel_t/K) - H_t`

with `r` and `K` calibrated to local ecology and extraction pressure.

### B) Demand (Settlement/Household Level)

Per household `i`:

- `D_fuel,i = D_heat(T_t) + D_cook(diet_i) + D_process(prod_i)`

Where:

1. `D_heat` rises in colder seasons.
2. `D_cook` rises with boiling-intensive diet share.
3. `D_process` rises with craft intensity (e.g., ceramics output).

### C) Scarcity Logic (Single Pool)

1. Attempt demand from `fuel_stock`.
2. If demand is unmet, apply scarcity penalty and hauling-distance increase.
3. Feed unmet demand and hauling burden directly into stress/migration hazard.

### D) Hauling Cost Coupling

For settlement `s`, effective gather distance `R_gather,s` rises as nearby fuel stock drops.

- `Cost_haul,s = f(R_gather,s, terrain, carried_mass)`

Use existing travel/load energetics (Pandolf/travel-cost module) and add to household expenditure.

### E) Pottery/Trade Coupling

If `fuel_stock` remains below craft threshold for multiple ticks:

1. reduce ceramic production success probability.
2. cap ceramic export volume.
3. reduce/reweight trade edge reinforcement from ceramics.

## Decision Triggers

Add fuel push term into migration/fission model:

- `push_fuel = normalized(Cost_haul + unmet_fuel_demand + substitution_penalty)`

If persistent `push_fuel` exceeds threshold over `n` ticks, increase relocation/fission hazard.

## Performance-Safe Implementation

1. Keep biomass pools at hex level only.
2. Precompute gather-cost bands by distance/terrain class.
3. Update regeneration once per seasonal tick.
4. Avoid per-tree or per-trip simulation.
5. Log only aggregate fuel metrics per settlement per tick.

## Minimal Outputs for Calibration

1. `fuel_demand_total`
2. `fuel_met_fraction`
3. `fuel_stock_level`
4. `mean_gather_distance_km`
5. `fuel_haul_energy_cost`
6. `fuel_driven_migration_events`
7. `ceramic_output_fuel_limited_events`

## Sources

Chaco/Southwest evidence and debate context:

1. Crabtree et al. (2021), ecosystem impacts and fuelwood demand discussion:
   - https://pmc.ncbi.nlm.nih.gov/articles/PMC8550600/
2. Samuels & Betancourt (1982), long-term fuelwood harvest modeling in pinyon-juniper systems (cited in USGS/NPS technical materials):
   - https://pubs.usgs.gov/of/2011/1109/of2011-1109.pdf

Fuel taxa evidence in Pueblo contexts:

3. Crow Canyon archaeobotanical report (juniper, sagebrush, pinyon and other taxa in thermal features):
   - https://crowcanyon.org/ResearchReports/SandCanyon/Text/scpw_archaeobotanicalremains.php

General energy/load coupling support:

4. Pandolf et al. (1977) load carriage energy model:
   - https://journals.physiology.org/doi/abs/10.1152/jappl.1977.43.4.577

Note:

- Use scenario sweeps to test whether fuel stress is necessary/sufficient in your synthetic and GIS-informed runs.
