# Water System Dynamics Implementation

Purpose:

- Evaluate and implement a water subsystem for semi-arid Southwest-style ABM at seasonal resolution.

## Evaluation of Proposed Approach

Strong:

1. Water should be a hard spatial constraint, not a minor modifier.
2. Separating source types (ephemeral stream vs perennial spring) is essential.
3. Sub-hex access cost is required even with 18 km flat-to-flat hexes.
4. Water quality/seasonality should feed into disease risk and settlement suitability.

Refinements:

1. Replace a single annual source yield with seasonal source reliability and storage carryover.
2. Couple water access to labor/energy budget, not only binary accessibility.
3. Keep quality effects bounded and type-specific (waterborne diseases only), not global SEIR inflation.

## Ideal Solution Under Current Constraints

Use a two-scale model:

1. Hex-scale hydrology state.
2. Settlement-scale access and hauling costs.

### A) Hex Hydrology State

Per hex `h`, source type `s`, tick `t`:

1. Climate forcing:
   - `P_t` from annual/seasonal drought index (e.g., tree-ring PDSI reconstructions).
2. Source yield:
   - `W[h,s,t] = B[h,s] * g_s(P_t, P_{t-1}, ..., P_{t-k})`
3. Lag structure:
   - Ephemeral sources: short memory (k = 0..1)
   - Stream/baseflow sources: medium memory (k = 1..4)
   - Perennial springs (groundwater-fed): longer memory (k = 4..20)
4. Practical parameterization:
   - `g_s = exp(sum_i a_i * P_{t-i})` or weighted linear form with clamped bounds.

### B) Settlement Access and Hauling

Each settlement gets a sub-hex location and nearest source set.

1. Daily/seasonal hauling burden:
   - `Cost_water = C_unloaded(d, rough) + C_loaded(d, rough, load_kg)`
   - use your existing travel/energy function (Pandolf or travel-time proxy).
2. Demand:
   - `Demand_water = persons * liters_per_person_per_day * days_in_tick`
3. Effective availability:
   - `Water_net = Water_collected - access_losses`
4. Suitability threshold:
   - settlement viability declines if water labor share exceeds cap or `Water_net < Demand_water`.

### C) Quality and Seasonality Coupling

Per source, track quality state `Q in [0,1]`:

1. `Q` degrades with stagnation duration and heat.
2. `Q` improves with flushing flow events.
3. Disease coupling:
   - apply multiplier only to waterborne transmission term:
   - `beta_water = beta_water_base * (1 + q_mult * (1 - Q))`

Avoid applying this multiplier to all disease pathways.

## Decision Rules in Simulation

1. Settlement location scoring must include water:
   - `Score = food_term + defense_term + network_term - water_access_penalty - water_quality_penalty`
2. If repeated water deficit:
   - trigger migration/fission pull to better-water hexes.
3. Seasonal camps:
   - ephemeral-source hexes can be occupied only in high-water seasons unless exceptional storage exists.

## Performance Design

1. Update hydrology at hex level only.
2. Precompute nearest-water candidate sources per settlement.
3. Recompute sub-hex route costs only when settlement moves or seasonal friction changes.
4. Cache drought-memory convolution terms per source class.

## Calibration Outputs

1. `distance_to_primary_water_km`
2. `water_labor_share`
3. `water_deficit_days`
4. `source_reliability_index`
5. `quality_index`
6. `water_driven_relocations`

## Sources

Southwest ABM + paleohydrology context:

1. Bocinsky & Kohler (2014), tree-ring-based maize/yield uncertainty context:
   - https://www.pnas.org/doi/10.1073/pnas.1404367111
2. Orth et al. (2024), integrating paleohydrology and least-cost analyses for ancient social systems:
   - https://www.cambridge.org/core/journals/american-antiquity/article/combining-paleohydrology-and-leastcost-analyses-for-inferring-possible-structure-of-ancient-social-systems-in-the-jemez-mountains-of-new-mexico-usa/333A3527A91E4B376BFB8A336B2A8A27

Hydroclimate reconstruction:

3. Cook et al. (2004), long-term aridity changes / drought atlas foundations:
   - https://www.science.org/doi/10.1126/science.1102586
4. NOAA drought-atlas overview (applied index source context):
   - https://www.drought.gov/data-maps-tools/north-american-drought-atlas

Water quantity and health risk framing:

5. WHO domestic water quantity and health service levels:
   - https://iris.who.int/handle/10665/204348
6. WHO Guidelines for Drinking-water Quality:
   - https://www.who.int/publications/i/item/9789241549950

Modeling note:

- Treat all water parameters as scenario priors with uncertainty bands.
- Validate water-driven settlement clustering before adding finer groundwater physics.
