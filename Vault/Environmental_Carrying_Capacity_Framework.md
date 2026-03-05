# Environmental Carrying Capacity Framework

Purpose:

- Evaluate and operationalize carrying-capacity modeling for mixed foraging + agriculture in a seasonal hex ABM.

## Evaluation of Proposed Approach

Strong decisions:

1. Do not use one static carrying-capacity scalar.
2. Separate resource streams into crops, wild plants, and animals.
3. Use dynamic annual/seasonal forcing (climate + depletion).
4. Couple resource shortfalls to migration, aggregation, and conflict outcomes.

Required refinements:

1. Keep units explicit and consistent:
   - biomass -> edible biomass -> kcals -> household support.
2. Include storage/decay buffers to avoid unrealistically immediate famine.
3. Add extraction limits per season (labor/time/technology constrained), not just stock availability.
4. Treat uncertainty in paleoclimate-to-yield mapping as scenario ranges.

## Recommended Seasonal State Equations

Per hex `h`, tick `t`:

1. Crop calories:
   - `CropKcal[h,t] = AreaCrop[h] * Yield[h,t] * kcal_per_kg_crop`
   - `Yield[h,t] = B[h] * f_climate[t] * f_soil[h] * f_water[h,t]`

2. Wild plant calories:
   - `WildKcal[h,t] = NPP[h,t] * edible_frac[h] * area[h] * kcal_per_biomass`
   - apply harvest-effort cap:
   - `WildHarvest[h,t] <= min(WildKcal[h,t], labor_hours[h,t] * return_rate[h,t])`

3. Animal stock dynamics:
   - `N[h,t+1] = N[h,t] + r[h]*N[h,t]*(1 - N[h,t]/K_animal[h,t]) - H[h,t]`
   - convert harvest to calories:
   - `AnimalHarvestKcal[h,t] = H[h,t] * kcal_per_animal_unit`

4. Available food after storage:
   - `FoodAvail[h,t] = CropHarvest + WildHarvest + AnimalHarvest + Store[h,t] - Spoilage[h,t]`

5. Store update:
   - `Store[h,t+1] = max(0, FoodAvail[h,t] - Consumption[h,t])`

## Agent Decision Coupling

Household/settlement in hex `h`:

1. Compute seasonal caloric demand.
2. Allocate labor in order:
   - crop tasks -> wild gathering fallback -> hunting fallback.
3. If expected deficit persists beyond threshold:
   - draw down storage.
   - increase foraging radius (higher travel cost).
   - split or migrate.
4. Record deficit pressure as a predictor for conflict risk.

## Performance Design (100k+ agents)

1. Update ecology at hex level, not per-agent resource stocks.
2. Precompute static terms:
   - area, soil class, baseline yield, terrain multipliers.
3. Keep per-tick updates to climate multiplier, extraction, and stock transitions.
4. Run household allocation with vectorized/SoA-friendly loops.
5. Cache neighborhood catchments by travel-time bands.

## Calibration and Falsification

1. Calibrate independently by stream:
   - crop yield ranges, wild return rates, animal rebound rates.
2. Check stylized dynamics:
   - drought -> storage drawdown -> mobility rise -> aggregation/conflict shifts.
3. Falsification test:
   - model should fail if all streams are made unrealistically insensitive to drought.

## Source Notes to Track

1. Kohler et al. (Village Ecodynamics work) for maize yield retrodiction with climate/soil/elevation.
2. Kelly (foraging return-rate framing and diet breadth principles).
3. Winterhalder & Lu (forager-resource depletion logic and prey switching implications).

Implementation guidance:

- Treat these as parameter families with uncertainty ranges.
- Keep import path for GIS/climate/yield layers to align with existing customizable-data architecture.
