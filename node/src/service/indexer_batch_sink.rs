// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

use super::FullClient;
use common_runtime::Block;
use futures::{StreamExt, channel::mpsc};
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use reqwest::header;
use sc_client_api::{BlockBackend, BlockchainEvents, HeaderBackend, StorageProvider};
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ImportResult, StateAction,
    StorageChanges as ImportStorageChanges,
};
use sc_service::{SpawnTaskHandle, error::Error as ServiceError};
use sp_api::{ApiExt, CallApiAt, Core as CoreApi, ProvideRuntimeApi};
use sp_consensus::SyncOracle;
use sp_core::{H256, traits::CallContext};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Hash, Header as HeaderT};
use sp_storage::{StorageData, StorageKey};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt, fs,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

const FIXTURE_FORMAT_VERSION: u32 = 1;
const STREAM_START_RETRY: Duration = Duration::from_secs(5);
const INITIAL_BATCH_RETRY: Duration = Duration::from_secs(1);
const MAX_BATCH_RETRY: Duration = Duration::from_secs(30);
const CLEANUP_RETRY: Duration = Duration::from_secs(5);
const PROGRESS_LOG_INTERVAL: usize = 100;
const SINK_CHUNK_MAX_BATCHES: usize = 64;
const LIVE_SYNC_TARGET_MAX_LAG: u32 = 2;
const LIVE_FINALIZED_BACKLOG_MAX_LAG: u32 = 2;
const BATCH_MODE_HEADER: &str = "duniter-batch-mode";
pub(super) const GROUP_COMMIT_BLOCKS: u32 = 100;
const JOURNAL_SEGMENT_MAX_BYTES: u64 = 64 * 1024 * 1024;
const JOURNAL_RECORD_MAGIC: [u8; 4] = *b"DIBJ";
const JOURNAL_RECORD_HEADER_BYTES: u64 = 4 + 4 + 32;
static DROPPED_CURSOR_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(codec::Encode, codec::Decode, Clone)]
struct FinalizedBatch {
    fixture_format_version: u32,
    chain_id: String,
    genesis_hash: [u8; 32],
    runtime: RuntimeIdentity,
    block: BlockData,
    extrinsics: Vec<ExtrinsicData>,
    storage_changes: Vec<StorageChange>,
}

#[derive(codec::Encode, codec::Decode, Clone)]
struct RuntimeIdentity {
    spec_name: String,
    spec_version: u32,
    transaction_version: u32,
}

#[derive(codec::Encode, codec::Decode, Clone)]
struct BlockData {
    number: u32,
    hash: [u8; 32],
    parent_hash: [u8; 32],
    state_root: [u8; 32],
    extrinsics_root: [u8; 32],
    header_raw_bytes: Option<Vec<u8>>,
}

#[derive(codec::Encode, codec::Decode, Clone)]
struct ExtrinsicData {
    index: u32,
    raw_bytes: Vec<u8>,
    hash: Option<[u8; 32]>,
}

#[derive(codec::Encode, codec::Decode, Clone)]
struct StorageChange {
    raw_key: Vec<u8>,
    old_raw_value: Option<Vec<u8>>,
    new_raw_value: Option<Vec<u8>>,
    operation: StorageOperation,
}

#[derive(codec::Encode, codec::Decode, Clone)]
enum StorageOperation {
    Upsert,
    Delete,
}

#[derive(Clone)]
struct ObservedStorageChange {
    raw_key: StorageKey,
    new_raw_value: Option<StorageData>,
}

#[derive(Clone)]
struct QueuedBatch {
    number: u32,
    hash: H256,
}

#[derive(codec::Encode, codec::Decode, Clone, Copy, Debug)]
struct BestBlockRef {
    number: u32,
    hash: [u8; 32],
}

#[derive(codec::Encode, codec::Decode)]
struct BestChainUpdate {
    fixture_format_version: u32,
    chain_id: String,
    genesis_hash: [u8; 32],
    from: BestBlockRef,
    to: BestBlockRef,
    retracted: Vec<BestBlockRef>,
    enacted_batches: Vec<Vec<u8>>,
}

// Legacy per-file store types are kept temporarily to make the on-disk format transition
// explicit in tests; the active sink exclusively uses `SegmentedBatchJournal`.
#[allow(dead_code)]
#[derive(codec::Encode, codec::Decode, Clone)]
struct PersistedStorageDiff {
    number: u32,
    hash: H256,
    storage_changes: Vec<StorageChange>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct StorageDiffStore {
    dir: Arc<PathBuf>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct FinalizedBatchStore {
    dir: Arc<PathBuf>,
}

#[derive(Default)]
struct ProgressLogCounter {
    count: usize,
    first_number: u32,
}

impl ProgressLogCounter {
    fn record(&mut self, number: u32) -> Option<(u32, u32, usize)> {
        if self.count == 0 {
            self.first_number = number;
        }
        self.count += 1;
        if self.count < PROGRESS_LOG_INTERVAL {
            return None;
        }

        let progress = (self.first_number, number, self.count);
        self.count = 0;
        Some(progress)
    }
}

#[derive(Clone, Copy)]
struct JournalLocation {
    segment_id: u64,
    payload_offset: u64,
    payload_len: u32,
}

type JournalKey = (u32, H256);
type RecoveredJournalSegment = (Vec<JournalKey>, Vec<(JournalKey, JournalLocation)>, u64);

struct JournalSegment {
    records: Vec<JournalKey>,
    file: fs::File,
    size: u64,
}

struct JournalInner {
    dir: PathBuf,
    segments: BTreeMap<u64, JournalSegment>,
    index: BTreeMap<JournalKey, JournalLocation>,
    current_segment_id: u64,
    unsynced_records: u32,
    acknowledged_up_to: Option<u32>,
    acknowledgements_since_gc: u32,
    stale: BTreeSet<(u32, H256)>,
    cleanup_directory_dirty: bool,
}

#[derive(Clone)]
pub(crate) struct SegmentedBatchJournal {
    inner: Arc<Mutex<JournalInner>>,
}

#[derive(Default)]
struct PendingCleanup {
    up_to: Option<u32>,
}

#[derive(Clone, Default)]
struct CleanupScheduler {
    pending: Arc<Mutex<PendingCleanup>>,
}

pub(crate) struct IndexerBlockImport<RuntimeApi, Executor, Inner> {
    inner: Inner,
    client: Arc<FullClient<RuntimeApi, Executor>>,
    journal: Option<SegmentedBatchJournal>,
    chain_id: String,
    genesis_hash: H256,
}

impl<RuntimeApi, Executor, Inner: Clone> Clone for IndexerBlockImport<RuntimeApi, Executor, Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            client: self.client.clone(),
            journal: self.journal.clone(),
            chain_id: self.chain_id.clone(),
            genesis_hash: self.genesis_hash,
        }
    }
}

#[derive(Clone)]
struct HttpSink {
    client: reqwest::Client,
    stream_start_url: String,
    batches_url: String,
    best_chain_url: String,
    token: Option<String>,
    chain_id: String,
    genesis_hash: H256,
}

pub(crate) struct BatchSinkConfig {
    pub(crate) requires_network_sync_target: bool,
    pub(crate) base_url: String,
    pub(crate) token: Option<String>,
    pub(crate) max_queue_len: usize,
    pub(crate) chain_id: String,
    pub(crate) genesis_hash: H256,
}

struct BatchSender<RuntimeApi, Executor> {
    client: Arc<FullClient<RuntimeApi, Executor>>,
    sync_service: Arc<sc_network_sync::SyncingService<Block>>,
    requires_network_sync_target: bool,
    sink: HttpSink,
    max_queue_len: usize,
    journal: SegmentedBatchJournal,
    cleanup_scheduler: CleanupScheduler,
}

struct HistoricalRangeContext<'a, RuntimeApi, Executor> {
    client: &'a Arc<FullClient<RuntimeApi, Executor>>,
    sink: &'a HttpSink,
    max_queue_len: usize,
    journal: &'a SegmentedBatchJournal,
    cleanup_scheduler: &'a CleanupScheduler,
}

#[derive(Clone, Copy, Debug)]
struct BlockCursor {
    number: u32,
    hash: Option<H256>,
}

#[derive(Clone, Copy, Debug)]
struct StreamStartAck {
    resume_from: u32,
    last_received: Option<BlockCursor>,
    best: Option<BlockCursor>,
}

#[derive(Clone, Copy, Debug)]
enum BatchDeliveryMode {
    Historical,
    Live,
}

impl BatchDeliveryMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Historical => "historical",
            Self::Live => "live",
        }
    }
}

#[derive(Debug)]
enum BatchBuildError {
    HistoricalBlockPruned {
        number: u32,
        hash: Option<H256>,
        detail: String,
    },
    HistoricalStatePruned {
        number: u32,
        hash: H256,
        detail: String,
    },
    #[allow(dead_code)]
    HistoricalStorageDiffUnavailable {
        number: u32,
        hash: H256,
        detail: String,
    },
    Unavailable {
        hash: H256,
        detail: String,
    },
}

impl fmt::Display for BatchBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HistoricalBlockPruned {
                number,
                hash,
                detail,
            } => write!(
                f,
                "historical_block_pruned: block_number={number} block_hash={hash:?}; {detail}"
            ),
            Self::HistoricalStatePruned {
                number,
                hash,
                detail,
            } => write!(
                f,
                "historical_state_pruned: block_number={number} block_hash={hash:?}; {detail}"
            ),
            Self::HistoricalStorageDiffUnavailable {
                number,
                hash,
                detail,
            } => write!(
                f,
                "historical_storage_diff_unavailable: block_number={number} block_hash={hash:?}; {detail}"
            ),
            Self::Unavailable { hash, detail } => {
                write!(f, "batch_unavailable: block_hash={hash:?}; {detail}")
            }
        }
    }
}

#[allow(dead_code)]
impl StorageDiffStore {
    pub(crate) fn new(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|err| {
            format!(
                "failed to create compatible indexer storage diff directory {}: {err}",
                dir.display(),
            )
        })?;

        Ok(Self { dir: Arc::new(dir) })
    }

    fn persist(&self, diff: &PersistedStorageDiff) -> Result<(), String> {
        let path = self.path(diff.number, diff.hash);
        write_durable(&path, &codec::Encode::encode(diff), "storage diff")
    }

    fn load(&self, number: u32, hash: H256) -> Result<PersistedStorageDiff, BatchBuildError> {
        let path = self.path(number, hash);
        let bytes =
            fs::read(&path).map_err(|err| BatchBuildError::HistoricalStorageDiffUnavailable {
                number,
                hash,
                detail: format!(
                    "persisted storage diff is unavailable at {}; this block was likely finalized before the indexer batch sink started journaling diffs, or the diff journal was pruned manually: {err}",
                    path.display(),
                ),
            })?;

        let mut input = bytes.as_slice();
        let diff = <PersistedStorageDiff as codec::Decode>::decode(&mut input)
            .map_err(|err| BatchBuildError::HistoricalStorageDiffUnavailable {
                number,
                hash,
                detail: format!(
                    "persisted storage diff at {} is corrupted or uses an unsupported format: {err}",
                    path.display(),
                ),
            })?;

        if !input.is_empty() {
            return Err(BatchBuildError::HistoricalStorageDiffUnavailable {
                number,
                hash,
                detail: format!(
                    "persisted storage diff at {} contains {} unexpected trailing bytes",
                    path.display(),
                    input.len(),
                ),
            });
        }

        if diff.number != number || diff.hash != hash {
            return Err(BatchBuildError::HistoricalStorageDiffUnavailable {
                number,
                hash,
                detail: format!(
                    "persisted storage diff at {} does not match requested block: stored_number={} stored_hash={:?}",
                    path.display(),
                    diff.number,
                    diff.hash,
                ),
            });
        }

        Ok(diff)
    }

    fn remove(&self, number: u32, hash: H256) {
        let path = self.path(number, hash);
        if let Err(err) = remove_file_if_exists(&path, "storage diff") {
            warn!(
                "Could not remove compatible indexer storage diff {}: {err}",
                path.display(),
            );
        }
    }

    fn try_remove(&self, number: u32, hash: H256) -> Result<(), String> {
        remove_file_if_exists(&self.path(number, hash), "storage diff")
    }

    fn remove_hash(&self, hash: H256) {
        let hash_hex = hex::encode(hash.as_bytes());
        let entries = match fs::read_dir(self.dir.as_ref()) {
            Ok(entries) => entries,
            Err(err) => {
                warn!(
                    "Could not scan compatible indexer storage diff directory {} for stale hash {:?}: {err}",
                    self.dir.display(),
                    hash,
                );
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
                continue;
            };
            if file_name.contains(&hash_hex)
                && let Err(err) = fs::remove_file(&path)
                && err.kind() != std::io::ErrorKind::NotFound
            {
                warn!(
                    "Could not remove stale compatible indexer storage diff {}: {err}",
                    path.display(),
                );
            }
        }
    }

    fn remove_up_to(&self, number: u32) -> Result<(), String> {
        remove_files_up_to(self.dir.as_ref(), number, "storage diff")
    }

    fn path(&self, number: u32, hash: H256) -> PathBuf {
        self.dir.join(format!(
            "{number:010}-{}.scale",
            hex::encode(hash.as_bytes())
        ))
    }
}

#[allow(dead_code)]
impl FinalizedBatchStore {
    fn new(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|err| {
            format!(
                "failed to create compatible indexer finalized batch directory {}: {err}",
                dir.display(),
            )
        })?;

        Ok(Self { dir: Arc::new(dir) })
    }

    fn persist(&self, batch: &FinalizedBatch) -> Result<QueuedBatch, String> {
        let queued = queue_batch(batch);
        let path = self.path(queued.number, queued.hash);
        write_durable(&path, &codec::Encode::encode(batch), "finalized batch")?;
        Ok(queued)
    }

    fn load(&self, number: u32, hash: H256) -> Result<Option<Vec<u8>>, String> {
        let path = self.path(number, hash);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(format!(
                    "failed to read compatible indexer finalized batch {}: {err}",
                    path.display(),
                ));
            }
        };
        let mut input = bytes.as_slice();
        let batch = <FinalizedBatch as codec::Decode>::decode(&mut input).map_err(|err| {
            format!(
                "compatible indexer finalized batch {} is corrupted or uses an unsupported format: {err}",
                path.display(),
            )
        })?;
        if !input.is_empty() {
            return Err(format!(
                "compatible indexer finalized batch {} contains {} unexpected trailing bytes",
                path.display(),
                input.len(),
            ));
        }
        if batch.block.number != number || H256::from(batch.block.hash) != hash {
            return Err(format!(
                "compatible indexer finalized batch {} does not match requested block_number={} block_hash={hash:?}",
                path.display(),
                number,
            ));
        }
        Ok(Some(bytes))
    }

    fn remove(&self, number: u32, hash: H256) {
        let path = self.path(number, hash);
        if let Err(err) = remove_file_if_exists(&path, "finalized batch") {
            warn!(
                "Could not remove compatible indexer finalized batch {}: {err}",
                path.display(),
            );
        }
    }

    fn try_remove(&self, number: u32, hash: H256) -> Result<(), String> {
        remove_file_if_exists(&self.path(number, hash), "finalized batch")
    }

    fn remove_up_to(&self, number: u32) -> Result<(), String> {
        remove_files_up_to(self.dir.as_ref(), number, "finalized batch")
    }

    fn path(&self, number: u32, hash: H256) -> PathBuf {
        self.dir.join(format!(
            "{number:010}-{}.scale",
            hex::encode(hash.as_bytes())
        ))
    }
}

impl SegmentedBatchJournal {
    pub(crate) fn new(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|err| {
            format!(
                "failed to create compatible indexer segmented journal directory {}: {err}",
                dir.display()
            )
        })?;

        let mut segment_ids = fs::read_dir(&dir)
            .map_err(|err| format!("failed to scan journal directory {}: {err}", dir.display()))?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.strip_prefix("segment-"))
                    .and_then(|name| name.strip_suffix(".log"))
                    .and_then(|id| id.parse::<u64>().ok())
            })
            .collect::<Vec<_>>();
        segment_ids.sort_unstable();
        segment_ids.dedup();
        let created_initial_segment = segment_ids.is_empty();
        if created_initial_segment {
            segment_ids.push(0);
        }

        let mut segments = BTreeMap::new();
        let mut index = BTreeMap::new();
        for segment_id in segment_ids.iter().copied() {
            let path = journal_segment_path(&dir, segment_id);
            let mut file = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|err| {
                    format!("failed to open journal segment {}: {err}", path.display())
                })?;
            let is_last = segment_id == *segment_ids.last().expect("non-empty; qed");
            let (records, locations, size) =
                recover_journal_segment(&mut file, segment_id, &path, is_last)?;
            file.sync_data().map_err(|err| {
                format!(
                    "failed to stabilize recovered journal segment {}: {err}",
                    path.display()
                )
            })?;
            for (key, location) in locations {
                index.insert(key, location);
            }
            segments.insert(
                segment_id,
                JournalSegment {
                    records,
                    file,
                    size,
                },
            );
        }
        if created_initial_segment {
            fs::File::open(&dir)
                .and_then(|dir| dir.sync_all())
                .map_err(|err| {
                    format!(
                        "failed to sync new journal directory {}: {err}",
                        dir.display()
                    )
                })?;
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(JournalInner {
                dir,
                segments,
                index,
                current_segment_id: *segment_ids.last().expect("non-empty; qed"),
                unsynced_records: 0,
                acknowledged_up_to: None,
                acknowledgements_since_gc: 0,
                stale: BTreeSet::new(),
                cleanup_directory_dirty: false,
            })),
        })
    }

    fn append(&self, batch: &FinalizedBatch) -> Result<QueuedBatch, String> {
        let queued = queue_batch(batch);
        let key = (queued.number, queued.hash);
        let payload = codec::Encode::encode(batch);
        let payload_len = u32::try_from(payload.len())
            .map_err(|_| "compatible indexer journal record exceeds u32::MAX bytes".to_string())?;
        let record_len = JOURNAL_RECORD_HEADER_BYTES + u64::from(payload_len);
        let mut inner = self.inner.lock();
        if inner.index.contains_key(&key) {
            inner.stale.remove(&key);
            if inner.unsynced_records >= GROUP_COMMIT_BLOCKS {
                inner
                    .segments
                    .get(&inner.current_segment_id)
                    .expect("current journal segment exists; qed")
                    .file
                    .sync_data()
                    .map_err(|err| format!("failed to retry journal group commit: {err}"))?;
                inner.unsynced_records = 0;
            }
            return Ok(queued);
        }

        let current_size = inner
            .segments
            .get(&inner.current_segment_id)
            .expect("current journal segment exists; qed")
            .size;
        if current_size > 0 && current_size + record_len > JOURNAL_SEGMENT_MAX_BYTES {
            rotate_journal_segment(&mut inner)?;
        }

        let segment_id = inner.current_segment_id;
        let segment = inner
            .segments
            .get_mut(&segment_id)
            .expect("current journal segment exists; qed");
        let mut record = Vec::with_capacity(record_len as usize);
        record.extend_from_slice(&JOURNAL_RECORD_MAGIC);
        record.extend_from_slice(&payload_len.to_le_bytes());
        record.extend_from_slice(&sp_core::blake2_256(&payload));
        record.extend_from_slice(&payload);
        if let Err(err) = segment
            .file
            .seek(SeekFrom::Start(segment.size))
            .and_then(|_| segment.file.write_all(&record))
        {
            let _ = segment.file.set_len(segment.size);
            return Err(format!(
                "failed to append compatible indexer block_number={} to journal segment {}: {err}",
                queued.number, segment_id
            ));
        }
        let location = JournalLocation {
            segment_id,
            payload_offset: segment.size + JOURNAL_RECORD_HEADER_BYTES,
            payload_len,
        };
        segment.size += record_len;
        segment.records.push(key);
        inner.index.insert(key, location);
        inner.unsynced_records += 1;
        if inner.unsynced_records >= GROUP_COMMIT_BLOCKS {
            inner
                .segments
                .get(&segment_id)
                .expect("current journal segment exists; qed")
                .file
                .sync_data()
                .map_err(|err| format!("failed to group-commit journal segment: {err}"))?;
            inner.unsynced_records = 0;
        }
        Ok(queued)
    }

    fn load(&self, number: u32, hash: H256) -> Result<Option<Vec<u8>>, String> {
        let mut inner = self.inner.lock();
        let Some(location) = inner.index.get(&(number, hash)).copied() else {
            return Ok(None);
        };
        let segment = inner
            .segments
            .get_mut(&location.segment_id)
            .ok_or_else(|| {
                format!(
                    "journal index references missing segment {}",
                    location.segment_id
                )
            })?;
        let mut payload = vec![0; location.payload_len as usize];
        segment
            .file
            .seek(SeekFrom::Start(location.payload_offset))
            .and_then(|_| segment.file.read_exact(&mut payload))
            .map_err(|err| {
                format!(
                    "failed to read compatible indexer journal record for block_number={number} block_hash={hash:?}: {err}"
                )
            })?;
        Ok(Some(payload))
    }

    fn mark_stale(&self, number: u32, hash: H256) -> Result<(), String> {
        let mut inner = self.inner.lock();
        inner.stale.insert((number, hash));
        gc_journal_segments(&mut inner)
    }

    fn keys(&self) -> Vec<(u32, H256)> {
        self.inner.lock().index.keys().copied().collect()
    }

    fn mark_stale_many(&self, stale: impl IntoIterator<Item = (u32, H256)>) -> Result<(), String> {
        let mut inner = self.inner.lock();
        inner.stale.extend(stale);
        gc_journal_segments(&mut inner)
    }

    fn acknowledge_up_to(&self, number: u32) -> Result<(), String> {
        let mut inner = self.inner.lock();
        inner.acknowledged_up_to = Some(
            inner
                .acknowledged_up_to
                .map_or(number, |current| current.max(number)),
        );
        inner.acknowledgements_since_gc += 1;
        if inner.acknowledgements_since_gc < GROUP_COMMIT_BLOCKS {
            return Ok(());
        }
        inner.acknowledgements_since_gc = 0;
        gc_journal_segments(&mut inner)
    }

    fn acknowledge_cursor(&self, number: u32) -> Result<(), String> {
        let mut inner = self.inner.lock();
        inner.acknowledged_up_to = Some(
            inner
                .acknowledged_up_to
                .map_or(number, |current| current.max(number)),
        );
        inner.acknowledgements_since_gc = 0;
        gc_journal_segments(&mut inner)
    }

    fn retry_gc(&self) -> Result<(), String> {
        gc_journal_segments(&mut self.inner.lock())
    }

    fn sync_current(&self) -> Result<(), String> {
        let mut inner = self.inner.lock();
        inner
            .segments
            .get(&inner.current_segment_id)
            .expect("current journal segment exists; qed")
            .file
            .sync_data()
            .map_err(|err| format!("failed to sync current journal segment: {err}"))?;
        inner.unsynced_records = 0;
        Ok(())
    }
}

fn journal_segment_path(dir: &std::path::Path, id: u64) -> PathBuf {
    dir.join(format!("segment-{id:010}.log"))
}

fn recover_journal_segment(
    file: &mut fs::File,
    segment_id: u64,
    path: &std::path::Path,
    is_last: bool,
) -> Result<RecoveredJournalSegment, String> {
    let file_len = file
        .metadata()
        .map_err(|err| format!("failed to stat journal segment {}: {err}", path.display()))?
        .len();
    let mut offset = 0u64;
    let mut records = Vec::new();
    let mut locations = Vec::new();
    while offset < file_len {
        if file_len - offset < JOURNAL_RECORD_HEADER_BYTES {
            file.set_len(offset)
                .map_err(|err| format!("failed to truncate incomplete journal tail: {err}"))?;
            break;
        }
        file.seek(SeekFrom::Start(offset))
            .map_err(|err| err.to_string())?;
        let mut magic = [0; 4];
        let mut len = [0; 4];
        let mut checksum = [0; 32];
        file.read_exact(&mut magic).map_err(|err| err.to_string())?;
        file.read_exact(&mut len).map_err(|err| err.to_string())?;
        file.read_exact(&mut checksum)
            .map_err(|err| err.to_string())?;
        if magic != JOURNAL_RECORD_MAGIC {
            if is_last {
                file.set_len(offset)
                    .map_err(|err| format!("failed to truncate invalid journal tail: {err}"))?;
                break;
            }
            return Err(format!(
                "journal segment {} has invalid record magic at offset {offset}",
                path.display()
            ));
        }
        let payload_len = u32::from_le_bytes(len);
        let record_end = offset + JOURNAL_RECORD_HEADER_BYTES + u64::from(payload_len);
        if record_end > file_len {
            file.set_len(offset)
                .map_err(|err| format!("failed to truncate incomplete journal payload: {err}"))?;
            break;
        }
        let mut payload = vec![0; payload_len as usize];
        file.read_exact(&mut payload)
            .map_err(|err| err.to_string())?;
        if sp_core::blake2_256(&payload) != checksum {
            if is_last {
                file.set_len(offset)
                    .map_err(|err| format!("failed to truncate corrupt journal tail: {err}"))?;
                break;
            }
            return Err(format!(
                "journal segment {} has a checksum mismatch at offset {offset}",
                path.display()
            ));
        }
        let mut input = payload.as_slice();
        let batch = <FinalizedBatch as codec::Decode>::decode(&mut input).map_err(|err| {
            format!(
                "cannot decode journal record at {}:{offset}: {err}",
                path.display()
            )
        })?;
        if !input.is_empty() {
            return Err(format!(
                "journal record at {}:{offset} has trailing bytes",
                path.display()
            ));
        }
        let key = (batch.block.number, H256::from(batch.block.hash));
        records.push(key);
        locations.push((
            key,
            JournalLocation {
                segment_id,
                payload_offset: offset + JOURNAL_RECORD_HEADER_BYTES,
                payload_len,
            },
        ));
        offset = record_end;
    }
    Ok((records, locations, offset))
}

fn rotate_journal_segment(inner: &mut JournalInner) -> Result<(), String> {
    inner
        .segments
        .get(&inner.current_segment_id)
        .expect("current journal segment exists; qed")
        .file
        .sync_data()
        .map_err(|err| format!("failed to sync completed journal segment: {err}"))?;
    inner.unsynced_records = 0;
    inner.current_segment_id += 1;
    let path = journal_segment_path(&inner.dir, inner.current_segment_id);
    let file = fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|err| format!("failed to create journal segment {}: {err}", path.display()))?;
    fs::File::open(&inner.dir)
        .and_then(|dir| dir.sync_all())
        .map_err(|err| {
            format!(
                "failed to sync journal directory {}: {err}",
                inner.dir.display()
            )
        })?;
    inner.segments.insert(
        inner.current_segment_id,
        JournalSegment {
            records: Vec::new(),
            file,
            size: 0,
        },
    );
    if let Err(err) = gc_journal_segments(inner) {
        error!(
            "COMPATIBLE INDEXER JOURNAL SEGMENT CLEANUP FAILED during rotation; indexing continues and cleanup will retry in the background: {err}"
        );
    }
    Ok(())
}

fn gc_journal_segments(inner: &mut JournalInner) -> Result<(), String> {
    let acknowledged_up_to = inner.acknowledged_up_to;
    let removable = inner
        .segments
        .iter()
        .filter(|(id, segment)| {
            **id != inner.current_segment_id
                && segment.records.iter().all(|key| {
                    acknowledged_up_to.is_some_and(|number| key.0 <= number)
                        || inner.stale.contains(key)
                })
        })
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut removed_any = false;
    for id in removable {
        let path = journal_segment_path(&inner.dir, id);
        match fs::remove_file(&path) {
            Ok(()) => {
                removed_any = true;
                if let Some(segment) = inner.segments.remove(&id) {
                    for key in segment.records {
                        inner.index.remove(&key);
                        inner.stale.remove(&key);
                    }
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                removed_any = true;
                if let Some(segment) = inner.segments.remove(&id) {
                    for key in segment.records {
                        inner.index.remove(&key);
                        inner.stale.remove(&key);
                    }
                }
            }
            Err(err) => errors.push(format!(
                "failed to remove journal segment {}: {err}",
                path.display()
            )),
        }
    }
    if removed_any {
        inner.cleanup_directory_dirty = true;
    }
    if inner.cleanup_directory_dirty {
        match fs::File::open(&inner.dir).and_then(|dir| dir.sync_all()) {
            Ok(()) => inner.cleanup_directory_dirty = false,
            Err(err) => errors.push(format!(
                "failed to sync journal directory {} after segment cleanup: {err}",
                inner.dir.display()
            )),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[allow(dead_code)]
fn write_durable(path: &std::path::Path, bytes: &[u8], kind: &str) -> Result<(), String> {
    let tmp_path = path.with_extension("scale.tmp");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&tmp_path)
        .map_err(|err| {
            format!(
                "failed to open compatible indexer {kind} {}: {err}",
                tmp_path.display()
            )
        })?;
    file.write_all(bytes).map_err(|err| {
        format!(
            "failed to write compatible indexer {kind} {}: {err}",
            tmp_path.display()
        )
    })?;
    file.sync_all().map_err(|err| {
        format!(
            "failed to sync compatible indexer {kind} {}: {err}",
            tmp_path.display()
        )
    })?;
    drop(file);
    fs::rename(&tmp_path, path).map_err(|err| {
        format!(
            "failed to commit compatible indexer {kind} {}: {err}",
            path.display()
        )
    })?;
    let parent = path.parent().ok_or_else(|| {
        format!(
            "compatible indexer {kind} path has no parent: {}",
            path.display()
        )
    })?;
    fs::File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|err| {
            format!(
                "failed to sync compatible indexer {kind} directory {}: {err}",
                parent.display()
            )
        })
}

#[allow(dead_code)]
fn remove_file_if_exists(path: &std::path::Path, kind: &str) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!(
            "failed to remove compatible indexer {kind} {}: {err}",
            path.display(),
        )),
    }
}

#[allow(dead_code)]
fn remove_files_up_to(dir: &std::path::Path, number: u32, kind: &str) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| {
        format!(
            "failed to scan compatible indexer {kind} directory {} for acknowledged files: {err}",
            dir.display(),
        )
    })?;
    let mut failures = Vec::new();

    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(err) => {
                failures.push(format!(
                    "failed to read an entry from compatible indexer {kind} directory {}: {err}",
                    dir.display(),
                ));
                continue;
            }
        };
        let Some(file_number) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.split_once('-'))
            .and_then(|(number, _)| number.parse::<u32>().ok())
        else {
            continue;
        };
        if file_number <= number
            && let Err(err) = remove_file_if_exists(&path, kind)
        {
            failures.push(err);
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn acknowledge_or_schedule(
    cleanup_scheduler: &CleanupScheduler,
    journal: &SegmentedBatchJournal,
    number: u32,
    force_gc: bool,
) {
    let result = if force_gc {
        journal.acknowledge_cursor(number)
    } else {
        journal.acknowledge_up_to(number)
    };
    if let Err(err) = result {
        error!(
            "Compatible indexer acknowledged batches through block_number={number}, but an eligible journal segment could not be removed; delivery continues and cleanup will be retried in the background: {err}"
        );
        cleanup_scheduler.schedule_up_to(number);
    }
}

impl CleanupScheduler {
    fn schedule_up_to(&self, number: u32) {
        let mut pending = self.pending.lock();
        pending.up_to = Some(pending.up_to.map_or(number, |current| current.max(number)));
    }

    async fn run(self, journal: SegmentedBatchJournal) {
        loop {
            async_io::Timer::after(CLEANUP_RETRY).await;
            let up_to = self.pending.lock().up_to;
            match journal.retry_gc() {
                Ok(()) => {
                    if let Some(number) = up_to {
                        let mut pending = self.pending.lock();
                        if pending.up_to == Some(number) {
                            pending.up_to = None;
                        }
                    }
                }
                Err(err) => error!(
                    "COMPATIBLE INDEXER JOURNAL SEGMENT CLEANUP FAILED after acknowledgement through block_number={up_to:?}; retrying in {CLEANUP_RETRY:?}: {err}"
                ),
            }
        }
    }
}

impl<RuntimeApi, Executor, Inner> IndexerBlockImport<RuntimeApi, Executor, Inner>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    pub(super) fn new(
        inner: Inner,
        client: Arc<FullClient<RuntimeApi, Executor>>,
        journal: Option<SegmentedBatchJournal>,
        chain_id: String,
        genesis_hash: H256,
    ) -> Result<Self, String> {
        Ok(Self {
            inner,
            client,
            journal,
            chain_id,
            genesis_hash,
        })
    }

    fn capture_storage_diff(
        &self,
        _number: u32,
        _hash: H256,
        parent_hash: H256,
        changes: &sp_state_machine::StorageCollection,
    ) -> Result<Vec<StorageChange>, sp_consensus::Error> {
        let Some(_journal) = &self.journal else {
            return Ok(Vec::new());
        };
        let storage_changes = changes
            .iter()
            .map(|(key, value)| {
                storage_change(
                    &self.client,
                    parent_hash,
                    ObservedStorageChange {
                        raw_key: StorageKey(key.clone()),
                        new_raw_value: value.clone().map(StorageData),
                    },
                )
            })
            .collect::<Vec<_>>();
        Ok(storage_changes)
    }
}

#[async_trait::async_trait]
impl<RuntimeApi, Executor, Inner> BlockImport<Block>
    for IndexerBlockImport<RuntimeApi, Executor, Inner>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
    Inner: BlockImport<Block, Error = sp_consensus::Error> + Send + Sync,
{
    type Error = sp_consensus::Error;

    async fn check_block(
        &self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(
        &self,
        mut block: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        let Some(journal) = &self.journal else {
            return self.inner.import_block(block).await;
        };
        let hash = block.post_hash();
        let number = *block.header.number();
        let parent_hash = *block.header.parent_hash();

        let storage_changes = match &block.state_action {
            StateAction::ApplyChanges(ImportStorageChanges::Changes(changes)) => {
                self.capture_storage_diff(number, hash, parent_hash, &changes.main_storage_changes)?
            }
            StateAction::ApplyChanges(ImportStorageChanges::Import(_)) => {
                return Err(sp_consensus::Error::ClientImport(format!(
                    "compatible indexer requires block-by-block sync from genesis; state import is not allowed at block_number={number} block_hash={hash:?}"
                )));
            }
            StateAction::Execute | StateAction::ExecuteIfPossible => {
                let body = block.body.clone().ok_or_else(|| {
                    sp_consensus::Error::ClientImport(format!(
                        "compatible indexer cannot capture storage changes without a body at block_number={number} block_hash={hash:?}"
                    ))
                })?;
                let mut runtime_api = self.client.runtime_api();
                runtime_api.set_call_context(CallContext::Onchain);
                runtime_api
                    .execute_block(parent_hash, Block::new(block.header.clone(), body).into())
                    .map_err(|err| {
                        sp_consensus::Error::ClientImport(format!(
                            "compatible indexer cannot execute block_number={number} block_hash={hash:?} to capture storage changes: {err}"
                        ))
                    })?;
                let state = self.client.state_at(parent_hash).map_err(|err| {
                    sp_consensus::Error::ClientImport(format!(
                        "compatible indexer cannot read parent state for block_number={number} block_hash={hash:?}: {err}"
                    ))
                })?;
                let changes = runtime_api
                    .into_storage_changes(&state, parent_hash)
                    .map_err(|err| {
                        sp_consensus::Error::ClientImport(format!(
                            "compatible indexer cannot extract storage changes for block_number={number} block_hash={hash:?}: {err}"
                        ))
                    })?;
                if block.header.state_root() != &changes.transaction_storage_root {
                    return Err(sp_consensus::Error::ClientImport(format!(
                        "compatible indexer execution produced an invalid state root for block_number={number} block_hash={hash:?}"
                    )));
                }
                let storage_changes = self.capture_storage_diff(
                    number,
                    hash,
                    parent_hash,
                    &changes.main_storage_changes,
                )?;
                block.state_action =
                    StateAction::ApplyChanges(ImportStorageChanges::Changes(changes));
                storage_changes
            }
            StateAction::Skip => {
                return Err(sp_consensus::Error::ClientImport(format!(
                    "compatible indexer requires executed blocks; skipped state transition at block_number={number} block_hash={hash:?}"
                )));
            }
        };

        let body = block.body.clone().ok_or_else(|| {
            sp_consensus::Error::ClientImport(format!(
                "compatible indexer cannot persist a complete batch without a body at block_number={number} block_hash={hash:?}"
            ))
        })?;
        let batch = build_batch_from_parts(
            &self.client,
            &self.chain_id,
            self.genesis_hash,
            block.post_header(),
            body,
            storage_changes,
        )
        .map_err(|err| sp_consensus::Error::ClientImport(err.to_string()))?;
        journal.append(&batch).map_err(|err| {
            sp_consensus::Error::ClientImport(format!(
                "cannot append compatible indexer batch for block_number={number} block_hash={hash:?}: {err}"
            ))
        })?;

        let result = self.inner.import_block(block).await;
        if !matches!(
            result,
            Ok(ImportResult::Imported(_)) | Ok(ImportResult::AlreadyInChain)
        ) {
            let _ = journal.mark_stale(number, hash);
        }
        result
    }
}

pub fn spawn<RuntimeApi, Executor>(
    spawn_handle: SpawnTaskHandle,
    client: Arc<FullClient<RuntimeApi, Executor>>,
    sync_service: Arc<sc_network_sync::SyncingService<Block>>,
    config: BatchSinkConfig,
    journal: SegmentedBatchJournal,
) -> Result<(), ServiceError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let BatchSinkConfig {
        requires_network_sync_target,
        base_url,
        token,
        max_queue_len,
        chain_id,
        genesis_hash,
    } = config;
    recover_journal_tail(&client, &journal, &chain_id, genesis_hash).map_err(ServiceError::from)?;
    let finality_stream = client.finality_notification_stream();
    let cleanup_scheduler = CleanupScheduler::default();
    let best_delivery_ready = Arc::new(AtomicBool::new(false));
    let (mut batch_tx, batch_rx) = mpsc::channel(max_queue_len.max(1));

    let producer = {
        let client = client.clone();
        let journal = journal.clone();
        async move {
            produce_batches(client, finality_stream, &mut batch_tx, journal).await;
        }
    };

    let finalized_base_url = base_url.clone();
    let finalized_token = token.clone();
    let finalized_chain_id = chain_id.clone();
    let sender = {
        let client = client.clone();
        let journal = journal.clone();
        let cleanup_scheduler = cleanup_scheduler.clone();
        let best_delivery_ready = best_delivery_ready.clone();
        async move {
            let sink = HttpSink::new(
                finalized_base_url,
                finalized_token,
                finalized_chain_id,
                genesis_hash,
            );
            send_batches(
                BatchSender {
                    client,
                    sync_service,
                    requires_network_sync_target,
                    sink,
                    max_queue_len,
                    journal,
                    cleanup_scheduler,
                },
                batch_rx,
                best_delivery_ready,
            )
            .await;
        }
    };

    let cleanup = cleanup_scheduler.run(journal.clone());

    let best_sender = {
        let client = client.clone();
        let journal = journal.clone();
        let best_delivery_ready = best_delivery_ready.clone();
        let sink = HttpSink::new(
            base_url.clone(),
            token.clone(),
            chain_id.clone(),
            genesis_hash,
        );
        async move {
            send_best_chain(client, sink, journal, best_delivery_ready).await;
        }
    };

    spawn_handle.spawn(
        "indexer-finalized-batch-producer",
        Some("indexer-batch-sink"),
        producer,
    );
    spawn_handle.spawn(
        "indexer-finalized-batch-sender",
        Some("indexer-batch-sink"),
        sender,
    );
    spawn_handle.spawn(
        "indexer-acknowledged-data-cleanup",
        Some("indexer-batch-sink"),
        cleanup,
    );
    spawn_handle.spawn(
        "indexer-best-chain-sender",
        Some("indexer-batch-sink"),
        best_sender,
    );

    Ok(())
}

async fn send_best_chain<RuntimeApi, Executor>(
    client: Arc<FullClient<RuntimeApi, Executor>>,
    sink: HttpSink,
    journal: SegmentedBatchJournal,
    live_delivery_ready: Arc<AtomicBool>,
) where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let mut acknowledged = None;
    let mut retry_delay = INITIAL_BATCH_RETRY;

    loop {
        if !live_delivery_ready.load(Ordering::Acquire) {
            async_io::Timer::after(Duration::from_millis(250)).await;
            continue;
        }
        if acknowledged.is_none() {
            match sink.open_stream().await.and_then(|ack| {
                resolve_best_cursor(&client, ack.best.or(ack.last_received), sink.genesis_hash)
            }) {
                Ok(cursor) => acknowledged = Some(cursor),
                Err(err) => {
                    error!(
                        "COMPATIBLE INDEXER BEST CHAIN STREAM IS NOT OPEN: {err}; retrying in {retry_delay:?}"
                    );
                    async_io::Timer::after(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                    continue;
                }
            }
        }

        let from = acknowledged.expect("best cursor was established above; qed");
        let info = client.info();
        let to = BestBlockRef {
            number: info.best_number,
            hash: h256_to_bytes(info.best_hash),
        };
        if from.hash != to.hash {
            let delivery = match build_best_chain_update(&client, &journal, &sink, from, to) {
                Ok(update) => sink.send_best_chain(update).await,
                Err(err) => Err(err),
            };
            match delivery {
                Ok(()) => {
                    acknowledged = Some(to);
                    retry_delay = INITIAL_BATCH_RETRY;
                    continue;
                }
                Err(err) => {
                    error!(
                        "Compatible indexer best-chain update was not acknowledged; stream-start will reconcile it after {retry_delay:?}: {err}"
                    );
                    acknowledged = None;
                    async_io::Timer::after(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                    continue;
                }
            }
        }

        // Polling also covers import-notification races during block commit and
        // nodes whose manual-seal import is finalized in the same operation.
        // No HTTP request is made while the head is unchanged.
        async_io::Timer::after(Duration::from_millis(250)).await;
    }
}

fn resolve_best_cursor<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    cursor: Option<BlockCursor>,
    genesis_hash: H256,
) -> Result<BestBlockRef, String>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let Some(cursor) = cursor else {
        return Ok(BestBlockRef {
            number: 0,
            hash: h256_to_bytes(genesis_hash),
        });
    };
    let hash = match cursor.hash {
        Some(hash) => hash,
        None => client
            .block_hash(cursor.number)
            .map_err(|err| format!("cannot resolve indexer BEST cursor: {err}"))?
            .ok_or_else(|| {
                format!(
                    "cannot resolve indexer BEST cursor at block_number={}",
                    cursor.number
                )
            })?,
    };
    let header = client
        .header(hash)
        .map_err(|err| format!("cannot read indexer BEST cursor header: {err}"))?
        .ok_or_else(|| format!("indexer BEST cursor hash is unknown locally: {hash:?}"))?;
    if *header.number() != cursor.number {
        return Err(format!(
            "indexer BEST cursor number/hash mismatch: reported={} local={}",
            cursor.number,
            header.number()
        ));
    }
    Ok(BestBlockRef {
        number: cursor.number,
        hash: h256_to_bytes(hash),
    })
}

fn build_best_chain_update<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    journal: &SegmentedBatchJournal,
    sink: &HttpSink,
    from: BestBlockRef,
    to: BestBlockRef,
) -> Result<BestChainUpdate, String>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let route = sp_blockchain::tree_route(&**client, H256::from(from.hash), H256::from(to.hash))
        .map_err(|err| format!("cannot calculate BEST chain route: {err}"))?;
    let finalized_number = client.info().finalized_number;
    if route
        .retracted()
        .iter()
        .any(|block| block.number <= finalized_number)
    {
        return Err(format!(
            "indexer BEST cursor conflicts with locally finalized chain at block_number={finalized_number}"
        ));
    }
    let retracted = route
        .retracted()
        .iter()
        .map(|block| BestBlockRef {
            number: block.number,
            hash: h256_to_bytes(block.hash),
        })
        .collect();
    let enacted_batches = route
        .enacted()
        .iter()
        .map(|block| {
            journal
                .load(block.number, block.hash)?
                .ok_or_else(|| {
                    format!(
                        "BEST batch is unavailable in the durable journal at block_number={} block_hash={:?}",
                        block.number, block.hash
                    )
                })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(BestChainUpdate {
        fixture_format_version: FIXTURE_FORMAT_VERSION,
        chain_id: sink.chain_id.clone(),
        genesis_hash: h256_to_bytes(sink.genesis_hash),
        from,
        to,
        retracted,
        enacted_batches,
    })
}

fn recover_journal_tail<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    journal: &SegmentedBatchJournal,
    chain_id: &str,
    genesis_hash: H256,
) -> Result<(), String>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let best = client.info().best_number;
    let finalized = client.info().finalized_number;
    let stale = journal
        .keys()
        .into_iter()
        .filter(|(number, hash)| {
            *number <= finalized
                && client
                    .block_hash(*number)
                    .ok()
                    .flatten()
                    .is_some_and(|canonical| canonical != *hash)
        })
        .collect::<Vec<_>>();
    journal.mark_stale_many(stale)?;
    let from = best.saturating_sub(GROUP_COMMIT_BLOCKS.saturating_sub(1));
    let mut recovered = 0u32;
    for number in from..=best {
        let Some(hash) = client
            .block_hash(number)
            .map_err(|err| format!("failed to inspect recoverable block #{number}: {err}"))?
        else {
            continue;
        };
        if journal.load(number, hash)?.is_some() {
            continue;
        }
        let batch = build_historical_batch(client, chain_id, genesis_hash, number)
            .map_err(|err| format!("cannot recover group-commit tail at block #{number}: {err}"))?;
        journal.append(&batch)?;
        recovered += 1;
    }
    journal.sync_current()?;
    if recovered > 0 {
        info!(
            "Recovered {recovered} compatible indexer journal batches from retained Substrate blocks and parent states; block_number_range={from}..={best}"
        );
    }
    Ok(())
}

fn ensure_genesis_batch<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    journal: &SegmentedBatchJournal,
    chain_id: &str,
    genesis_hash: H256,
) -> Result<(), String>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    if journal.load(0, genesis_hash)?.is_some() {
        return Ok(());
    }

    let storage_changes = client
        .storage_pairs(genesis_hash, None, None)
        .map_err(|err| {
            format!(
                "cannot build the compatible indexer genesis batch from top-level storage; start from a database that still has genesis state: {err}"
            )
        })?
        .map(|(key, value)| StorageChange {
            raw_key: key.0,
            old_raw_value: None,
            new_raw_value: Some(value.0),
            operation: StorageOperation::Upsert,
        })
        .collect::<Vec<_>>();
    let batch = build_live_batch(
        client,
        chain_id,
        genesis_hash,
        genesis_hash,
        &storage_changes,
    )
    .map_err(|err| format!("cannot build compatible indexer genesis batch: {err}"))?;
    journal.append(&batch)?;
    info!(
        "Persisted compatible indexer genesis batch with {} top-level storage entries",
        storage_changes.len(),
    );
    Ok(())
}

async fn produce_batches<RuntimeApi, Executor>(
    client: Arc<FullClient<RuntimeApi, Executor>>,
    mut finality_stream: sc_client_api::FinalityNotifications<Block>,
    batch_tx: &mut mpsc::Sender<QueuedBatch>,
    journal: SegmentedBatchJournal,
) where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let mut pending_finalized = VecDeque::new();

    while let Some(finality) = finality_stream.next().await {
        for hash in finality
            .tree_route
            .iter()
            .copied()
            .chain(std::iter::once(finality.hash))
        {
            pending_finalized.push_back(hash);
        }
        for stale_hash in finality
            .stale_blocks
            .iter()
            .filter(|block| block.is_head)
            .map(|block| block.hash)
        {
            remove_stale_branch(&client, stale_hash, &journal);
        }

        if !flush_ready_finalized(&client, &mut pending_finalized, batch_tx, &journal) {
            return;
        }
    }

    error!(
        "Compatible indexer finalized batch producer stopped: finality notification stream ended"
    );
}

fn remove_stale_branch<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    stale_head: H256,
    journal: &SegmentedBatchJournal,
) where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let mut hash = stale_head;

    loop {
        let header = match client.header(hash) {
            Ok(Some(header)) => header,
            Ok(None) => {
                warn!(
                    "Could not walk stale compatible indexer branch from block_hash={hash:?}: header is unavailable"
                );
                return;
            }
            Err(err) => {
                warn!(
                    "Could not walk stale compatible indexer branch from block_hash={hash:?}: {err}"
                );
                return;
            }
        };
        let number = *header.number();

        match client.block_hash(number) {
            Ok(Some(canonical_hash)) if canonical_hash == hash => return,
            Ok(_) => {}
            Err(err) => {
                warn!(
                    "Could not identify canonical block while pruning stale compatible indexer branch at block_number={number} block_hash={hash:?}: {err}"
                );
                return;
            }
        }

        if let Err(err) = journal.mark_stale(number, hash) {
            warn!("Could not garbage-collect stale compatible indexer journal data: {err}");
        }
        if number == 0 {
            return;
        }
        hash = *header.parent_hash();
    }
}

fn flush_ready_finalized<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    pending_finalized: &mut VecDeque<H256>,
    batch_tx: &mut mpsc::Sender<QueuedBatch>,
    _journal: &SegmentedBatchJournal,
) -> bool
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    while let Some(hash) = pending_finalized.front().copied() {
        let number = match client.header(hash) {
            Ok(Some(header)) => *header.number(),
            Ok(None) => {
                error!(
                    "Compatible indexer cannot read finalized header for block_hash={hash:?}; producer stops to keep memory bounded"
                );
                return false;
            }
            Err(err) => {
                error!(
                    "Compatible indexer cannot read finalized header for block_hash={hash:?}: {err}"
                );
                return false;
            }
        };
        pending_finalized.pop_front();
        if let Err(err) = batch_tx.try_send(QueuedBatch { number, hash }) {
            if err.is_full() {
                debug!(
                    "Compatible indexer sender notification channel is full; finalized batch remains discoverable from finalized chain continuity"
                );
            } else {
                error!(
                    "Compatible indexer finalized batch producer stopped: sender task is not available"
                );
                return false;
            }
        }
    }

    true
}

fn build_live_batch<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    chain_id: &str,
    genesis_hash: H256,
    block_hash: H256,
    storage_changes: &[StorageChange],
) -> Result<FinalizedBatch, BatchBuildError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let header = client
        .header(block_hash)
        .map_err(|err| BatchBuildError::Unavailable {
            hash: block_hash,
            detail: format!("failed to read block header: {err}"),
        })?
        .ok_or_else(|| BatchBuildError::Unavailable {
            hash: block_hash,
            detail: "block header is missing".to_string(),
        })?;
    let body = client
        .block_body(block_hash)
        .map_err(|err| BatchBuildError::Unavailable {
            hash: block_hash,
            detail: format!("failed to read block body: {err}"),
        })?
        .ok_or_else(|| BatchBuildError::Unavailable {
            hash: block_hash,
            detail: "block body is missing".to_string(),
        })?;
    build_batch_from_parts(
        client,
        chain_id,
        genesis_hash,
        header,
        body,
        storage_changes.to_vec(),
    )
}

fn build_batch_from_parts<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    chain_id: &str,
    genesis_hash: H256,
    header: <Block as BlockT>::Header,
    body: Vec<<Block as BlockT>::Extrinsic>,
    storage_changes: Vec<StorageChange>,
) -> Result<FinalizedBatch, BatchBuildError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let block_hash = header.hash();
    // Extrinsics in this block were decoded and executed by the runtime in the parent state.
    // The block state may already contain a runtime upgrade performed by one of those extrinsics.
    let runtime_at = if *header.number() == 0 {
        block_hash
    } else {
        *header.parent_hash()
    };
    let runtime = client
        .runtime_version_at(runtime_at, CallContext::Onchain)
        .map_err(|err| BatchBuildError::Unavailable {
            hash: block_hash,
            detail: format!("failed to read block execution runtime version: {err}"),
        })?;

    let header_raw_bytes = codec::Encode::encode(&header);

    Ok(FinalizedBatch {
        fixture_format_version: FIXTURE_FORMAT_VERSION,
        chain_id: chain_id.to_owned(),
        genesis_hash: h256_to_bytes(genesis_hash),
        runtime: RuntimeIdentity {
            spec_name: runtime.spec_name.to_string(),
            spec_version: runtime.spec_version,
            transaction_version: runtime.transaction_version,
        },
        block: BlockData {
            number: *header.number(),
            hash: h256_to_bytes(block_hash),
            parent_hash: h256_to_bytes(*header.parent_hash()),
            state_root: h256_to_bytes(*header.state_root()),
            extrinsics_root: h256_to_bytes(*header.extrinsics_root()),
            header_raw_bytes: Some(header_raw_bytes),
        },
        extrinsics: body
            .into_iter()
            .enumerate()
            .map(|(index, extrinsic)| {
                let raw_bytes = codec::Encode::encode(&extrinsic);
                ExtrinsicData {
                    index: index as u32,
                    hash: Some(h256_to_bytes(BlakeTwo256::hash(&raw_bytes))),
                    raw_bytes,
                }
            })
            .collect(),
        storage_changes,
    })
}

fn build_historical_batch<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    chain_id: &str,
    genesis_hash: H256,
    number: u32,
) -> Result<FinalizedBatch, BatchBuildError>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let block_hash = client
        .block_hash(number)
        .map_err(|err| BatchBuildError::HistoricalBlockPruned {
            number,
            hash: None,
            detail: format!("failed to read block hash: {err}"),
        })?
        .ok_or_else(|| BatchBuildError::HistoricalBlockPruned {
            number,
            hash: None,
            detail: "block hash is unavailable".to_string(),
        })?;

    let header = client
        .header(block_hash)
        .map_err(|err| BatchBuildError::HistoricalBlockPruned {
            number,
            hash: Some(block_hash),
            detail: format!("failed to read block header: {err}"),
        })?
        .ok_or_else(|| BatchBuildError::HistoricalBlockPruned {
            number,
            hash: Some(block_hash),
            detail: "block header is unavailable".to_string(),
        })?;
    let body = client
        .block_body(block_hash)
        .map_err(|err| BatchBuildError::HistoricalBlockPruned {
            number,
            hash: Some(block_hash),
            detail: format!("failed to read block body: {err}"),
        })?
        .ok_or_else(|| BatchBuildError::HistoricalBlockPruned {
            number,
            hash: Some(block_hash),
            detail: "block body is unavailable".to_string(),
        })?;
    // Runtime upgrades take effect in the resulting block state, so the parent state identifies
    // the runtime that actually decoded and executed this block's extrinsics.
    let runtime_at = if *header.number() == 0 {
        block_hash
    } else {
        *header.parent_hash()
    };
    let runtime = client
        .runtime_version_at(runtime_at, CallContext::Onchain)
        .map_err(|err| BatchBuildError::HistoricalStatePruned {
            number,
            hash: block_hash,
            detail: format!("failed to read execution runtime version from parent state: {err}"),
        })?;

    let storage_changes = if number == 0 {
        client
            .storage_pairs(block_hash, None, None)
            .map_err(|err| BatchBuildError::HistoricalStatePruned {
                number,
                hash: block_hash,
                detail: format!("failed to read genesis storage: {err}"),
            })?
            .map(|(key, value)| StorageChange {
                raw_key: key.0,
                old_raw_value: None,
                new_raw_value: Some(value.0),
                operation: StorageOperation::Upsert,
            })
            .collect()
    } else {
        let parent_hash = *header.parent_hash();
        let mut runtime_api = client.runtime_api();
        runtime_api.set_call_context(CallContext::Onchain);
        runtime_api
            .execute_block(parent_hash, Block::new(header.clone(), body.clone()).into())
            .map_err(|err| BatchBuildError::HistoricalStatePruned {
                number,
                hash: block_hash,
                detail: format!("failed to re-execute retained block: {err}"),
            })?;
        let state =
            client
                .state_at(parent_hash)
                .map_err(|err| BatchBuildError::HistoricalStatePruned {
                    number,
                    hash: block_hash,
                    detail: format!("failed to read retained parent state: {err}"),
                })?;
        runtime_api
            .into_storage_changes(&state, parent_hash)
            .map_err(|err| BatchBuildError::HistoricalStatePruned {
                number,
                hash: block_hash,
                detail: format!("failed to recover storage changes: {err}"),
            })?
            .main_storage_changes
            .iter()
            .map(|(key, value)| {
                storage_change(
                    client,
                    parent_hash,
                    ObservedStorageChange {
                        raw_key: StorageKey(key.clone()),
                        new_raw_value: value.clone().map(StorageData),
                    },
                )
            })
            .collect()
    };
    let header_raw_bytes = codec::Encode::encode(&header);

    Ok(FinalizedBatch {
        fixture_format_version: FIXTURE_FORMAT_VERSION,
        chain_id: chain_id.to_owned(),
        genesis_hash: h256_to_bytes(genesis_hash),
        runtime: RuntimeIdentity {
            spec_name: runtime.spec_name.to_string(),
            spec_version: runtime.spec_version,
            transaction_version: runtime.transaction_version,
        },
        block: BlockData {
            number: *header.number(),
            hash: h256_to_bytes(block_hash),
            parent_hash: h256_to_bytes(*header.parent_hash()),
            state_root: h256_to_bytes(*header.state_root()),
            extrinsics_root: h256_to_bytes(*header.extrinsics_root()),
            header_raw_bytes: Some(header_raw_bytes),
        },
        extrinsics: body
            .into_iter()
            .enumerate()
            .map(|(index, extrinsic)| {
                let raw_bytes = codec::Encode::encode(&extrinsic);
                ExtrinsicData {
                    index: index as u32,
                    hash: Some(h256_to_bytes(BlakeTwo256::hash(&raw_bytes))),
                    raw_bytes,
                }
            })
            .collect(),
        storage_changes,
    })
}

fn storage_change<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    parent_hash: H256,
    change: ObservedStorageChange,
) -> StorageChange
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let old_raw_value = client
        .storage(parent_hash, &change.raw_key)
        .map_err(|err| {
            debug!(
                "Could not read parent storage value for compatible indexer batch old_raw_value: {err}"
            );
        })
        .ok()
        .flatten()
        .map(|value| value.0);
    let raw_key = change.raw_key.0;
    let new_raw_value = change.new_raw_value.map(|value| value.0);
    let operation = if new_raw_value.is_some() {
        StorageOperation::Upsert
    } else {
        StorageOperation::Delete
    };

    StorageChange {
        raw_key,
        old_raw_value,
        new_raw_value,
        operation,
    }
}

async fn send_batches<RuntimeApi, Executor>(
    sender: BatchSender<RuntimeApi, Executor>,
    mut batch_rx: mpsc::Receiver<QueuedBatch>,
    best_delivery_ready: Arc<AtomicBool>,
) where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let BatchSender {
        client,
        sync_service,
        requires_network_sync_target,
        sink,
        max_queue_len,
        journal,
        cleanup_scheduler,
    } = sender;
    let best_number_at_start = client.info().best_number;
    let mut queue = VecDeque::new();
    let mut stream_open = false;
    let mut retry_delay = INITIAL_BATCH_RETRY;
    let mut next_expected_number = 0;
    let mut live_delivery_ready = false;
    let mut historical_acknowledgements = ProgressLogCounter::default();
    let mut already_submitted_live_drops = ProgressLogCounter::default();

    loop {
        drain_incoming_batches(&mut batch_rx, &mut queue, max_queue_len);

        if !stream_open {
            match sink.open_stream().await {
                Ok(ack) => {
                    if let Err(err) = validate_stream_start_cursor(&client, ack.last_received) {
                        error!(
                            "COMPATIBLE INDEXER FINALIZED BATCH SINK STREAM IS NOT OPEN: the indexer reported a last received finalized block that conflicts with Duniter's known canonical chain. Duniter refuses to continue to avoid mixing two finalized histories. Spooled finalized batches remain on disk and no batch data will be dropped. Retrying stream-start in 5 seconds. Error: {err}"
                        );
                        async_io::Timer::after(STREAM_START_RETRY).await;
                        continue;
                    }

                    if ack.resume_from == 0
                        && let Err(err) = ensure_genesis_batch(
                            &client,
                            &journal,
                            &sink.chain_id,
                            sink.genesis_hash,
                        )
                    {
                        error!(
                            "COMPATIBLE INDEXER FINALIZED BATCH SINK STREAM IS NOT OPEN: the indexer needs the genesis batch, but Duniter cannot build or load it. Start from a database that still has genesis state, or restore the persisted indexer batch spool. Retrying stream-start in 5 seconds. Error: {err}"
                        );
                        async_io::Timer::after(STREAM_START_RETRY).await;
                        continue;
                    }

                    if let Some(cursor) = ack.last_received {
                        acknowledge_or_schedule(&cleanup_scheduler, &journal, cursor.number, true);
                        queue.retain(|batch| batch.number > cursor.number);
                    }

                    stream_open = true;
                    next_expected_number = ack.resume_from;
                    retry_delay = INITIAL_BATCH_RETRY;
                    info!(
                        "Compatible indexer finalized batch sink stream opened for chain_id={} genesis_hash={:?}; next finalized batch to submit is block_number={}",
                        sink.chain_id, sink.genesis_hash, next_expected_number,
                    );
                }
                Err(err) => {
                    error!(
                        "COMPATIBLE INDEXER FINALIZED BATCH SINK STREAM IS NOT OPEN: cannot reach or initialize the indexer at {}; finalized batches remain spooled on disk and no batch data will be dropped. Retrying stream-start in 5 seconds. Error: {err}",
                        sink.stream_start_url,
                    );
                    async_io::Timer::after(STREAM_START_RETRY).await;
                    continue;
                }
            }
        }

        let finalized_number = client.info().finalized_number;
        if !live_delivery_ready
            && historical_catch_up_reached_live_boundary(next_expected_number, finalized_number)
        {
            let best_number = client.info().best_number;
            let network_progress_observed = best_number > best_number_at_start;
            match sync_service.status().await {
                Ok(status) => {
                    let is_major_syncing = sync_service.is_major_syncing();
                    live_delivery_ready = sync_target_reached(
                        is_major_syncing,
                        requires_network_sync_target,
                        network_progress_observed,
                        best_number,
                        status.best_seen_block,
                    );
                    debug!(
                        "Compatible indexer live boundary check: live_delivery_ready={} is_major_syncing={} requires_network_sync_target={} network_progress_observed={} local_best_number={} network_best_seen_block={:?} next_expected_number={} local_finalized_number={}",
                        live_delivery_ready,
                        is_major_syncing,
                        requires_network_sync_target,
                        network_progress_observed,
                        best_number,
                        status.best_seen_block,
                        next_expected_number,
                        finalized_number,
                    );
                    if live_delivery_ready {
                        best_delivery_ready.store(true, Ordering::Release);
                        info!(
                            "Compatible indexer delivery switched permanently to live mode at next_expected_block_number={} local_best_number={} network_best_seen_block={:?}",
                            next_expected_number, best_number, status.best_seen_block,
                        );
                    }
                }
                Err(err) => {
                    warn!(
                        "Could not read Duniter sync status before enabling live indexer delivery; historical mode remains active: {err}"
                    );
                }
            }
        }
        if next_expected_number <= finalized_number {
            let mode = delivery_mode(live_delivery_ready);
            match send_historical_range(
                HistoricalRangeContext {
                    client: &client,
                    sink: &sink,
                    max_queue_len,
                    journal: &journal,
                    cleanup_scheduler: &cleanup_scheduler,
                },
                mode,
                &mut batch_rx,
                &mut queue,
                next_expected_number..=finalized_number,
                &mut historical_acknowledgements,
            )
            .await
            {
                Ok(next_number) => {
                    next_expected_number = next_number;
                    continue;
                }
                Err(err) => {
                    stream_open = false;
                    error!(
                        "Compatible indexer historical catch-up failed; queued live batches remain in memory and stream-start will be retried after {retry_delay:?}. Error: {err}",
                    );
                    async_io::Timer::after(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                    continue;
                }
            }
        }

        if let Some(batch) = queue.front() {
            let batch_number = batch.number;
            let batch_hash = batch.hash;

            if batch_number < next_expected_number {
                acknowledge_or_schedule(&cleanup_scheduler, &journal, batch_number, false);
                queue.pop_front();
                if let Some((first, last, count)) =
                    already_submitted_live_drops.record(batch_number)
                {
                    info!(
                        "Removed {count} already submitted live finalized batch cursors from the memory queue after catch-up; block_number_range={first}..={last}"
                    );
                }
                continue;
            }

            if batch_number > next_expected_number {
                match send_historical_range(
                    HistoricalRangeContext {
                        client: &client,
                        sink: &sink,
                        max_queue_len,
                        journal: &journal,
                        cleanup_scheduler: &cleanup_scheduler,
                    },
                    delivery_mode(live_delivery_ready),
                    &mut batch_rx,
                    &mut queue,
                    next_expected_number..=batch_number.saturating_sub(1),
                    &mut historical_acknowledgements,
                )
                .await
                {
                    Ok(next_number) => {
                        next_expected_number = next_number;
                        continue;
                    }
                    Err(err) => {
                        stream_open = false;
                        error!(
                            "Compatible indexer gap catch-up failed before live batch block_number={batch_number} block_hash={batch_hash:?}; queued batches remain in memory and stream-start will be retried after {retry_delay:?}. Error: {err}",
                        );
                        async_io::Timer::after(retry_delay).await;
                        retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                        continue;
                    }
                }
            }

            let encoded = match journal.load(batch_number, batch_hash) {
                Ok(Some(encoded)) => encoded,
                Ok(None) => {
                    stream_open = false;
                    error!(
                        "Compatible indexer complete finalized batch is missing for queued block_number={batch_number} block_hash={batch_hash:?}; retrying stream-start without acknowledging or dropping the batch",
                    );
                    async_io::Timer::after(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                    continue;
                }
                Err(err) => {
                    stream_open = false;
                    error!(
                        "Compatible indexer complete finalized batch cannot be loaded for block_number={batch_number} block_hash={batch_hash:?}; retrying stream-start without acknowledging or dropping the batch: {err}",
                    );
                    async_io::Timer::after(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                    continue;
                }
            };
            let mode = delivery_mode(live_delivery_ready);
            match sink.send_batches(vec![encoded], mode).await {
                Ok(()) => {
                    info!(
                        "Compatible indexer durably acknowledged finalized batch chunk block_number={} block_hash={:?} delivery_mode={}; removing it from memory queue and storage diff journal",
                        batch_number,
                        batch_hash,
                        mode.as_str(),
                    );
                    acknowledge_or_schedule(&cleanup_scheduler, &journal, batch_number, false);
                    next_expected_number = batch_number.saturating_add(1);
                    queue.pop_front();
                    retry_delay = INITIAL_BATCH_RETRY;
                }
                Err(err) => {
                    stream_open = false;
                    error!(
                        "Compatible indexer did not durably acknowledge live finalized batch chunk block_number={batch_number} block_hash={batch_hash:?}; it stays queued and will be retried after {retry_delay:?}. Error: {err}",
                    );
                    async_io::Timer::after(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_BATCH_RETRY);
                }
            }
        } else {
            match batch_rx.next().await {
                Some(batch) => queue.push_back(batch),
                None => {
                    error!(
                        "Compatible indexer finalized batch sender stopped: producer channel closed"
                    );
                    return;
                }
            }
        }
    }
}

fn drain_incoming_batches(
    batch_rx: &mut mpsc::Receiver<QueuedBatch>,
    queue: &mut VecDeque<QueuedBatch>,
    max_queue_len: usize,
) {
    let max_queue_len = max_queue_len.max(1);
    while let Ok(batch) = batch_rx.try_recv() {
        if queue.len() < max_queue_len {
            queue.push_back(batch);
        } else {
            let dropped = DROPPED_CURSOR_LOG_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            if dropped.is_multiple_of(PROGRESS_LOG_INTERVAL) {
                warn!(
                    "Compatible indexer in-memory cursor queue reached configured limit {max_queue_len}; dropped {PROGRESS_LOG_INTERVAL} additional redundant wake-up cursors while batches remain recoverable from journal continuity"
                );
            }
        }
    }
}

async fn send_historical_range<RuntimeApi, Executor>(
    context: HistoricalRangeContext<'_, RuntimeApi, Executor>,
    mode: BatchDeliveryMode,
    batch_rx: &mut mpsc::Receiver<QueuedBatch>,
    queue: &mut VecDeque<QueuedBatch>,
    range: std::ops::RangeInclusive<u32>,
    historical_acknowledgements: &mut ProgressLogCounter,
) -> Result<u32, String>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let HistoricalRangeContext {
        client,
        sink,
        max_queue_len,
        journal,
        cleanup_scheduler,
    } = context;
    let (from, to) = range.into_inner();
    if from > to {
        return Ok(from);
    }

    let mut chunk_from = from;
    while chunk_from <= to {
        drain_incoming_batches(batch_rx, queue, max_queue_len);
        let chunk_to = historical_chunk_end(chunk_from, to);
        let mut encoded_batches = Vec::with_capacity((chunk_to - chunk_from + 1) as usize);
        let mut first_queued = None;

        for number in chunk_from..=chunk_to {
            let hash = client
                .block_hash(number)
                .map_err(|err| format!("failed to read historical block hash at #{number}: {err}"))?
                .ok_or_else(|| format!("historical block hash is unavailable at #{number}"))?;
            first_queued.get_or_insert(QueuedBatch { number, hash });
            let encoded = match journal.load(number, hash)? {
                Some(encoded) => encoded,
                None => {
                    let batch =
                        build_historical_batch(client, &sink.chain_id, sink.genesis_hash, number)
                            .map_err(|err| err.to_string())?;
                    journal.append(&batch)?;
                    codec::Encode::encode(&batch)
                }
            };
            encoded_batches.push(encoded);
        }

        let chunk_len = encoded_batches.len();
        match sink.send_batches(encoded_batches, mode).await {
            Ok(()) => {
                acknowledge_or_schedule(cleanup_scheduler, journal, chunk_to, false);
                for number in chunk_from..=chunk_to {
                    if let Some((first, last, count)) = historical_acknowledgements.record(number) {
                        info!(
                            "Compatible indexer acknowledged {count} historical finalized batches; block_number_range={first}..={last}; catch-up continues"
                        );
                    }
                }
                debug!(
                    "Compatible indexer durably acknowledged recovered batch chunk block_number_range={}..={} batch_count={} delivery_mode={}",
                    chunk_from,
                    chunk_to,
                    chunk_len,
                    mode.as_str(),
                );
            }
            Err(err) => {
                let queued = first_queued.expect("a non-empty historical chunk has a first batch");
                if !queue
                    .iter()
                    .any(|batch| batch.number == queued.number && batch.hash == queued.hash)
                {
                    queue.push_front(queued);
                }
                return Err(format!(
                    "historical batch chunk block_number_range={chunk_from}..={chunk_to} was built but not durably acknowledged; it remains in the journal and will be retried: {err}"
                ));
            }
        }
        if chunk_to == to {
            break;
        }
        chunk_from = chunk_to + 1;
    }

    Ok(to.saturating_add(1))
}

fn delivery_mode(live_delivery_ready: bool) -> BatchDeliveryMode {
    if live_delivery_ready {
        BatchDeliveryMode::Live
    } else {
        BatchDeliveryMode::Historical
    }
}

fn sync_target_reached(
    is_major_syncing: bool,
    requires_network_sync_target: bool,
    network_progress_observed: bool,
    best_number: u32,
    best_seen_block: Option<u32>,
) -> bool {
    !is_major_syncing
        && match best_seen_block {
            Some(target) => best_number.saturating_add(LIVE_SYNC_TARGET_MAX_LAG) >= target,
            None => !requires_network_sync_target || network_progress_observed,
        }
}

fn historical_catch_up_reached_live_boundary(
    next_expected_number: u32,
    finalized_number: u32,
) -> bool {
    // At the network head, a continuously producing chain may finalize another
    // block while PostgreSQL commits and acknowledges the preceding one. Requiring
    // an empty finalized backlog would then keep delivery historical forever. A
    // small bounded backlog is the live boundary once the node itself has reached
    // the network sync target.
    next_expected_number.saturating_add(LIVE_FINALIZED_BACKLOG_MAX_LAG) >= finalized_number
}

fn historical_chunk_end(from: u32, to: u32) -> u32 {
    to.min(from.saturating_add(SINK_CHUNK_MAX_BATCHES as u32 - 1))
}

fn queue_batch(batch: &FinalizedBatch) -> QueuedBatch {
    QueuedBatch {
        number: batch.block.number,
        hash: H256::from(batch.block.hash),
    }
}

fn validate_stream_start_cursor<RuntimeApi, Executor>(
    client: &Arc<FullClient<RuntimeApi, Executor>>,
    cursor: Option<BlockCursor>,
) -> Result<(), String>
where
    RuntimeApi: sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>
        + Send
        + Sync
        + 'static,
    RuntimeApi::RuntimeApi: super::RuntimeApiCollection,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
    Executor: sc_executor::sp_wasm_interface::HostFunctions + 'static,
{
    let Some(cursor) = cursor else {
        return Ok(());
    };

    let finalized_number = client.info().finalized_number;
    if cursor.number > finalized_number {
        warn!(
            "Compatible indexer stream-start cursor is ahead of local finalized head; accepting the indexer as source of truth and waiting for Duniter to catch up if needed. indexer_last_received_block_number={} local_finalized_number={}",
            cursor.number, finalized_number,
        );
    }

    let Some(expected_hash) = cursor.hash else {
        warn!(
            "Compatible indexer stream-start cursor provided block_number={} without block_hash; accepting it without continuity hash verification",
            cursor.number,
        );
        return Ok(());
    };

    let local_hash = match client.block_hash(cursor.number) {
        Ok(Some(local_hash)) => local_hash,
        Ok(None) => {
            warn!(
                "Compatible indexer stream-start cursor block hash is not available locally; accepting the indexer as source of truth. block_number={} indexer_hash={:?}",
                cursor.number, expected_hash,
            );
            return Ok(());
        }
        Err(err) => {
            warn!(
                "Compatible indexer stream-start cursor block hash could not be read locally; accepting the indexer as source of truth. block_number={} indexer_hash={:?} error={err}",
                cursor.number, expected_hash,
            );
            return Ok(());
        }
    };

    if local_hash != expected_hash {
        return Err(format!(
            "indexer canonical-chain conflict: last_received_block_number={} indexer_hash={:?} local_canonical_hash={:?}",
            cursor.number, expected_hash, local_hash,
        ));
    }

    Ok(())
}

impl HttpSink {
    fn new(base_url: String, token: Option<String>, chain_id: String, genesis_hash: H256) -> Self {
        let base_url = base_url.trim_end_matches('/');
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("reqwest client configuration is valid; qed"),
            stream_start_url: format!("{base_url}/stream-start"),
            batches_url: format!("{base_url}/batches"),
            best_chain_url: format!("{base_url}/best-chain"),
            token,
            chain_id,
            genesis_hash,
        }
    }

    async fn open_stream(&self) -> Result<StreamStartAck, String> {
        let body = serde_json::json!({
            "chain_id": self.chain_id,
            "genesis_hash": format!("{:?}", self.genesis_hash),
        })
        .to_string();
        let mut request = self
            .client
            .post(&self.stream_start_url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| format!("stream-start transport failure: {err}"))?;
        let body = classify_response("stream-start", response).await?;
        parse_stream_start_ack(&body)
    }

    async fn send_batches(
        &self,
        encoded_batches: Vec<Vec<u8>>,
        mode: BatchDeliveryMode,
    ) -> Result<(), String> {
        debug_assert!(!encoded_batches.is_empty());
        debug_assert!(encoded_batches.len() <= SINK_CHUNK_MAX_BATCHES);
        let mut request = self
            .client
            .post(&self.batches_url)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(BATCH_MODE_HEADER, mode.as_str())
            .body(codec::Encode::encode(&encoded_batches));
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| format!("batch chunk transport failure: {err}"))?;
        classify_response("batch chunk", response).await.map(|_| ())
    }

    async fn send_best_chain(&self, update: BestChainUpdate) -> Result<(), String> {
        let mut request = self
            .client
            .post(&self.best_chain_url)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(codec::Encode::encode(&update));
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .await
            .map_err(|err| format!("BEST chain update transport failure: {err}"))?;
        classify_response("BEST chain update", response)
            .await
            .map(|_| ())
    }
}

async fn classify_response(endpoint: &str, response: reqwest::Response) -> Result<String, String> {
    let status = response.status();
    let body = response
        .text()
        .await
        .unwrap_or_else(|err| format!("<failed to read compatible indexer response body: {err}>"));
    let normalized = body.to_ascii_lowercase();

    if status.is_success() {
        if is_duplicate_response(&normalized) {
            info!("Compatible indexer acknowledged an already-ingested duplicate finalized batch");
        }
        if normalized.contains("unsupported_runtime") {
            warn!(
                "Compatible indexer stored raw finalized batch but reported unsupported_runtime projection status"
            );
        }
        return Ok(body);
    }

    Err(format!(
        "{endpoint} HTTP status={} indexer_error={} body={}",
        status,
        describe_indexer_error(&normalized),
        body,
    ))
}

fn parse_stream_start_ack(body: &str) -> Result<StreamStartAck, String> {
    if body.trim().is_empty() {
        return Ok(StreamStartAck {
            resume_from: 0,
            last_received: None,
            best: None,
        });
    }

    let value = serde_json::from_str::<serde_json::Value>(body).map_err(|err| {
        format!("stream-start response must be JSON or empty: {err}; body={body}")
    })?;
    let cursor = parse_last_received_cursor(&value)?;
    let best = value
        .get("last_received_best_block")
        .map(|value| parse_cursor_value(value, "last_received_best_block"))
        .transpose()?
        .flatten();
    Ok(StreamStartAck {
        resume_from: cursor
            .map(|cursor| cursor.number.saturating_add(1))
            .unwrap_or(0),
        last_received: cursor,
        best,
    })
}

fn parse_last_received_cursor(value: &serde_json::Value) -> Result<Option<BlockCursor>, String> {
    if value.is_null() {
        return Ok(None);
    }

    for key in [
        "last_received_finalized_block",
        "last_received_block",
        "last_finalized_batch",
    ] {
        if let Some(cursor_value) = value.get(key) {
            return parse_cursor_value(cursor_value, key);
        }
    }

    let number = value
        .get("last_received_finalized_block_number")
        .or_else(|| value.get("last_received_block_number"))
        .or_else(|| value.get("last_block_number"));
    let hash = value
        .get("last_received_finalized_block_hash")
        .or_else(|| value.get("last_received_block_hash"))
        .or_else(|| value.get("last_block_hash"));

    match number {
        Some(number) => Ok(Some(BlockCursor {
            number: parse_u32(number, "last received block number")?,
            hash: hash
                .map(|hash| parse_hash(hash, "last received block hash"))
                .transpose()?,
        })),
        None => Ok(None),
    }
}

fn parse_cursor_value(
    value: &serde_json::Value,
    field_name: &str,
) -> Result<Option<BlockCursor>, String> {
    if value.is_null() {
        return Ok(None);
    }

    let Some(object) = value.as_object() else {
        return Err(format!("{field_name} must be an object or null"));
    };
    let number = object
        .get("number")
        .or_else(|| object.get("block_number"))
        .ok_or_else(|| format!("{field_name} must include number"))?;
    let hash = object.get("hash").or_else(|| object.get("block_hash"));

    Ok(Some(BlockCursor {
        number: parse_u32(number, field_name)?,
        hash: hash.map(|hash| parse_hash(hash, field_name)).transpose()?,
    }))
}

fn parse_u32(value: &serde_json::Value, field_name: &str) -> Result<u32, String> {
    let number = value
        .as_u64()
        .ok_or_else(|| format!("{field_name} must be an unsigned integer"))?;
    u32::try_from(number).map_err(|_| format!("{field_name} is too large for u32: {number}"))
}

fn parse_hash(value: &serde_json::Value, field_name: &str) -> Result<H256, String> {
    let hash = value
        .as_str()
        .ok_or_else(|| format!("{field_name} hash must be a hex string"))?;
    hash.parse::<H256>()
        .map_err(|err| format!("{field_name} hash is invalid: {err}"))
}

fn describe_indexer_error(normalized_body: &str) -> &'static str {
    if normalized_body.contains("invalid genesis") {
        "invalid genesis hash"
    } else if normalized_body.contains("malformed scale") {
        "malformed SCALE body"
    } else if normalized_body.contains("json") {
        "accidental JSON submission"
    } else if normalized_body.contains("invalid batch hash") {
        "invalid batch hash"
    } else if normalized_body.contains("malformed storage") {
        "malformed storage change"
    } else if normalized_body.contains("missing parent") || normalized_body.contains("continuity") {
        "missing parent / stream continuity error"
    } else if normalized_body.contains("historical_block_pruned") {
        "historical block pruned"
    } else if normalized_body.contains("historical_state_pruned") {
        "historical state pruned"
    } else if normalized_body.contains("historical_storage_diff_unavailable") {
        "historical storage diff unavailable"
    } else if normalized_body.contains("unsupported_runtime") {
        "unsupported runtime projection"
    } else if is_duplicate_response(normalized_body) {
        "duplicate already-ingested batch"
    } else {
        "unclassified indexer error"
    }
}

fn is_duplicate_response(normalized_body: &str) -> bool {
    normalized_body.contains("duplicate")
        || normalized_body.contains("already-ingested")
        || normalized_body.contains("already_ingested")
}

fn h256_to_bytes(hash: H256) -> [u8; 32] {
    hash.to_fixed_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_batch() -> FinalizedBatch {
        FinalizedBatch {
            fixture_format_version: FIXTURE_FORMAT_VERSION,
            chain_id: "test".to_owned(),
            genesis_hash: [1; 32],
            runtime: RuntimeIdentity {
                spec_name: "test".to_owned(),
                spec_version: 1,
                transaction_version: 1,
            },
            block: BlockData {
                number: 7,
                hash: [2; 32],
                parent_hash: [3; 32],
                state_root: [4; 32],
                extrinsics_root: [5; 32],
                header_raw_bytes: None,
            },
            extrinsics: Vec::new(),
            storage_changes: Vec::new(),
        }
    }

    fn sample_batch_at(number: u32) -> FinalizedBatch {
        let mut batch = sample_batch();
        batch.block.number = number;
        batch.block.hash = H256::from_low_u64_be(u64::from(number)).to_fixed_bytes();
        batch
    }

    fn temporary_directory(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "duniter-indexer-batch-{test_name}-{}",
            std::process::id(),
        ))
    }

    #[test]
    fn historical_chunks_are_bounded_and_include_the_tail() {
        assert_eq!(historical_chunk_end(0, 1_000), 63);
        assert_eq!(historical_chunk_end(64, 1_000), 127);
        assert_eq!(historical_chunk_end(995, 1_000), 1_000);
        assert_eq!(historical_chunk_end(42, 42), 42);
        assert_eq!(historical_chunk_end(u32::MAX - 1, u32::MAX), u32::MAX);
    }

    #[test]
    fn delivery_mode_switches_only_after_catch_up() {
        assert_eq!(delivery_mode(false).as_str(), "historical");
        assert_eq!(delivery_mode(true).as_str(), "live");
    }

    #[test]
    fn live_delivery_waits_until_the_network_sync_target_is_reached() {
        assert!(!sync_target_reached(true, true, true, 1_000, Some(1_000)));
        assert!(!sync_target_reached(false, true, false, 997, Some(1_000)));
        assert!(sync_target_reached(false, true, false, 998, Some(1_000)));
        assert!(sync_target_reached(false, true, false, 999, Some(1_000)));
        assert!(sync_target_reached(false, true, false, 1_000, Some(1_000)));
        assert!(sync_target_reached(false, true, false, 1_001, Some(1_000)));
        assert!(!sync_target_reached(false, true, false, 0, None));
        assert!(sync_target_reached(false, true, true, 1, None));
        assert!(sync_target_reached(false, false, false, 0, None));
    }

    #[test]
    fn live_boundary_allows_a_small_bounded_finalized_backlog() {
        assert!(!historical_catch_up_reached_live_boundary(39, 42));
        assert!(historical_catch_up_reached_live_boundary(40, 42));
        assert!(historical_catch_up_reached_live_boundary(41, 42));
        assert!(historical_catch_up_reached_live_boundary(42, 42));
        assert!(historical_catch_up_reached_live_boundary(43, 42));
        assert!(historical_catch_up_reached_live_boundary(
            u32::MAX,
            u32::MAX
        ));
    }

    #[test]
    fn stream_start_empty_body_starts_at_genesis() {
        let ack = parse_stream_start_ack("").expect("empty response is valid");

        assert_eq!(ack.resume_from, 0);
        assert!(ack.last_received.is_none());
    }

    #[test]
    fn stream_start_parses_nested_cursor() {
        let hash = H256::repeat_byte(0x2a);
        let body = serde_json::json!({
            "last_received_finalized_block": {
                "number": 41,
                "hash": format!("{hash:?}"),
            }
        })
        .to_string();

        let ack = parse_stream_start_ack(&body).expect("cursor is valid");
        let cursor = ack.last_received.expect("cursor is present");
        assert_eq!(ack.resume_from, 42);
        assert_eq!(cursor.number, 41);
        assert_eq!(cursor.hash, Some(hash));
    }

    #[test]
    fn stream_start_rejects_out_of_range_block_number() {
        let body = serde_json::json!({
            "last_received_finalized_block_number": u64::from(u32::MAX) + 1,
        })
        .to_string();

        assert!(parse_stream_start_ack(&body).is_err());
    }

    #[test]
    fn finalized_batch_store_round_trips_exact_http_body() {
        let dir = temporary_directory("round-trip");
        let store = FinalizedBatchStore::new(dir.clone()).expect("store can be created");
        let batch = sample_batch();
        let queued = store.persist(&batch).expect("batch can be persisted");

        let loaded = store
            .load(queued.number, queued.hash)
            .expect("batch can be loaded")
            .expect("batch exists");
        assert_eq!(loaded, codec::Encode::encode(&batch));

        store.remove(queued.number, queued.hash);
        fs::remove_dir_all(dir).expect("test directory can be removed");
    }

    #[test]
    fn finalized_batch_store_rejects_trailing_bytes() {
        let dir = temporary_directory("trailing-bytes");
        let store = FinalizedBatchStore::new(dir.clone()).expect("store can be created");
        let batch = sample_batch();
        let queued = store.persist(&batch).expect("batch can be persisted");
        let path = store.path(queued.number, queued.hash);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(path)
            .expect("batch file can be opened");
        file.write_all(&[0]).expect("trailing byte can be added");

        assert!(store.load(queued.number, queued.hash).is_err());

        fs::remove_dir_all(dir).expect("test directory can be removed");
    }

    #[test]
    fn incoming_cursor_queue_never_exceeds_configured_bound() {
        let (mut tx, mut rx) = mpsc::channel(4);
        for number in 1..=3 {
            tx.try_send(QueuedBatch {
                number,
                hash: H256::from_low_u64_be(u64::from(number)),
            })
            .expect("test channel has capacity");
        }
        let mut queue = VecDeque::new();

        drain_incoming_batches(&mut rx, &mut queue, 2);

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.front().map(|batch| batch.number), Some(1));
        assert_eq!(queue.back().map(|batch| batch.number), Some(2));
    }

    #[test]
    fn accepted_cursor_removes_only_older_spool_files() {
        let dir = temporary_directory("cursor-cleanup");
        fs::create_dir_all(&dir).expect("test directory can be created");
        let acknowledged = dir.join(format!("{:010}-old.scale", 7));
        let pending = dir.join(format!("{:010}-new.scale", 8));
        let unrelated = dir.join("README");
        fs::write(&acknowledged, []).expect("acknowledged fixture can be written");
        fs::write(&pending, []).expect("pending fixture can be written");
        fs::write(&unrelated, []).expect("unrelated fixture can be written");

        remove_files_up_to(&dir, 7, "test").expect("acknowledged files can be removed");

        assert!(!acknowledged.exists());
        assert!(pending.exists());
        assert!(unrelated.exists());
        fs::remove_dir_all(dir).expect("test directory can be removed");
    }

    #[test]
    fn cleanup_scheduler_keeps_highest_cumulative_cursor() {
        let scheduler = CleanupScheduler::default();
        scheduler.schedule_up_to(7);
        scheduler.schedule_up_to(6);

        let pending = scheduler.pending.lock();
        assert_eq!(pending.up_to, Some(7));
    }

    #[test]
    fn progress_log_counter_reports_each_group_of_one_hundred() {
        let mut counter = ProgressLogCounter::default();

        for number in 7..106 {
            assert_eq!(counter.record(number), None);
        }
        assert_eq!(counter.record(106), Some((7, 106, 100)));
        assert_eq!(counter.record(200), None);
    }

    #[test]
    fn segmented_journal_group_commits_one_hundred_batches() {
        let dir = temporary_directory("segmented-group-commit");
        let journal = SegmentedBatchJournal::new(dir.clone()).expect("journal can be created");

        for number in 1..=GROUP_COMMIT_BLOCKS {
            journal
                .append(&sample_batch_at(number))
                .expect("batch can be appended");
        }

        assert_eq!(journal.inner.lock().unsynced_records, 0);
        drop(journal);
        let reopened = SegmentedBatchJournal::new(dir.clone()).expect("journal can be reopened");
        let hash = H256::from_low_u64_be(u64::from(GROUP_COMMIT_BLOCKS));
        assert!(
            reopened
                .load(GROUP_COMMIT_BLOCKS, hash)
                .expect("journal can be read")
                .is_some()
        );
        fs::remove_dir_all(dir).expect("test directory can be removed");
    }

    #[test]
    fn segmented_journal_truncates_incomplete_crash_tail() {
        let dir = temporary_directory("segmented-crash-tail");
        let journal = SegmentedBatchJournal::new(dir.clone()).expect("journal can be created");
        let batch = sample_batch_at(9);
        journal.append(&batch).expect("batch can be appended");
        let valid_len = journal.inner.lock().segments[&0].size;
        drop(journal);

        let path = journal_segment_path(&dir, 0);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("segment can be opened");
        file.write_all(&JOURNAL_RECORD_MAGIC[..2])
            .expect("partial record can be written");
        drop(file);

        let reopened = SegmentedBatchJournal::new(dir.clone()).expect("tail can be recovered");
        assert_eq!(fs::metadata(path).expect("segment exists").len(), valid_len);
        assert!(
            reopened
                .load(9, H256::from_low_u64_be(9))
                .expect("journal can be read")
                .is_some()
        );
        fs::remove_dir_all(dir).expect("test directory can be removed");
    }

    #[test]
    fn segmented_journal_deletes_only_fully_acknowledged_closed_segments() {
        let dir = temporary_directory("segmented-ack-gc");
        let journal = SegmentedBatchJournal::new(dir.clone()).expect("journal can be created");
        journal
            .append(&sample_batch_at(1))
            .expect("batch can be appended");
        {
            let mut inner = journal.inner.lock();
            rotate_journal_segment(&mut inner).expect("journal can be rotated");
        }
        assert!(journal_segment_path(&dir, 0).exists());

        journal
            .acknowledge_cursor(1)
            .expect("acknowledged segment can be removed");

        assert!(!journal_segment_path(&dir, 0).exists());
        assert!(journal_segment_path(&dir, 1).exists());
        assert!(
            journal
                .load(1, H256::from_low_u64_be(1))
                .expect("journal can be read")
                .is_none()
        );
        fs::remove_dir_all(dir).expect("test directory can be removed");
    }
}
