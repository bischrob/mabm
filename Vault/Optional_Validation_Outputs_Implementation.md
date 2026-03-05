# Optional Validation Outputs Implementation

Purpose:

- Add validation-focused outputs without increasing default run I/O.

Implemented outputs:

1. `settlement_trait_deposition` snapshot rows:
   - accumulated trait deposition counts by settlement/trait.
2. `network_interaction_snapshot` rows:
   - proxy edge ledger for topology comparison.

Config-gated behavior:

1. `mvp.validation_outputs.enable_trait_deposition`
2. `mvp.validation_outputs.enable_network_snapshot`
3. `mvp.validation_outputs.deposition_rate_per_tick`
4. `mvp.validation_outputs.network_min_weight`

Code changes:

1. [output.rs](C:\Users\rjbischo\Documents\mabm\src\output.rs)
   - added row structs, collectors, and CSV writers for optional outputs.
2. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
   - validation-output config and run-result fields.
   - deposition accumulation in run loop.
3. [main.rs](C:\Users\rjbischo\Documents\mabm\src\main.rs)
   - conditional writing of optional CSV files.
4. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
   - validation parameter checks.
5. [model.rs](C:\Users\rjbischo\Documents\mabm\src\model.rs)
   - per-settlement deposition accumulator state.
6. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)
   - default validation-output section.

Validation:

1. Unit test confirms optional rows can be emitted.
2. Full test suite passes.
3. Runtime check with validation-enabled config emits all three CSVs:
   - trait frequency
   - trait deposition
   - network snapshot
