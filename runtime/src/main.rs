#[tokio::main]
async fn main() {
    runtime::start_server(
        "..\\models".to_string(),
        "..\\whisper\\whisper-cli".to_string(),
    )
    .await;
}
