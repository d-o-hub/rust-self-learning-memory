//! End-to-end CLI wiring for semantic episode search.
//!
//! Exercises the real binary against an OpenAI-compatible fixture endpoint:
//! the episode description and the query share no words, so only an actually
//! activated embedding provider can return the episode.
//!
//! Requires the `openai` feature (the custom provider speaks the OpenAI wire
//! format); declared as a `required-features` test in `Cargo.toml`.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use assert_cmd::Command;

/// Fixed 3-dimensional vector served for every input.
const FIXTURE_EMBEDDING: [f32; 3] = [1.0, 0.0, 0.0];

/// Minimal in-process OpenAI-compatible embeddings server.
struct FixtureServer {
    base_url: String,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl FixtureServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        listener
            .set_nonblocking(true)
            .expect("non-blocking listener");
        let addr = listener.local_addr().expect("fixture address");
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            while !stop_flag.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = handle_connection(stream);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            base_url: format!("http://{addr}/v1"),
            stop,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Wake the blocked accept loop so the thread observes the flag.
        let _ = TcpStream::connect(
            self.base_url
                .trim_start_matches("http://")
                .split('/')
                .next()
                .unwrap_or_default(),
        );
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(header_end) = find_header_end(&buffer) {
            let headers = String::from_utf8_lossy(&buffer[..header_end]).to_string();
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);

            let body_start = header_end + 4;
            while buffer.len() < body_start + content_length {
                let read = stream.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }

            let body = &buffer[body_start..(body_start + content_length).min(buffer.len())];
            let inputs = embedding_input_count(body);
            let response = embedding_response(inputs);
            write_response(&mut stream, &response)?;
            return Ok(());
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn embedding_input_count(body: &[u8]) -> usize {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return 1;
    };
    match &value["input"] {
        serde_json::Value::Array(items) => items.len().max(1),
        _ => 1,
    }
}

fn embedding_response(count: usize) -> String {
    let data: Vec<serde_json::Value> = (0..count)
        .map(|index| {
            serde_json::json!({
                "object": "embedding",
                "index": index,
                "embedding": FIXTURE_EMBEDDING,
            })
        })
        .collect();

    serde_json::json!({
        "object": "list",
        "data": data,
        "model": "text-embedding-3-small",
        "usage": {"prompt_tokens": count, "total_tokens": count},
    })
    .to_string()
}

fn write_response(stream: &mut TcpStream, body: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn write_config(dir: &Path, base_url: &str) -> PathBuf {
    let redb_path = dir.join("memory.redb");
    let config = format!(
        r#"[database]
redb_path = "{redb}"

[embeddings]
enabled = true
provider = "custom"
model = "text-embedding-3-small"
dimension = 3
base_url = "{base_url}"
similarity_threshold = 0.0
"#,
        redb = redb_path.display(),
        base_url = base_url,
    );
    let config_path = dir.join("config.toml");
    std::fs::write(&config_path, config).expect("write config");
    config_path
}

fn run(config: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("do-memory-cli")
        .expect("built CLI binary")
        .arg("--config")
        .arg(config)
        .args(args)
        .output()
        .expect("run CLI")
}

fn assert_success(output: &std::process::Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed with {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn semantic_search_returns_episode_without_word_overlap() {
    let server = FixtureServer::start();
    let dir = tempfile::tempdir().expect("tempdir");
    let config = write_config(dir.path(), server.base_url());

    let created = run(
        &config,
        &[
            "--format",
            "json",
            "episode",
            "create",
            "--task",
            "Deploy authentication service",
        ],
    );
    assert_success(&created, "episode create");
    let created_json: serde_json::Value =
        serde_json::from_slice(&created.stdout).expect("create output is JSON");
    let episode_id = created_json["episode_id"]
        .as_str()
        .expect("episode_id field")
        .to_string();

    let completed = run(
        &config,
        &[
            "--format",
            "json",
            "episode",
            "complete",
            &episode_id,
            "success",
        ],
    );
    assert_success(&completed, "episode complete");

    let rebuilt = run(&config, &["embedding", "rebuild"]);
    assert_success(&rebuilt, "embedding rebuild");

    // "login security" shares no token with "Deploy authentication service";
    // a keyword fallback would return nothing here.
    let searched = run(
        &config,
        &[
            "--format",
            "json",
            "episode",
            "search",
            "login security",
            "--semantic",
        ],
    );
    assert_success(&searched, "episode search --semantic");

    let results: serde_json::Value =
        serde_json::from_slice(&searched.stdout).expect("search output is JSON");
    let episodes = results["episodes"]
        .as_array()
        .expect("search output has an episodes array");
    assert!(
        episodes
            .iter()
            .any(|episode| episode["episode_id"] == episode_id.as_str()),
        "semantic search did not return the embedded episode: {results}"
    );

    // `episode list --semantic-search` uses the same activation path.
    let listed = run(
        &config,
        &[
            "--format",
            "json",
            "episode",
            "list",
            "--semantic-search",
            "login security",
        ],
    );
    assert_success(&listed, "episode list --semantic-search");
    let listed: serde_json::Value =
        serde_json::from_slice(&listed.stdout).expect("list output is JSON");
    assert!(
        listed["episodes"]
            .as_array()
            .expect("episodes array")
            .iter()
            .any(|episode| episode["episode_id"] == episode_id.as_str()),
        "semantic list did not return the embedded episode: {listed}"
    );
}

#[test]
fn semantic_search_rejects_lexical_modes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = write_config(dir.path(), "http://127.0.0.1:9/v1");

    let output = run(
        &config,
        &["episode", "search", "query", "--semantic", "--fuzzy"],
    );

    assert_eq!(
        output.status.code(),
        Some(2),
        "clap must reject the combination before running: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--semantic") && stderr.contains("--fuzzy"),
        "unexpected clap error: {stderr}"
    );
    assert!(
        stderr.contains("cannot be used with"),
        "unexpected clap error: {stderr}"
    );
}
