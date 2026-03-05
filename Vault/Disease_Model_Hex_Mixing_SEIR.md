# Disease Model Hex Mixing SEIR

Purpose:

- Evaluate proposed network-based SEIR approach.
- Implement a performant heuristic compatible with the current hex ABM design.

## Evaluation

What is correct:

1. SEIR is an appropriate baseline for historical infectious disease dynamics.
2. Transmission should not be globally well-mixed across the full map.
3. Trade/mobility links are critical for long-range spread and epidemic shocks.

What to change for performance:

1. Do not simulate infection by pairwise agent contact in MVP.
2. Use universal mixing within each hex (your proposed heuristic) to reduce complexity.
3. Add sparse between-hex import via movement/trade flows to preserve spatial realism.

## Recommended MVP: Hybrid Hex-Mixing SEIR

Within each hex `h` each tick:

1. Track counts: `S_h, E_h, I_h, R_h, N_h`.
2. Compute force of infection:
   - `lambda_h = beta_h * (I_h / N_h)`.
3. Convert to infection probability:
   - `p_inf_h = 1 - exp(-lambda_h * dt)`.
4. New exposures:
   - `new_E_h ~ Binomial(S_h, p_inf_h)`.
5. Progression:
   - `new_I_h ~ Binomial(E_h, sigma_h)`.
   - `new_R_h ~ Binomial(I_h, gamma_h)`.
6. Optional mortality split:
   - move a share of `new_R_h` to death for demographic coupling.

Between hexes (import term):

1. Compute infectious import from incoming travelers/traders:
   - `I_import_h = sum_over_neighbors(flow_jh * I_j / N_j)`.
2. Adjust force of infection:
   - `lambda_h = beta_local_h * (I_h/N_h) + beta_import_h * (I_import_h/N_h)`.

This keeps computation near `O(num_hexes)` per tick for disease updates.

## Why This Fits Current Decisions

1. Seasonal model:
   - Use `dt = 0.25` years and calibrate `sigma`, `gamma` to seasonal transition probabilities.
2. Hex spatial model:
   - Infection and mobility already aggregate naturally by hex.
3. Performance goal:
   - Avoids expensive per-contact graph simulation at 100k+ agents.

## Risks and Guardrails

Risks:

1. Within-hex random mixing can overestimate spread in fragmented settlements.
2. No household clustering in MVP can flatten superspreader structure.

Guardrails:

1. Add per-hex contact multiplier by settlement density.
2. Keep feature flag for future household/social-network layer.
3. Validate against stylized outcomes:
   - low-density burn-out vs high-density persistence.

## Parameters to Expose

1. `beta_local_h`
2. `beta_import_h`
3. `sigma` (E->I progression)
4. `gamma` (I->R or I->D progression)
5. `ifr` or disease-specific fatality schedule
6. density/contact modifier by hex

## Source Notes

1. Kermack, W. O., & McKendrick, A. G. (1927). Foundational compartment framework.
2. Sattenspiel, L. (1990). Anthropological adaptation of epidemic models and mobility/social structure relevance.
3. Keeling, M. J., & Rohani, P. (2008). Network and stochastic epidemic mechanics for individual/contact structures.
