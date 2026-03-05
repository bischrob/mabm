# Config System Implementation

Purpose:

- Make MVP runs reproducible and scenario-driven from configuration files.

Implemented:

1. File-based config loading from TOML.
2. Validation of required fields and basic constraints.
3. Deterministic SHA-256 config hash generation.
4. Injection of config hash into run metadata and output rows.

Code:

1. [config.rs](C:\Users\rjbischo\Documents\mabm\src\config.rs)
2. [main.rs](C:\Users\rjbischo\Documents\mabm\src\main.rs)
3. [mvp.rs](C:\Users\rjbischo\Documents\mabm\src\mvp.rs)
4. [versioning.rs](C:\Users\rjbischo\Documents\mabm\src\versioning.rs)
5. [output.rs](C:\Users\rjbischo\Documents\mabm\src\output.rs)

Default config:

1. [mvp.toml](C:\Users\rjbischo\Documents\mabm\configs\mvp.toml)

Run behavior:

1. `cargo run` loads `configs/mvp.toml` by default.
2. You can pass a custom path as first CLI arg.
3. Output filename includes scenario id and run id.

Validation checks:

1. non-empty `scenario_id`
2. `ticks > 0`
3. `snapshot_every_ticks > 0`
4. `snapshot_every_ticks <= ticks`
5. `settlement_count > 0`
6. `base_population > 0`
