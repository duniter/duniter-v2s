# Private indexer batch sink API

Duniter can stream raw finalized block batches to a private compatible indexer.
This API is opt-in and intended for local or private-network ingestion services
that store raw facts, decode runtimes, and build projections outside Duniter.

Duniter does not compute indexer business views. It only sends raw finalized
block data and raw top-level storage changes.

## Enable the stream

Start Duniter with the indexer batch sink URL. Providing this URL enables the
sink:

```sh
duniter \
  --state-pruning 256 \
  --blocks-pruning archive-canonical \
  --indexer-batch-sink-url http://127.0.0.1:9949/private/duniter
```

Optional bearer authentication:

```sh
install -m 600 /dev/null /path/to/indexer-sink-token
# Write the token to that file without exposing it in shell history or argv.
duniter \
  --state-pruning 256 \
  --blocks-pruning archive-canonical \
  --indexer-batch-sink-url http://127.0.0.1:9949/private/duniter \
  --indexer-batch-sink-token-file /path/to/indexer-sink-token
```

When no token file is configured, Duniter reads the token from
`DUNITER_INDEXER_BATCH_SINK_TOKEN`. The environment variable name can be
changed with `--indexer-batch-sink-token-env <ENV_VAR>`. The token itself is
never accepted as a command-line argument.

HTTP delivery is handled by a non-essential asynchronous task. If the indexer
is unavailable, Duniter keeps complete batches on disk and retries without
blocking finalization.

Capture and journal append are part of the block-import pipeline and are
therefore import-critical. Duniter rejects a block import if its batch cannot be
appended. Durability is established by the 100-record group commit described
below; its recoverable crash-loss window is the reason for the pruning checks.

## Required endpoints

The configured URL is a base URL. The compatible indexer must expose:

```text
POST <base-url>/stream-start
POST <base-url>/batches
```

For example, with `--indexer-batch-sink-url http://127.0.0.1:9949/private/duniter`,
this means:

```text
POST /private/duniter/stream-start
POST /private/duniter/batches
```

If a sink token file or environment variable is configured, both endpoints receive:

```http
Authorization: Bearer <token>
```

## Stream start

Duniter calls `stream-start` before sending batches. The request body is JSON:

```json
{
  "chain_id": "g1",
  "genesis_hash": "0xfeb770bbb0344dabc8366b0d1f889a8e4e6ca09b914006655fe795920deb6d56"
}
```

A compatible indexer should validate that the announced chain matches the store
it is about to write to.

The 2xx response body can be empty. If it is JSON, the compatible indexer can
tell Duniter where to resume by returning the last finalized batch it has
durably ingested:

```json
{
  "last_received_finalized_block": {
    "number": 123,
    "hash": "0x..."
  }
}
```

Equivalent top-level fields are also accepted:

```json
{
  "last_received_finalized_block_number": 123,
  "last_received_finalized_block_hash": "0x..."
}
```

If no last received block is provided, Duniter starts at genesis. The genesis
batch contains an `Upsert` for every top-level genesis storage entry, so an
empty indexer can reconstruct its initial state without an archive node. If a
cursor is provided, Duniter treats the indexer as the source of truth and starts
at `number + 1`. Providing the hash is strongly recommended because Duniter logs
local continuity disagreements. If Duniter knows the canonical block hash at
that number and it differs from the indexer hash, Duniter refuses to open the
stream and logs a loud error every 5 seconds: this means the indexer is on a
different finalized history from the chain Duniter knows.

To guarantee a complete history, enable the sink when creating and synchronizing
the Duniter database for the first time.

**If the sink was not enabled when the existing database was synchronized, do
not reuse that database for the sink. Resynchronize Duniter from genesis with
the sink enabled and `--sync full`.** Blocks imported before the sink was enabled
do not have the required import-time storage changes, even if their headers and
bodies are still available locally.

If `stream-start` fails, Duniter logs a loud error every 5 seconds and does not
drop queued finalized batches.

## Batch request

Duniter sends SCALE chunks containing up to 64 consecutive finalized batches
during historical catch-up. Once caught up, each newly finalized block is sent
immediately as a one-element chunk, without an accumulation delay:

```http
POST /private/duniter/batches
Content-Type: application/octet-stream
Authorization: Bearer <token>
Duniter-Batch-Mode: historical | live
```

`Duniter-Batch-Mode` is mandatory. Duniter sends `historical` until the node is
within two best blocks of the network sync target (not merely outside
Substrate's roughly 256-block major-sync window) and the initial range requested
by the cursor returned from `stream-start` is fully acknowledged. The two-block
tolerance avoids an unreachable moving-target race while keeping the boundary
within a few seconds of real time. Local and development chains without a
network target use their local head as that boundary. Once the boundary is
crossed, every later finalized batch is sent as `live`, including gap catch-up
after an interruption and batches recovered through chain continuity after an
in-memory wake-up cursor was dropped.

The header describes the producer path, not whether a consumer is bootstrapping.
A consumer that maintains a transient event journal should suppress historical
notifications only until it has accepted its first `live` chunk. Once live mode
has started, historical gap catch-up represents missed real-time data and should
remain replayable.

The request body is a SCALE-encoded `Vec<Vec<u8>>`. Each inner byte vector is
the unchanged SCALE encoding of one finalized batch described below. The
indexer must validate and durably commit the complete chunk atomically before
returning 2xx. Duniter advances a cumulative
acknowledgement cursor and deletes a journal segment only when every record in
the acknowledged chunk is committed or belongs to a stale branch. Failed segment deletion is
retried every 5 seconds by an independent background task and never blocks
delivery.
Duplicate batches must also be acknowledged with a 2xx response: a 409 is never
sufficient proof that the stored block has the same hash and content.

## Inner batch SCALE format

Each element of the outer `Vec<Vec<u8>>` uses this field order:

```text
u32 fixture_format_version = 1

String chain_id
[u8; 32] genesis_hash

String runtime.spec_name
u32 runtime.spec_version
u32 runtime.transaction_version

u32 block.number
[u8; 32] block.hash
[u8; 32] block.parent_hash
[u8; 32] block.state_root
[u8; 32] block.extrinsics_root
Option<Vec<u8>> block.header_raw_bytes

Vec<Extrinsic> extrinsics:
  u32 index
  Vec<u8> raw_bytes
  Option<[u8; 32]> hash

Vec<StorageChange> storage_changes:
  Vec<u8> raw_key
  Option<Vec<u8>> old_raw_value
  Option<Vec<u8>> new_raw_value
  StorageOperation operation

StorageOperation:
  0 = Upsert
  1 = Delete
```

Rules for consumers:

- `fixture_format_version` must be `1`.
- `runtime` identifies the runtime from the parent state that decoded and
  executed the block. For genesis, it identifies the genesis runtime.
- `block.header_raw_bytes`, when present, must hash to `block.hash` with
  BLAKE2b-256.
- `extrinsic.hash`, when present, must equal BLAKE2b-256 over
  `extrinsic.raw_bytes`.
- `Upsert` must include `new_raw_value`.
- `Delete` must not include `new_raw_value`.
- `old_raw_value` can be absent when Duniter cannot cheaply provide it.
- Storage changes are captured by the block-import pipeline, not reconstructed
  from targeted JSON-RPC reads.
- Child-storage changes are outside this protocol. `storage_changes` covers
  top-level storage only.

## Ordering and finality

Duniter sends only finalized blocks. A compatible indexer can therefore avoid
reorg semantics for this stream.

The stream preserves finalized block order. The indexer should reject a batch
when the parent is missing or when the next block does not continue the stored
finalized chain.

## Historical catch-up

When the indexer reports an old cursor at `stream-start`, Duniter catches it up
before sending live queued batches.

When the sink is enabled, Duniter uses the block-import pipeline. Each candidate
block is encoded once as a complete batch, including its top-level storage diff,
and appended to a segmented journal. Only canonical finalized batches are
announced to the sender. The journal directory is:

```text
indexer-batch-sink/journal
```

Duniter stores many batches in each 64 MiB append-only segment and group-commits
the journal every 100 appended records. This avoids per-block file creation,
rename and directory synchronization. A crash can discard the uncommitted tail,
containing at most 99 records. On restart Duniter reconstructs missing canonical
batches by re-executing the retained block bodies against their retained parent
states.

The sink therefore requires explicit block and state pruning settings retaining
at least 100 finalized blocks. Archive modes are also accepted. Duniter refuses
to start otherwise. The normal default state window is 256, but it must be
specified explicitly when enabling the sink so an existing database with a
smaller stored window cannot be used accidentally.

For blocks imported before this feature was enabled, Duniter can serve the batch
only while the required block body and parent state are retained. Re-executing
an entire old chain is intentionally not used as a substitute for resyncing with
the sink enabled.

This requires the historical block body and runtime parent state.

If the block data is unavailable, Duniter logs:

```text
historical_block_pruned
```

If the block body exists but the runtime state needed to describe the batch is
unavailable, Duniter logs:

```text
historical_state_pruned
```

Those cases are intentionally distinct: a compatible deployment can fix missing
block data by retaining/importing block bodies and missing runtime state by
running with enough state history for the requested range.

## Idempotency

Duniter may resend a batch after a network error or timeout. Consumers must make
batch ingestion idempotent.

Recommended behavior:

- If the same finalized block number and hash is already stored with identical
  raw content, return 2xx. The body may identify it as a duplicate for logging.
- If the same block number is already stored with a different hash, reject the
  batch as a finalized block conflict.
- Make the acknowledgement happen after the raw batch is durably accepted.

Duniter treats a duplicate as acknowledged only when the response status is
successful. In particular, a text-matched `409 Conflict` is not accepted,
because it could describe the same block number with a different hash.

## Error responses

Use clear response bodies. Duniter classifies known failures from response text
and logs them distinctly when possible:

- invalid genesis hash;
- malformed SCALE body;
- accidental JSON submission to `/batches`;
- invalid batch hash;
- malformed storage change;
- missing parent or stream continuity error;
- historical block pruned;
- historical state pruned;
- unsupported runtime projection;
- duplicate already-ingested batch.

`unsupported_runtime` should not be a transport failure if raw facts were
stored successfully. Return 2xx and include `unsupported_runtime` in the body if
the raw batch is accepted but projections cannot be decoded yet.

## Backpressure

Duniter keeps a bounded queue of finalized block cursors in memory and their
complete encoded batches on disk until the indexer acknowledges them or reports
a later cursor. The CLI option `--indexer-batch-sink-max-queue-len <n>` sets that
cursor bound. Once it is reached, redundant wake-up cursors are discarded
safely: chain continuity lets the sender discover every pending batch from the
durable spool.

Enabling the sink disables warp/state sync. The node must execute every block
from genesis so that every top-level storage transition can be captured. The
node rejects `--sync fast`, `--sync fast-unsafe`, and `--sync warp` when the sink
URL is configured.

The sink additionally requires explicit `--state-pruning` and
`--blocks-pruning` values of at least `100`, or an archive mode, so the tail of
the 100-record group-commit window can be recovered after a crash.

Consumers should avoid long request handling that blocks ingestion. A robust
pattern is:

1. Read and validate the full binary body.
2. Store raw batch facts inside a transaction keyed by `(chain_id, block.number,
   block.hash)`.
3. Commit the transaction.
4. Return the HTTP acknowledgement.
5. Decode runtime-specific projections asynchronously.

## Security

This API is private. A production deployment should not expose it directly to
the public internet.

Minimum recommendations:

- bind the indexer private API to loopback or a private network;
- configure `--indexer-batch-sink-token-file` (preferred) or
  `DUNITER_INDEXER_BATCH_SINK_TOKEN` when the endpoint is reachable by other
  processes or hosts;
- put the indexer behind a reverse proxy with TLS if remote submission is
  required;
- reject unsupported `Content-Type` values and missing or invalid
  `Duniter-Batch-Mode` values on `/batches`.

## End-to-end test

The node crate contains an in-process indexer mock that accepts `stream-start`,
decodes the complete SCALE batch, validates the stream identity and bearer
token, and acknowledges the received batch. The test launches a real
`gtest_local` node, so it must run in an environment where local HTTP and P2P
sockets are allowed:

```sh
cargo test -p duniter --no-default-features --features gtest,fast \
  --test indexer_sink_e2e -- --nocapture
```
