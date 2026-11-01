#[hibana_runtime::main]
async fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: philosophers-wasi GUEST.wasm 9E|9M|9H")?;
    let task = args.next().ok_or("missing dining task")?;
    if path == "--projection" {
        let projection = choreofix::tasks::philosophers::choreofs::projection(&task)
            .map_err(|error| error.to_string())?;
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path).map_err(|error| error.to_string())?;
    choreofix::tasks::philosophers::choreofs::run_stdio(&module, &task)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
