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

#[derive(Debug, Decode, Clone, Copy)]
struct BestBlockRef {
    number: u32,
    hash: [u8; 32],
}

#[derive(Debug, Decode)]
struct BestChainUpdate {
    fixture_format_version: u32,
    chain_id: String,
    genesis_hash: [u8; 32],
    from: BestBlockRef,
    to: BestBlockRef,
    retracted: Vec<BestBlockRef>,
    enacted_batches: Vec<Vec<u8>>,
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn local_node_submits_finalized_and_best_chain_batches_to_an_indexer() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind indexer mock");
    let address = listener.local_addr().expect("read indexer mock address");
    let rpc_listener = TcpListener::bind("127.0.0.1:0").expect("reserve RPC port");
    let rpc_address = rpc_listener.local_addr().expect("read RPC address");
    drop(rpc_listener);
    let (batch_tx, batch_rx) = mpsc::sync_channel(1);

    let server = std::thread::spawn(move || {
        let mut genesis_hash = None;
        let mut received_finalized_genesis = false;
        let mut received_best = false;
        while !received_finalized_genesis || !received_best {
            let request = read_request(listener.accept().expect("accept sink request").0);
            assert_eq!(
                request.authorization.as_deref(),
                Some("Bearer indexer-e2e-secret")
            );
            match request.path.as_str() {
                "/stream-start" => {
                    assert_eq!(request.content_type.as_deref(), Some("application/json"));
                    let identity: serde_json::Value =
                        serde_json::from_slice(&request.body).expect("decode stream-start JSON");
                    assert_eq!(identity["chain_id"], "gtest_local");
                    let hash = identity["genesis_hash"]
                        .as_str()
                        .expect("stream-start genesis hash")
                        .to_owned();
                    genesis_hash.get_or_insert_with(|| hash.clone());
                    let response = format!(
                        "{{\"last_received_best_block\":{{\"number\":0,\"hash\":\"{hash}\"}}}}"
                    );
                    respond(
                        request.stream,
                        "202 Accepted",
                        "application/json",
                        response.as_bytes(),
                    );
                }
                "/batches" => {
                    assert_eq!(
                        request.content_type.as_deref(),
                        Some("application/octet-stream")
                    );
                    assert_eq!(request.batch_mode.as_deref(), Some("live"));
                    let mut payload = request.body.as_slice();
                    let encoded_batches = Vec::<Vec<u8>>::decode(&mut payload)
                        .expect("decode finalized batch chunk SCALE");
                    assert!(payload.is_empty(), "batch chunk has trailing SCALE bytes");
                    for encoded_batch in encoded_batches {
                        let batch = decode_batch(&encoded_batch);
                        if batch.block.number == 0 {
                            assert_eq!(
                                format_hash(batch.genesis_hash),
                                genesis_hash.as_deref().expect("known genesis hash")
                            );
                            assert_eq!(batch.block.hash, batch.genesis_hash);
                            assert_eq!(batch.block.parent_hash, [0; 32]);
                            assert!(batch.block.header_raw_bytes.is_some());
                            assert!(batch.extrinsics.is_empty());
                            assert!(!batch.storage_changes.is_empty());
                            assert!(batch.storage_changes.iter().all(|change| {
                                !change.raw_key.is_empty()
                                    && change.old_raw_value.is_none()
                                    && change.new_raw_value.is_some()
                                    && matches!(change.operation, StorageOperation::Upsert)
                            }));
                            received_finalized_genesis = true;
                        }
                    }
                    respond(
                        request.stream,
                        "200 OK",
                        "application/json",
                        b"{\"status\":\"accepted\"}",
                    );
                }
                "/best-chain" => {
                    assert_eq!(
                        request.content_type.as_deref(),
                        Some("application/octet-stream")
                    );
                    let mut payload = request.body.as_slice();
                    let update = BestChainUpdate::decode(&mut payload)
                        .expect("decode BEST chain update SCALE");
                    assert!(payload.is_empty(), "BEST update has trailing SCALE bytes");
                    assert_eq!(update.fixture_format_version, 1);
                    assert_eq!(update.chain_id, "gtest_local");
                    assert_eq!(update.from.number, 0);
                    assert_eq!(update.from.hash, update.genesis_hash);
                    assert!(update.to.number > 0);
                    assert!(update.retracted.is_empty());
                    assert_eq!(update.enacted_batches.len(), update.to.number as usize);
                    let mut parent = update.from.hash;
                    for encoded_batch in &update.enacted_batches {
                        let batch = decode_batch(encoded_batch);
                        assert_eq!(batch.block.parent_hash, parent);
                        parent = batch.block.hash;
                    }
                    assert_eq!(parent, update.to.hash);
                    received_best = true;
                    respond(
                        request.stream,
                        "200 OK",
                        "application/json",
                        b"{\"status\":\"accepted\"}",
                    );
                }
                path => panic!("unexpected sink path {path}"),
            }
        }
        batch_tx.send(()).expect("report consumed batch");
    });

    let child = Command::new(env!("CARGO_BIN_EXE_duniter"))
        .args([
            "--tmp",
            "--chain",
            "gtest_local",
            "--validator",
            "--unsafe-force-node-key-generation",
            "--sealing",
            "manual",
            "--rpc-port",
            &rpc_address.port().to_string(),
            "--rpc-methods",
            "unsafe",
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

    create_manual_block(rpc_address);

    batch_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("indexer did not consume the genesis batch in time");
    server.join().expect("indexer mock thread");
}

fn create_manual_block(address: std::net::SocketAddr) {
    let body =
        br#"{"jsonrpc":"2.0","id":1,"method":"engine_createBlock","params":[true,true,null]}"#;
    for _ in 0..100 {
        if let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(200)) {
            let request = format!(
                "POST / HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            if stream.write_all(request.as_bytes()).is_ok()
                && stream.write_all(body).is_ok()
                && stream.flush().is_ok()
            {
                let mut response = Vec::new();
                if stream.read_to_end(&mut response).is_ok()
                    && String::from_utf8_lossy(&response).contains("\"result\"")
                {
                    return;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("manual-seal RPC did not create a block in time");
}

fn decode_batch(encoded: &[u8]) -> FinalizedBatch {
    let mut input = encoded;
    let batch = FinalizedBatch::decode(&mut input).expect("decode batch SCALE");
    assert!(input.is_empty(), "batch has trailing SCALE bytes");
    assert_eq!(batch.fixture_format_version, 1);
    assert_eq!(batch.chain_id, "gtest_local");
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
    batch
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
