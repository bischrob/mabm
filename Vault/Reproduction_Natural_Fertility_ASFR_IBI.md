# Reproduction Natural Fertility ASFR IBI

Purpose:

- Evaluate proposed natural-fertility modeling approach.
- Define a seasonal ABM implementation aligned with current project constraints.

## Evaluation of Proposed Response

What is strong:

1. Using natural fertility as baseline is correct for premodern contexts without modern contraception.
2. Pairing age-specific fertility schedules (ASFR) with interbirth interval (IBI) constraints is the right core structure.
3. Highlighting lactational amenorrhea and postpartum recovery as spacing mechanisms is methodologically sound.

What needs adjustment:

1. Yearly boolean eligibility is too coarse for this project because the model uses seasonal ticks.
2. Birth spacing should be represented as state transitions (pregnant -> postpartum amenorrheic -> fecund) rather than only a cooldown timer.
3. Agriculture generally shortens IBI in many cases, but this should be a parameterized scenario assumption, not a universal rule.
4. Add fetal loss and partnership/exposure terms to avoid unrealistically high completed fertility.

## Recommended Seasonal Implementation

State variables per reproduction-capable agent:

1. `age_years`
2. `repro_state` in `{fecund, pregnant, postpartum_amenorrheic, infecund}`
3. `gestation_ticks_remaining` (for 4 ticks/year, full term about 3 ticks)
4. `ppa_ticks_remaining` (postpartum amenorrhea duration in ticks)
5. `parity`
6. optional `partnered` / `exposure_rate`

Seasonal loop:

1. If `repro_state = fecund`, compute conception probability:
   - `p_conception_tick = f_asfr(age) * m_energy * m_stress * m_exposure`
2. Draw conception event.
3. On conception: set `pregnant`, initialize gestation countdown.
4. During pregnancy: allow miscarriage/stillbirth hazard by trimester-equivalent tick risk.
5. On live birth: increment parity; set postpartum amenorrhea countdown.
6. When amenorrhea ends: return to `fecund` unless age-related infecundity reached.

## Parameterization Guidance

1. ASFR source profile:
   - Built-in default schedule family.
   - External import (same philosophy as GIS and life tables).
2. IBI decomposition:
   - gestation + postpartum amenorrhea + waiting-to-conception.
3. Subsistence regime effect:
   - Foraging-like regime: longer `ppa_ticks`.
   - Agro-pastoral/sedentary regime: shorter `ppa_ticks`.
4. Energetics effect:
   - Link Pandolf-based travel/work energy deficit to reduced conception probability and/or elevated fetal loss.

## Suggested MVP Values (Configurable Priors)

1. Tick scale: 4/year.
2. Gestation: 3 ticks.
3. Postpartum amenorrhea:
   - Foraging baseline: 10-16 ticks.
   - More sedentary/agricultural baseline: 6-12 ticks.
4. Conception chance:
   - Derived from ASFR annual rates converted to seasonal hazard.

## Data/Schema Requirement

Demographic schedules must support custom import:

1. `mortality_table` (life expectancy note).
2. `fertility_schedule` (ASFR).
3. `ibi_profile` (state-duration priors by regime).

Store citation and version metadata in run configs for reproducibility.

## Sources to Track

1. Henry, L. (1961). "Some Data on Natural Fertility." *Eugenics Quarterly* 8(2).
2. Coale, A. J., & Trussell, T. J. (1974). "Model fertility schedules..." *Population Index*.
3. Howell, N. (1979). *Demography of the Dobe !Kung*.
4. Wood, J. W. (1994). *Dynamics of Human Reproduction*.
5. Bentley, G. R., Jasienska, G., & Goldberg, T. (1993). "Is the fertility of agriculturalists higher than that of nonagriculturalists?" *Current Anthropology* 34(5).
