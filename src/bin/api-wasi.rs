#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let module = std::fs::read(
        std::env::args()
            .nth(1)
            .ok_or("usage: api-wasi GUEST.wasm")?,
    )
    .map_err(|e| e.to_string())?;
    choreofix::tasks::api::choreofs::run_stdio(&module)
        .await
        .map_err(|e| e.to_string())
}
