# Model Impetus and System Summary

## Impetus

This model exists to examine how long-run environmental constraints shape:

1. demographic trajectories,
2. settlement formation, aggregation, relocation, and abandonment,
3. interaction networks (exchange, conflict, and cultural transmission)

across a large semi-arid region over hundreds to thousands of years.

The central research goal is to test how climate variability, resource friction, mobility costs, and social structure combine to produce historically plausible regional patterns, rather than treating any single factor as determinative.

## Core Modeling Approach

The simulation uses:

1. Seasonal time steps (4 ticks/year),
2. 1 km flat-to-flat hexes as spatial units,
   with a flat-ground travel baseline of 36 km/day (`1/36 day` per hex crossing),
3. deterministic staged updates for reproducibility,
4. high-scale design choices (settlement/hex-level aggregation where possible),
5. uncertainty-aware parameters for calibration and sweeps.

## Model Components

### 1) Environment and Carrying Capacity

Environmental productivity is modeled through separate streams:

1. crops,
2. wild plants,
3. game/animal stocks.

Each stream responds to climate and local conditions with explicit unit accounting and seasonal updates.

### 2) Water System

Hydrology is source-typed (ephemeral/stream/spring), with lagged climate response. Settlement viability depends on sub-hex access cost, hauling burden, and seasonal water quality. Water stress can force relocation and increase waterborne disease risk.

### 3) Fuel System

Fuel is modeled as a single stock variable with depletion/regeneration and hauling costs. Fuel scarcity competes for labor, constrains craft production (including ceramics), and contributes to migration pressure.

### 4) Labor-Time Constraints

Households operate under seasonal zero-sum labor budgets with age-weighted contributions. Survival tasks (water/fuel) crowd out farming, foraging, construction, and trade when stress increases, preventing unrealistic “super-agent” behavior.

### 5) Demography

Mortality is life-table based and configurable. Fertility uses natural-fertility logic with age-specific schedules and interbirth interval states. Demographic outcomes are endogenous to food, disease, migration, and conflict.

### 6) Disease

A performant hex-mixing SEIR design is used for MVP, with between-hex import via movement/trade. Water-quality effects are applied to waterborne pathways. Disease interacts with labor capacity and settlement persistence.

### 7) Settlement Dynamics

Settlement fission/abandonment follows a push-pull framework combining:

1. social stress,
2. factional structure,
3. environmental return differentials,
4. epidemic shocks,
5. destination pull (kin ties, opportunity, travel cost).

IDD-style tenure inequality is included as a within-settlement mechanism that can push late-arriving households toward earlier migration.

### 8) Conflict and Defensibility

Conflict is modeled as settlement-level hazard + dyadic targeting + stochastic outcomes. Defensibility enters settlement suitability under elevated threat, but defensive choices carry a caloric trap: higher defensibility increases provisioning and access costs.

### 9) Trade Networks

Trade uses a dynamic sparse settlement graph with gravity-style formation, tie decay/reinforcement, reciprocity debt, and route-loss-aware utility. This supports formation, persistence, and collapse of regional exchange networks under stress.

### 10) Cultural Transmission

Material culture is represented with lightweight trait state (bitset-friendly design), allowing neutral drift and biased transmission (conformist/prestige). The framework distinguishes living/systemic trait states from deposited assemblage accumulation for archaeological comparison.

### 11) Multi-Hazard Coupling

Hazards are coupled through settlement-level stress components and selected interaction terms (e.g., drought×disease, drought×conflict), with deterministic seasonal ordering to maintain interpretability and stability.

## Outputs

Practical default:

1. `settlement_trait_frequency` table (sparse rows):
   - `run_id`, `tick`, `year`, `settlement_id`, `trait_id`, `trait_count`, `trait_frequency`

Validation/analysis add-ons (enabled when needed):

1. `settlement_trait_deposition` for assemblage accumulation,
2. `network_interaction_snapshot` for edge topology inference,
3. core demographic/settlement/conflict diagnostics when calibrating non-cultural subsystems.

This output strategy keeps routine runs lightweight while preserving the ability to perform archaeological and network validation in targeted runs.
