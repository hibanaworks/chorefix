#[hibana_runtime::main]
async fn main() -> choreofix::shell::ShellResult<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or_else(|| {
        choreofix::shell::ShellError::message("usage: writing-wasi GUEST.wasm 2E|2M|2H")
    })?;
    let level = args
        .next()
        .ok_or_else(|| choreofix::shell::ShellError::message("missing writing task"))?;
    let module = std::fs::read(path)?;
    choreofix::tasks::writing::choreofs::run_stdio(&module, &level).await
}
