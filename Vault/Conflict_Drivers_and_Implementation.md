# Conflict Drivers and Implementation

Purpose:

- Summarize research-backed conflict drivers for premodern ABM.
- Define a performant implementation for seasonal, hex-based simulation.

## Evidence-Based Drivers to Model

1. Resource stress and unpredictability:
   - Drought/shortfall increases raiding risk, especially where storage buffers fail.
2. Population pressure and local circumscription:
   - Limited high-quality land/water raises competition over defendable patches.
3. Social scale and coordination strain:
   - Large groups without integrative institutions face higher internal/external violence risk.
4. Network exposure/opportunity:
   - More contact edges increase both trade and conflict opportunities.
5. Shock amplification:
   - Epidemics and rapid demographic collapse can destabilize deterrence and defense.

## Recommended Conflict Model (MVP)

Use settlement-level hazard + dyadic target choice.

For settlement `s` at tick `t`:

1. Compute attack propensity:
   - `A_s = sigmoid(b0 + b1*stress_food_s + b2*pop_pressure_s + b3*recent_losses_s + b4*network_exposure_s - b5*defense_s)`
2. Draw attack intent:
   - `attack_intent_s ~ Bernoulli(A_s)`
3. If intent = 1, score candidates `d` in reachable radius:
   - `U_sd = c1*expected_gain_sd - c2*travel_cost_sd - c3*target_defense_d + c4*grievance_sd + c5*opportunity_sd`
4. Choose `d = argmax(U_sd)` above minimum utility threshold.
5. Resolve outcome stochastically:
   - `P_success_sd = sigmoid(k0 + k1*attacker_force_s - k2*defender_force_d - k3*fortification_d - k4*distance_sd)`
6. Apply consequences:
   - mortality/dispersal,
   - store/resource losses,
   - retaliation memory update.

This avoids per-agent combat simulation and scales well.

## Best Practices for This Project

1. Keep conflict generation at settlement level for speed.
2. Separate:
   - conflict onset hazard,
   - target selection,
   - battle outcome.
3. Include memory terms:
   - recent raids raise retaliation probability for limited horizon.
4. Add hard floors:
   - no attack if labor/food deficit makes expedition impossible.
5. Couple with travel-cost model:
   - rough terrain lowers opportunity via higher expedition cost.
6. Keep all coefficients scenario-configurable for sweeps.

## Minimal Calibration Outputs

1. `attacks_initiated`
2. `attacks_successful`
3. `conflict_mortality`
4. `resource_loss_kcal`
5. `mean_attack_distance`
6. `retaliation_rate`
7. `conflict_cluster_persistence`

## Sources

Foundational/empirical conflict context:

1. Carneiro, R. L. (1970). Circumscription and warfare theory:
   - https://www.science.org/doi/10.1126/science.169.3947.733
2. Kelly, R. C. (2000). Evolutionary/anthropological synthesis of war:
   - https://www.annualreviews.org/doi/10.1146/annurev.anthro.29.1.873
3. Ember, C. R., & Ember, M. (1992). Resource unpredictability and war:
   - https://journals.sagepub.com/doi/10.1177/0022002792036003001
4. Fry, D. P., & Soderberg, P. (2013). Lethal aggression in mobile foragers:
   - https://pubmed.ncbi.nlm.nih.gov/23898172/

Modeling/ABM implementation reference:

5. Epstein, J. M. (2002). Agent-based civil violence model:
   - https://www.pnas.org/doi/10.1073/pnas.132341299

Southwest archaeology context and data-rich ABM ecosystem:

6. Village Ecodynamics Project overview volume:
   - https://www.ucpress.edu/book/9780520271663/emergence-and-collapse-of-early-villages

Note:

- Use these as theory/calibration anchors, then fit coefficients to your synthetic + GIS-informed scenario tracks.
