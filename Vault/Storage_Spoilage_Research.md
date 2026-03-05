# Storage Spoilage Research

Scope:

- Research-backed best practices for modeling food storage and spoilage.
- Deficit sharing intentionally excluded for now.

## Why This Matters

For long-horizon ABMs, storage is a key buffer between climate shocks and migration/conflict dynamics. Without explicit spoilage, models overstate resilience.

## Core Modeling Practice

Model storage as stock-flow with decay:

1. `Store[h,t+1] = max(0, (Store[h,t] + Inflow[h,t] - Use[h,t]) * (1 - d[h,t]))`
2. `d[h,t]` is seasonal spoilage fraction in `[0,1]`.

Seed reserve requirement (must-have for agricultural persistence):

1. Reserve planting fraction before consumption accounting:
   - `Y_usable = Y_total * (1 - sigma_seed)`
2. Storage transition with seed tax:
   - `Store[t+1] = max(0, (Store[t] * (1 - d_t)) + Y_usable - C_t)`
3. If food demand cannot be met without seed drawdown:
   - trigger desperation branch (`consume_seed` or `emergency_reciprocity`).
4. If `consume_seed` is used:
   - reduce next-season planted area/yield capacity accordingly.

Recommended decomposition:

1. `d[h,t] = 1 - exp(-lambda[h,t] * dt)`
2. `lambda[h,t] = lambda_base + lambda_moist + lambda_temp + lambda_pest`

Where:

- `lambda_moist` rises when grain moisture exceeds safe thresholds.
- `lambda_temp` rises at higher storage temperatures.
- `lambda_pest` rises with infestation pressure and weak storage architecture.

## Practical Parameter Strategy (Seasonal Model)

1. Distinguish storage classes:
   - Sealed/protected granary.
   - Household bin/pit.
   - Exposed short-term storage.
2. Use class-specific baseline decay priors.
3. Apply climate multipliers by season (wet/hot seasons increase decay risk).
4. Apply pest shocks stochastically (rare high-loss events).
5. Cap extraction by labor/time to avoid unrealistically instant drawdown.

## Archaeological and Historical Anchors

1. Early granary storage as risk buffering:
   - Kuijt, I., & Finlayson, B. (2009), PNAS.
   - Evidence that intentional storage facilities were central in early surplus/risk management.
2. Southwest case with storage loss/failure under moisture:
   - Fish et al. (2017), documented Late Archaic pit-storage failure episodes linked to moisture at Las Capas.
3. Risk-buffer framing in agrarian archaeology:
   - Halstead & O'Shea (1989), *Bad Year Economics*.

## Postharvest Spoilage Process Anchors

1. FAO grain storage guidance:
   - Moisture and temperature are first-order controls of fungi/insects and storage losses.
2. Grain-loss synthesis:
   - Regional/contextual losses can be substantial; treat loss rates as uncertain and scenario-specific, not universal constants.

## ABM Best Practices

1. Keep storage and spoilage at hex or settlement level for performance.
2. Store separate food pools (grain vs wild plant dry stores vs animal dry stores) with different decay rates.
3. Encode spoilage as both:
   - background continuous loss, and
   - occasional discrete shock events (mold, pests, flood damage).
4. Track calibration metrics:
   - mean months of food in store,
   - frequency of total storage failure,
   - recovery time after drought years.
5. Validate against stylized behavior:
   - moderate drought buffered by storage,
   - multi-year drought causing depletion cascades.

## Sources

- Kuijt, I., & Finlayson, B. (2009). PNAS (early Neolithic granaries):
  - https://www.pnas.org/doi/10.1073/pnas.0812764106
- Fish et al. (2017). Late Archaic pit storage failure (Las Capas):
  - https://www.cambridge.org/core/journals/american-antiquity/article/abs/storage-and-subsistence-a-case-study-from-the-late-archaic-sonoran-desert/EA40A4BC3E1B8A6EF6DB4E2677A1DC8B
- Halstead, P., & O'Shea, J. (1989). *Bad Year Economics*:
  - https://www.cambridge.org/core/books/bad-year-economics/4E0184E0A454D740ACA7A523ACAABC53
- FAO (grain storage fundamentals/engineering guidance):
  - https://www.fao.org/4/x5065e/x5065e00.htm
  - https://www.fao.org/4/y4011e/y4011e00.htm
- Postharvest loss review context:
  - https://www.nature.com/articles/s41893-021-00833-1
