# Trade Network Dynamics Implementation

Purpose:

- Evaluate trade-network formation, persistence, and collapse for this ABM.
- Specify a high-performance implementation for seasonal, 100k+ agent scale.

## Evaluation of Proposed Approach

Strong:

1. Gravity-style partner selection is a good baseline for spatial exchange.
2. Edge-strength decay is the right mechanism for persistence/break conditions.
3. Route difficulty and transport loss should directly reduce realized exchange.

Refinements for your constraints:

1. Build trade links at settlement level (not household/agent global graph) for MVP.
2. Do not run A* for every attempted trade event.
3. Precompute route kernels (cost and survival) and refresh periodically.
4. Keep reciprocity debt and trust as edge state variables.

## Ideal Solution Under Current Constraints

Use a dynamic sparse settlement graph with three layers:

1. Candidate layer:
   - settlements within max effective travel cost.
2. Relationship layer:
   - active ties with `strength`, `debt_balance`, `last_exchange_tick`.
3. Route layer:
   - precomputed least-cost path stats between local candidate pairs:
   - `travel_days`, `survival_factor_by_good`, `risk_factor`.

### 1) Formation (Risk-Buffered Gravity Rule)

For settlements `i, j` in candidate radius:

`score_ij = k * (mass_i^alpha) * (mass_j^beta) * exp(-gamma * cost_ij) * risk_need_i * complementarity_ij`

Where:

- `mass` can be population or surplus.
- `cost_ij` is terrain/weather-weighted travel cost.
- `risk_need_i` increases when local storage reliability falls.
- `complementarity_ij` captures differing resource profiles.

Create tie if `score_ij` exceeds stochastic threshold.

### 2) Persistence and Break

Edge state:

- `strength: f32`
- `debt_balance: f32`
- `last_exchange_tick: u32`

Update each tick:

1. Decay:
   - `strength <- strength * exp(-lambda_decay * dt)`
2. Reinforcement after successful exchange:
   - `strength <- min(1.0, strength + delta_reinforce * reciprocity_quality)`
3. Debt penalty:
   - if `abs(debt_balance) > debt_limit`, apply extra decay.
4. Break condition:
   - remove edge if `strength < break_threshold`.

### 3) Transport Loss and Route Friction

Per-hex good survival:

`S_hex(good) = clamp(1 - (base_loss_good + mu_good * roughness_hex + weather_penalty_hex), 0, 1)`

Path survival:

`S_path(good) = product(S_hex(good) over route)`

Net delivered goods:

`G_recv = G_sent * S_path(good)`

Use route utility:

`U_trade = value_recv - transport_labor_cost - security_risk_cost`

Only execute exchanges with positive expected utility.

## Performance Architecture

1. Settlement graph only in MVP.
2. Spatial index by hex and bounded candidate radius.
3. Route precompute:
   - recompute every N ticks or when seasonal terrain modifiers change.
4. Trade simulation uses cached route metrics, not live pathfinding.
5. Store graph in compact adjacency vectors, not heavyweight per-edge objects if profiling demands.

Complexity target:

- Candidate discovery near `O(S * k)` with bounded neighbors `k`.
- Edge updates near `O(E_active)`.
- Route recompute amortized by periodic batches.

## Calibration Outputs

1. Active tie count over time.
2. Edge survival half-life.
3. Mean reciprocity imbalance.
4. Mean trade distance and route roughness.
5. Delivered/sent ratio by good type.
6. Network fragmentation around drought shocks.

## Sources

Risk buffering and exchange logic:

1. Halstead, P., & O'Shea, J. (1989). *Bad Year Economics*.
   - https://www.cambridge.org/core/books/bad-year-economics/4E0184E0A454D740ACA7A523ACAABC53
2. Wiessner, P. (1982). Risk, reciprocity, and hxaro exchange.
   - https://www.jstor.org/stable/3629819

Spatial interaction and distance-decay in archaeology:

3. Renfrew, C. (1977). Alternative models for exchange and spatial distribution.
   - https://www.cambridge.org/core/books/exchange-systems-in-prehistory/9A6D6AAE9A6D7669E0E0A585BDE8F256
4. Peeples, M. A., & Roberts, J. M. (2013). Social network analysis in archaeology.
   - https://www.annualreviews.org/doi/10.1146/annurev-anthro-092412-155706

Modeling reference for gravity-like interaction and network dynamics:

5. Wilson, A. G. (1971). A family of spatial interaction models.
   - https://www.jstor.org/stable/143141

Note:

- Keep gravity exponents and decay constants as sweepable priors.
- Use settlement-level network first; household-level network can be a later feature flag.
