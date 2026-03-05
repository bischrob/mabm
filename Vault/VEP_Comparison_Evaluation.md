# VEP Comparison Evaluation

Purpose:

- Evaluate the provided VEP-vs-current-framework notes.
- Record what to adopt, modify, and reject for this project.

## Overall Assessment

The notes are directionally strong and useful, but some claims are overstated.

## What Is Strong and Should Be Adopted

1. Scale distinction is correct:
   - VEP-style high-resolution local land-use models and this project’s 18 km hex macro-regional model answer different questions.
2. Preprocessing high-res environmental data is correct:
   - Aggregate off-line; do not run 200m-cell calculations at runtime.
3. MAUP warning is correct:
   - Plain averaging can erase critical productive micro-zones.
4. Tiered arable-land representation is strong:
   - Fractional binning by suitability classes fits current performance constraints.
5. IDD integration with tier claims is strong:
   - Enables realistic differential vulnerability under drought.

## What Should Be Modified

1. "VEP is mostly environmental determinism":
   - Too strong. Better statement: VEP emphasizes subsistence/environment and household decisions, while your model elevates regional network and interaction dynamics.
2. "ECS is categorically faster than Java/OOP":
   - Too absolute. Better statement: your Rust data-oriented design is better aligned with targeted scale and deterministic batch updates.
3. "A* across massive regions as primary runtime mechanism":
   - Must be bounded/cached. Use precomputed route kernels and periodic refresh, not per-event global pathfinding.
4. "Practically instantaneous for 100,000 agents":
   - Overstated. Use profiling-based performance claims only.

## What Is Missing in the Notes

1. Uncertainty propagation from aggregation:
   - Need uncertainty bands for tier area/yield estimates.
2. Boundary-policy interaction:
   - Macro-regional network outputs depend on edge assumptions.
3. Validation linkage:
   - Need explicit mapping from simulated outputs to archaeological observables.

## Implementation Decision for This Project

Adopt:

1. Fractional binning preprocessor for high-res -> hex aggregation.
2. Hex `ArableLand` tier arrays:
   - `tier_hectares`, `tier_base_yields`, `claimed_hectares`.
3. Seasonal climate modifier applied at tier level.
4. IDD allocation over tiers (early arrivals preferential access).

Add safeguards:

1. Track per-hex aggregation uncertainty.
2. Run sensitivity tests with alternative bin definitions.
3. Use cached path metrics for network/transport systems.

## Minimal Next Step

Create a preprocessing spec file defining:

1. input rasters,
2. tier classification rules,
3. per-hex output schema,
4. uncertainty fields,
5. reproducible version hash for generated layers.
