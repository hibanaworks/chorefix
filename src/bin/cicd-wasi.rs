#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| choreofix::shell::ShellError::message("usage: cicd-wasi GUEST.wasm"))?;
    let module = std::fs::read(path)?;
    choreofix::tasks::cicd::choreofs::run_stdio(&module).await
}
