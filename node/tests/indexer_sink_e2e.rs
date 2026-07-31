#![cfg(feature = "gtest")]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use codec::Decode;

const TOKEN: &str = "indexer-e2e-secret";

#[derive(Debug, Decode)]
struct FinalizedBatch {
    fixture_format_version: u32,
    chain_id: String,
    genesis_hash: [u8; 32],
    runtime: RuntimeIdentity,
    block: BlockData,
    extrinsics: Vec<ExtrinsicData>,
    storage_changes: Vec<StorageChange>,
}

#[derive(Debug, Decode)]
struct RuntimeIdentity {
    spec_name: String,
    spec_version: u32,
    transaction_version: u32,
}

#[derive(Debug, Decode)]
struct BlockData {
    number: u32,
    hash: [u8; 32],
    parent_hash: [u8; 32],
    state_root: [u8; 32],
    extrinsics_root: [u8; 32],
    header_raw_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Decode)]
struct ExtrinsicData {
    index: u32,
    raw_bytes: Vec<u8>,
    hash: Option<[u8; 32]>,
}

#[derive(Debug, Decode)]
struct StorageChange {
    raw_key: Vec<u8>,
    old_raw_value: Option<Vec<u8>>,
    new_raw_value: Option<Vec<u8>>,
    operation: StorageOperation,
}

#[derive(Debug, Decode)]
enum StorageOperation {
    Upsert,
    Delete,
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn local_node_submits_a_complete_genesis_batch_to_an_indexer() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind indexer mock");
    let address = listener.local_addr().expect("read indexer mock address");
    let (batch_tx, batch_rx) = mpsc::sync_channel(1);

    let server = std::thread::spawn(move || {
        let stream_start = read_request(listener.accept().expect("accept stream-start").0);
        assert_eq!(stream_start.path, "/stream-start");
        assert_eq!(
            stream_start.content_type.as_deref(),
            Some("application/json")
        );
        assert_eq!(
            stream_start.authorization.as_deref(),
            Some("Bearer indexer-e2e-secret")
        );
        let stream_identity: serde_json::Value =
            serde_json::from_slice(&stream_start.body).expect("decode stream-start JSON");
        assert_eq!(stream_identity["chain_id"], "gtest_local");
        let genesis_hash = stream_identity["genesis_hash"]
            .as_str()
            .expect("stream-start genesis hash")
            .to_owned();
        respond(
            stream_start.stream,
            "202 Accepted",
            "application/json",
            b"{}",
        );

        let request = read_request(listener.accept().expect("accept batch").0);
        assert_eq!(request.path, "/batches");
        assert_eq!(
            request.content_type.as_deref(),
            Some("application/octet-stream")
        );
        assert_eq!(
            request.authorization.as_deref(),
            Some("Bearer indexer-e2e-secret")
        );
        assert_eq!(request.batch_mode.as_deref(), Some("live"));
        let mut payload = request.body.as_slice();
        let encoded_batches =
            Vec::<Vec<u8>>::decode(&mut payload).expect("decode finalized batch chunk SCALE");
        assert!(payload.is_empty(), "batch chunk has trailing SCALE bytes");
        assert_eq!(
            encoded_batches.len(),
            1,
            "genesis is sent as one batch chunk"
        );
        let mut encoded_batch = encoded_batches[0].as_slice();
        let batch =
            FinalizedBatch::decode(&mut encoded_batch).expect("decode finalized batch SCALE");
        assert!(encoded_batch.is_empty(), "batch has trailing SCALE bytes");
        assert_eq!(batch.fixture_format_version, 1);
        assert_eq!(batch.chain_id, "gtest_local");
        assert_eq!(format_hash(batch.genesis_hash), genesis_hash);
        assert_eq!(batch.block.number, 0);
        assert_eq!(batch.block.hash, batch.genesis_hash);
        assert_eq!(batch.block.parent_hash, [0; 32]);
        assert!(batch.block.header_raw_bytes.is_some());
        assert!(batch.extrinsics.is_empty());
        assert!(
            !batch.storage_changes.is_empty(),
            "genesis batch must contain the complete top-level storage state"
        );
        assert!(batch.storage_changes.iter().all(|change| {
            !change.raw_key.is_empty()
                && change.old_raw_value.is_none()
                && change.new_raw_value.is_some()
                && matches!(change.operation, StorageOperation::Upsert)
        }));
        assert!(!batch.runtime.spec_name.is_empty());
        let _ = (
            batch.runtime.spec_version,
            batch.runtime.transaction_version,
            batch.block.state_root,
            batch.block.extrinsics_root,
        );
        for extrinsic in &batch.extrinsics {
            let _ = (extrinsic.index, &extrinsic.raw_bytes, extrinsic.hash);
        }

        respond(
            request.stream,
            "200 OK",
            "application/json",
            b"{\"status\":\"accepted\"}",
        );
        batch_tx.send(()).expect("report consumed batch");
    });

    let child = Command::new(env!("CARGO_BIN_EXE_duniter"))
        .args([
            "--tmp",
            "--chain",
            "gtest_local",
            "--validator",
            "--unsafe-force-node-key-generation",
            "--sync",
            "full",
            "--state-pruning",
            "256",
            "--blocks-pruning",
            "archive-canonical",
            "--indexer-batch-sink-url",
            &format!("http://{address}"),
        ])
        .env("DUNITER_INDEXER_BATCH_SINK_TOKEN", TOKEN)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start local Duniter node");
    let _node = ChildGuard(child);

    batch_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("indexer did not consume the genesis batch in time");
    server.join().expect("indexer mock thread");
}

struct HttpRequest {
    stream: TcpStream,
    path: String,
    content_type: Option<String>,
    authorization: Option<String>,
    batch_mode: Option<String>,
    body: Vec<u8>,
}

fn read_request(mut stream: TcpStream) -> HttpRequest {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("set request timeout");
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut chunk = [0_u8; 4096];
        let read = stream.read(&mut chunk).expect("read HTTP request");
        assert!(read > 0, "HTTP connection closed before headers");
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(position) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
        assert!(
            bytes.len() < 64 * 1024,
            "HTTP request headers are too large"
        );
    };

    let headers = std::str::from_utf8(&bytes[..header_end]).expect("UTF-8 HTTP headers");
    let mut lines = headers.split("\r\n");
    let request_line = lines.next().expect("HTTP request line");
    let path = request_line
        .split_whitespace()
        .nth(1)
        .expect("HTTP request path")
        .to_owned();
    let mut content_length = 0_usize;
    let mut content_type = None;
    let mut authorization = None;
    let mut batch_mode = None;
    for line in lines.filter(|line| !line.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        match name.to_ascii_lowercase().as_str() {
            "content-length" => content_length = value.trim().parse().expect("content length"),
            "content-type" => content_type = Some(value.trim().to_owned()),
            "authorization" => authorization = Some(value.trim().to_owned()),
            "duniter-batch-mode" => batch_mode = Some(value.trim().to_owned()),
            _ => {}
        }
    }
    while bytes.len() - header_end < content_length {
        let mut chunk = [0_u8; 64 * 1024];
        let read = stream.read(&mut chunk).expect("read HTTP body");
        assert!(read > 0, "HTTP connection closed before request body");
        bytes.extend_from_slice(&chunk[..read]);
    }
    let body = bytes[header_end..header_end + content_length].to_vec();

    HttpRequest {
        stream,
        path,
        content_type,
        authorization,
        batch_mode,
        body,
    }
}

fn respond(mut stream: TcpStream, status: &str, content_type: &str, body: &[u8]) {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write HTTP response headers");
    stream.write_all(body).expect("write HTTP response body");
    stream.flush().expect("flush HTTP response");
}

fn format_hash(hash: [u8; 32]) -> String {
    format!("0x{}", hex::encode(hash))
}
