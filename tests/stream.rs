use tokio::process::Command;
use tokio::time::{timeout, Duration};
use warp::Filter;
use std::sync::{Arc, Mutex};
use futures_util::stream;

#[tokio::test]
async fn test_lc_stream() {
    // Mock SSE server
    let messages = Arc::new(Mutex::new(vec![]));
    let messages_clone = messages.clone();

    let routes = warp::path("sse")
        .map(move || {
            let mut messages = messages_clone.lock().unwrap();
            messages.push("data: counting\n\n".to_string());
            warp::sse::reply(warp::sse::keep_alive().stream(stream::iter(messages.clone().into_iter().map(|msg| Ok::<warp::sse::Event, std::convert::Infallible>(warp::sse::Event::default().data(msg))))))
        });

    let (_addr, server) = warp::serve(routes).bind_ephemeral(([127, 0, 0, 1], 3030));
    tokio::task::spawn(server);

    // Spawn the command
    let mut cmd = Command::new("lc")
        .arg("--stream")
        .arg("count to 20")
        .env("LC_PROVIDER_URL", "http://127.0.0.1:3030/sse")
        .spawn()
        .expect("Failed to spawn lc command");

    // Check output within 1 second
    let result = timeout(Duration::from_secs(1), async {
        let output = cmd.wait_with_output().await.expect("Failed to read stdout");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("counting"));
    }).await;

    assert!(result.is_ok(), "The operation timed out");
}
