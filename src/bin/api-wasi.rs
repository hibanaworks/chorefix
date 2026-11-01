#[hibana_runtime::main]
async fn main() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: api-wasi GUEST.wasm")?;
    if path == "--projection" {
        println!("{}", choreofix::tasks::api::choreofs::projection());
        return Ok(());
    }
    let module = std::fs::read(path).map_err(|e| e.to_string())?;
    choreofix::tasks::api::choreofs::run_stdio(&module)
        .await
        .map_err(|e| e.to_string())
}
