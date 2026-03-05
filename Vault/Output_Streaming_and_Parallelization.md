# Output Streaming and Parallelization

Goal:

- Avoid keeping full history in memory.
- Keep write throughput high.
- Parallelize model updates without race conditions or nondeterministic drift.

## Output Without Large In-Memory Buffers

Use an async writer pipeline:

1. Simulation threads produce compact row batches per tick (or per N ticks).
2. Push batches to a bounded lock-free channel.
3. Dedicated writer thread(s) serialize to columnar files (Parquet preferred).
4. Rotate files by row count or time window (chunking).

Rules:

1. Never store full-run history in RAM.
2. Keep only current state + tiny rolling diagnostics.
3. Apply backpressure with bounded channel capacity.
4. If writer lags, downsample noncritical logs first (not core metrics).

## High-Performance Write Strategy

1. Write append-only, chunked files.
2. Batch rows (e.g., 10k-100k records per write).
3. Compress with fast codecs (zstd low/medium level).
4. Partition by `run_id` and coarse tick buckets.
5. Flush on interval and on graceful shutdown/checkpoint.

Recommended split:

1. `core_metrics` (every tick, never dropped)
2. `edge_events` (non-zero only)
3. `debug/detail` (sampled/downsampled)

## Deterministic Parallel Model Structure

Use staged tick execution with barriers:

1. Stage A: read-only state scan (parallel).
2. Stage B: compute intents/deltas into thread-local buffers (parallel).
3. Stage C: deterministic reduction/merge of deltas.
4. Stage D: apply merged updates to canonical state.
5. Stage E: emit output batches.

Key pattern:

- No thread writes directly into shared canonical state during Stage B.
- Merge order is deterministic (sorted by entity/hex id).

## What to Parallelize

1. Hex-level environment updates (independent per hex).
2. Settlement-level demand/resource accounting.
3. Disease updates by hex (SEIR counts).
4. Movement/travel-cost evaluation by agent/household partition.
5. Interaction proposal generation (trade/conflict intents).
6. Aggregations and metrics computation.

## What Should Stay Serialized (or Single-Merge)

1. Final conflict resolution when multiple intents target same resource/edge.
2. Final state-application step after delta merge.
3. Writer ordering for deterministic file content (if required).

## Data Layout and Runtime Best Practices (Rust)

1. Prefer SoA layout for hot loops.
2. Preallocate vectors and reuse buffers each tick.
3. Avoid per-agent heap allocations in hot path.
4. Use rayon for CPU parallel loops.
5. Use one-way message passing to writer (avoid shared mutex logging).

## Minimal Implementation Blueprint

1. `TickWorkerPool`:
   - runs Stage A/B in parallel partitions.
2. `DeltaMerger`:
   - deterministic sort/reduce.
3. `StateApplier`:
   - single pass apply.
4. `OutputEmitter`:
   - builds typed batch records.
5. `WriterService`:
   - async Parquet chunk writer with bounded queue.

## Failure Safety

1. Periodic checkpoints of canonical state only.
2. Write-ahead metadata for output chunks (chunk id + checksum).
3. On restart, resume from latest checkpoint and continue with new chunk series.
