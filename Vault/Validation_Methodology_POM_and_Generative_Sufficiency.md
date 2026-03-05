# Validation Methodology POM and Generative Sufficiency

Purpose:

- Record validation strategy for a long-horizon historical ABM where direct experimentation is impossible.

## Core Framing

1. Verification is necessary but insufficient:
   - passing tests and correct code execution do not validate causal realism.
2. Validation target:
   - demonstrate that model mechanisms can generate observed historical patterns for the right reasons.
3. Guiding principle:
   - Generative sufficiency ("If you didn’t grow it, you didn’t explain it").

## 1) Pattern-Oriented Modeling (POM)

Use multiple target patterns across scales, defined before calibration:

1. Macro:
   - long-run regional population trajectory (booms, busts, timing).
2. Meso:
   - settlement clustering/aggregation shifts (including defensible terrain occupation).
3. Micro:
   - distance-decay and style-distribution patterns in interaction/material culture.

Validation rule:

- A candidate model must reproduce several independent patterns simultaneously, not just one headline metric.

## 2) Empirical Ground-Truthing of Network Topology

For simulated vs empirical network comparisons:

1. Build similarity-based settlement networks from simulated deposited assemblages.
2. Compute topology metrics on both simulated and empirical networks:
   - density,
   - modularity,
   - centrality profiles,
   - community structure timing.
3. Compare temporal transitions (fragmentation/reorganization windows), not only static snapshots.

## 3) Sensitivity Analysis and Parameter Sweeps

Goal:

- prove robustness and avoid brittle parameter dependence.

Method:

1. Use Latin Hypercube Sampling (LHS) over bounded priors.
2. Run large replicate sets with controlled seeds.
3. Evaluate whether key target patterns persist across broad parameter regions.

Failure signal:

- tiny perturbations produce systematic model collapse or complete pattern reversal.

## 4) Knockout (Null) Experiments

Method:

1. Run full baseline with all major mechanisms active.
2. Disable one mechanism at a time (feature-flag knockout).
3. Measure which target patterns disappear or degrade.

Interpretation:

- If removing a mechanism destroys specific historical pattern fits, that mechanism has explanatory necessity in the current model family.

## Rust Implementation Notes

1. Keep mechanism toggles as explicit feature flags in config.
2. Build a sweep runner that:
   - ingests LHS parameter matrix,
   - executes runs in parallel (rayon/process pool),
   - writes compact metric summaries per run.
3. Persist run metadata for reproducibility:
   - run version,
   - seed,
   - config hash,
   - knockout flag set.

## Practical Validation Stack for This Project

1. Simulated-data phase:
   - internal pattern consistency,
   - sweep robustness,
   - knockout diagnostics.
2. Data-integrated phase:
   - assemblage/network comparison against archaeological datasets,
   - proxy-specific acceptance bands,
   - out-of-sample tests.

## Candidate Metrics to Track Per Run

1. Pattern fit:
   - population-curve error,
   - aggregation timing error,
   - distance-decay fit error.
2. Network structure:
   - density/modularity/centrality deltas vs target.
3. Stability:
   - collapse frequency,
   - recovery time,
   - compound-shock response.
