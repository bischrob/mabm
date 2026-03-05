# Population Growth Controls and Empirical Priors

## Where hex count is controlled

Current MVP does not yet maintain a separate empty-hex lattice object. In practice:

1. `mvp.settlement_count` controls both:
   - number of starting settlements
   - number of active hex ids at initialization

Files:

1. `configs/mvp.toml`
2. `configs/sweep.toml`
3. `configs/sweep_long_transition.toml`
4. `src/mvp.rs` (default config values)

## New hard controls added

`mvp.spatial` now contains:

1. `hex_diameter_km`
2. `flat_travel_km_per_day`
3. `population_capacity_per_hex`
4. `min_population_capacity_per_hex`
5. `stores_capacity_fraction`

These feed into:

1. labor burden scaling (travel-time burden),
2. migration destination penalties by distance/roughness proxy,
3. trade edge and transfer penalties by distance/roughness,
4. demography crowding pressure (birth suppression + death add-on when above capacity).

## Resource-derived carrying-capacity formula (implemented)

Fixed `260 people/hex` is no longer used as the primary capacity estimate.

Each tick, for each settlement:

1. Seasonal per-person requirement:
   - `need_person = 2500 * 90 * burden_multiplier`
2. Effective available calories for carrying capacity:
   - `effective_kcal = yield_kcal + stores_capacity_fraction * stores_kcal`
3. Raw resource capacity:
   - `K_raw = effective_kcal / need_person`
4. Bounded operational capacity:
   - `K = clamp(K_raw, min_population_capacity_per_hex, population_capacity_per_hex)`

Crowding pressure in demography then uses:

1. `crowding = max(0, population / K - 1)`
2. crowding contributes to birth suppression and mortality add-on.

## Empirical-prior guidance (early farming / non-industrial baseline)

For long-horizon realism, prefer conservative annual rates:

1. `annual_birth_rate_override`: `0.040` to `0.055`
2. `annual_death_rate_override`: `0.035` to `0.050`
3. Long-run net growth target: around `0.0%` to `0.3%` per year for baseline stress regimes
   - with episodic declines during shock years

Storage/resource priors:

1. `sigma_seed`: `0.08` to `0.14`
2. `spoilage_rate`: `0.03` to `0.08`
3. Keep `yield_multiplier` and `stores_multiplier` near `1.0` unless justified by explicit archaeological calibration.

Spatial capacity prior:

1. `population_capacity_per_hex` should be calibrated to your unit interpretation:
   - small residential-only hex interpretation: lower values,
   - catchment-inclusive hex interpretation: higher values.
2. For current synthetic 1 km hex regime, treat this as a calibration parameter and sweep it.

## Recommended calibration workflow

1. Hold climate and storage near default literature-informed priors.
2. Sweep only:
   - `annual_birth_rate_override`
   - `annual_death_rate_override`
   - `population_capacity_per_hex`
3. Select regimes with:
   - no explosive growth,
   - no irreversible collapse,
   - slow positive or near-stationary baseline trend.
