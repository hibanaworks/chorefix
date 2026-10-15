#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let module = std::fs::read(
        args.next()
            .ok_or("usage: philosophers-wasi GUEST.wasm 9E|9M|9H")?,
    )
    .map_err(|error| error.to_string())?;
    let task = args.next().ok_or("missing dining task")?;
    choreofix::tasks::philosophers::choreofs::run_stdio(&module, &task)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
