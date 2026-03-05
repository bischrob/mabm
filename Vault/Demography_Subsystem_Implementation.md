# Demography Subsystem Implementation

Date: 2026-03-04

## Why this exists in the ABM

Demography is required to close the causal loop between environmental/social stress and long-run regional outcomes. Without births and deaths, stress changes do not propagate into population trajectories, settlement scaling, or downstream interaction structure.

## Implemented behavior

1. Seasonal births and deaths:
   - Birth and death rates are applied per seasonal tick from annual baselines.
   - Seasonal modulation is included (slightly higher births in spring, higher deaths in winter).

2. Stress-coupled vital rates:
   - Births are suppressed by composite stress, disease burden, and defensibility burden.
   - Deaths increase with stress, disease burden, defensibility burden, and emergency-reciprocity flags.

3. Population-linked state maintenance:
   - Households are recalculated from population (`population / 5`, bounded >= 1).
   - Labor budgets are refreshed from household counts each tick.
   - Trait counts are clamped to household bounds.

4. Disease compartment consistency:
   - After demographic change, S/E/I/R compartments are proportionally rebalanced so they exactly sum to current population.

## Validation added

1. Low-stress growth test:
   - Under abundant food/water/fuel and low stress, population increases.

2. High-stress decline test:
   - Under severe stress and infection pressure, population declines.
   - Disease compartments remain population-consistent after update.

## Notes

This is still MVP-level demography (aggregate rates, no explicit age-sex cohorts yet). It provides a stable and computationally cheap demographic feedback path while preserving future extension points for age-structured fertility/mortality tables.
