#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: review-wasi GUEST.wasm"))?;
    if path == "--projection" {
        println!("{}", choreofix::tasks::review::choreofs::projection());
        return Ok(());
    }
    let module = std::fs::read(path)?;
    choreofix::tasks::review::choreofs::run_stdio(&module).await
}
