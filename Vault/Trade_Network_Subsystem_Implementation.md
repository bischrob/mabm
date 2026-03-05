# Trade Network Subsystem Implementation

Date: 2026-03-04

## Why this exists in the ABM

Interaction networks are a primary model objective, not a post-hoc diagnostic. The trade subsystem is required so settlement stress and surplus conditions can alter exchange connectivity and caloric buffering in a causal way.

## Implemented mechanics

1. Per-tick dynamic edge formation:
   - Candidate edges are evaluated pairwise across occupied settlements.
   - Edge strength uses trait similarity, stress-gap penalty, and effective trade labor scale.

2. Tie persistence/decay:
   - Prior edge weights are carried forward and blended with current suitability.
   - New ties initialize directly from current suitability.
   - Low-weight edges are dropped via a minimum threshold.

3. Directed surplus-to-deficit transfer:
   - Donor/receiver roles are assigned by deficit comparison.
   - Transfer amount is capped by donor surplus buffer, receiver need, and edge capacity.
   - Food-store and deficit updates are applied after all pair evaluations for deterministic state updates.

4. Persistent trade ledger in model state:
   - `SimulationState.trade_edges` stores active edge rows (`source`, `target`, `weight`, `goods`, `tick`).
   - Network outputs now serialize this ledger directly instead of reconstructing proxy edges.

## Output/metrics integration

- `collect_network_snapshot_rows` now emits rows from the explicit trade edge ledger.
- Baseline network metrics now reflect actual model-generated trade edges.

## Validation added

- `trade_network_transfers_calories_from_surplus_to_deficit` test verifies:
  - receiver stores increase,
  - active trade edges are produced,
  - at least one edge transacts positive goods.

## Scope note

This remains MVP-level trade dynamics. It provides deterministic and interpretable exchange behavior while leaving room for future route-cost pathfinding, explicit tie types, and debt/trust memory.
