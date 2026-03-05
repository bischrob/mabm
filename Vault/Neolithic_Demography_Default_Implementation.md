# Neolithic Demography Default Implementation

Date: 2026-03-05

## What was implemented

1. Added default life-table-driven demography using:
   - `input/neolithicdemographytable.csv`
2. Added `mvp.demography` config block with:
   - `use_life_table_default`
   - `life_table_csv_path`
   - optional `annual_birth_rate_override`
   - optional `annual_death_rate_override`
3. Demography engine now reads annual birth/death baselines from simulation state policy instead of fixed literals.

## FB(X) handling

- `FB(X)` is interpreted as annual ASFR for female offspring.
- In a sex-parity model, total fertility is approximately `2 * FB(X)`.
- For crude per-capita birth-rate derivation, this becomes weighted `FB(X)` by age-share (`C(X)` normalized).

## Derived default rates from current table

- Annual birth rate: ~`0.0719`
- Annual death rate: ~`0.0546`

These are used as default baseline rates unless explicit overrides are provided.
