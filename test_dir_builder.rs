#[tokio::main]
async fn main() {
    let parent = "test_dir";
    #[cfg(unix)]
    {
        tokio::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)
            .await.unwrap();
    }
}
