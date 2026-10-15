#[tokio::main(flavor = "current_thread")]
async fn main() {
    let result = async {
        let path = std::env::args().nth(1).ok_or_else(|| {
            choreofix::shell::ShellError::message("usage: review-wasi GUEST.wasm")
        })?;
        let module = std::fs::read(path)?;
        choreofix::tasks::review::choreofs::run_stdio(&module).await
    }
    .await;
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
