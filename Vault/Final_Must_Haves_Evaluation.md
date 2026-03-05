# Final Must Haves Evaluation

Purpose:

- Evaluate the three final recommendations and define the best integrated solution for current ABM constraints.

## Direct Answers

1. Recommended solution:
   - Adopt all three as core subsystems, with one priority adjustment:
   - `Storage/Spoilage` and `Threat-Defensibility` remain mandatory in MVP.
   - `Cultural Transmission` becomes MVP-lite (small trait set, low-cost update), then expanded in Phase 3.

2. Assume full knowledge:
   - With full knowledge, this is still the right architecture because constraints are computational and structural, not only informational.

3. Good way to do this:
   - Implement at settlement/household aggregate scale with bounded state variables and seasonal updates.
   - Avoid per-artifact or per-interaction microsimulation in MVP.

## Evaluation Against Existing Coverage

1. Multi-year storage/spoilage:
   - Already covered in `Storage_Spoilage_Research` and environmental subsystem.
   - Remaining need: calibrate storage-capacity and spoilage priors by storage technology.
2. Conflict/threat/defensibility:
   - Mostly covered in conflict + settlement push/pull notes.
   - Remaining need: explicit climate-to-threat mapping calibration and defensibility weight tuning.
3. Cultural transmission and trait evolution:
   - Partially implied but not yet specified as a standalone subsystem.
   - This is the largest current gap.

## Best Integrated Implementation

### A) Storage and Spoilage (Keep As Core)

Use:

- `S_{t+1} = (S_t * (1 - rho_t)) + Y_t - C_t`

with:

1. seasonal `rho_t` (moisture/temperature/pest effects),
2. storage capacity constraints tied to labor/infrastructure,
3. explicit interaction with migration thresholds (buffer delays abandonment/fission).

### B) Cultural Transmission (Add New Subsystem)

Use a lightweight household trait model:

1. Represent material-culture repertoire as `u64` bitset.
2. Transmission modes:
   - neutral drift (mutation),
   - conformist bias (copy common local traits),
   - prestige bias (copy high-success neighbors).
3. Update cadence:
   - seasonal update only for connected households/settlements.
4. Keep trait count small in MVP (e.g., 16-32 active bits).

This provides dynamic material culture without heavy compute.

### C) Threat and Defensibility (Tighten Existing Rules)

Use perceived suitability:

- `S_perceived = S_resource * (1 + T_regional * D_hex)`

Where:

1. `T_regional` is a smoothed regional stress index from climate + shortages + conflict memory.
2. `D_hex` is precomputed defensibility from terrain relief/access constraints.
3. Settlement choice/fission destination logic uses `S_perceived`, not only caloric return.

Caloric trap coupling (required):

1. High `D_hex` must increase daily provisioning burden:
   - `C_total = C_base * (1 + k_def * D_hex)`
2. Apply this multiplier to:
   - water hauling cost,
   - fuel hauling cost,
   - field-access travel cost.
3. Result:
   - defensible aggregation appears only under elevated threat and relaxes when threat subsides.

## MVP Prioritization

1. Finalize storage-capacity + spoilage priors.
2. Add cultural transmission MVP-lite (bitset + two transmission rules).
3. Add explicit threat index and defensibility weighting in settlement suitability.

## Minimal New Outputs to Add

1. `settlement_trait_frequency` table:
   - `run_id`, `tick`, `year`, `settlement_id`, `trait_id`, `trait_count`, `trait_frequency`
