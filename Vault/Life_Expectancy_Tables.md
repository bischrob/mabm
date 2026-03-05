# Life Expectancy Tables

Purpose:

- Define mortality schedules for long-horizon premodern population dynamics.
- Support both default model tables and externally supplied custom tables.

## Research Basis

1. Coale, A. J., & Demeny, P. (1983). *Regional Model Life Tables and Stable Populations*. Academic Press.
   - Foundational model-life-table framework for populations without modern medicine.
   - Useful for baseline mortality schedules when direct local data is unavailable.

2. Weiss, K. M. (1973). *Demographic Models for Anthropology*. Memoirs of the Society for American Archaeology, No. 27.
   - Designed for anthropological/archaeological population modeling.
   - Includes mortality patterns relevant to hunter-gatherer and early agricultural contexts.

## ABM Implementation Notes

1. Mortality table format:
   - Age interval.
   - `q_x` (probability of death in interval).
   - Optional sex-specific variants.
2. Optional derived columns:
   - `l_x`, `d_x`, `L_x`, `T_x`, `e_x`.
3. Tick conversion:
   - Convert annual `q_x` to seasonal hazard for 4 ticks/year.
4. Validation:
   - Ensure monotonic survivorship behavior.
   - Check that implied life expectancy matches selected table family/range.

## Configurability Requirement

Life expectancy tables must be customizable, similar to GIS inputs:

1. Built-in defaults:
   - Ship one or more baseline tables (e.g., Coale-Demeny style profiles).
2. External import path:
   - Load user-supplied tables from file (CSV/JSON/Parquet).
3. Scenario-level override:
   - A simulation config can select built-in or external table.
4. Provenance metadata:
   - Store source name, citation, version, and transformation notes in run metadata.
5. Backward compatibility:
   - Version table schema so older saved scenarios remain reproducible.

## Minimum MVP Decision

- Start with one baseline premodern mortality table.
- Implement external table loading in MVP so demographic assumptions are not hard-coded.
