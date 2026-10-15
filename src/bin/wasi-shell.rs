#[tokio::main(flavor = "current_thread")]
async fn main() {
    let result = async {
        let path = std::env::args()
            .nth(1)
            .ok_or_else(|| choreofix::shell::ShellError::message("usage: wasi-shell GUEST.wasm"))?;
        let module = std::fs::read(path)?;
        choreofix::shell::run(&module).await
    }
    .await;
    match result {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
