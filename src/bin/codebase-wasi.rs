#[tokio::main(flavor = "current_thread")]
async fn main() {
    let result = async {
        let mut args = std::env::args().skip(1);
        let path = args.next().ok_or_else(|| {
            choreofix::shell::ShellError::message("usage: codebase-wasi GUEST.wasm 1E|1M")
        })?;
        let level = args
            .next()
            .ok_or_else(|| choreofix::shell::ShellError::message("missing codebase task"))?;
        let module = std::fs::read(path)?;
        choreofix::tasks::codebase::choreofs::run_stdio(&module, &level).await
    }
    .await;
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
