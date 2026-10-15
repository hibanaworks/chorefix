use choreofix::{bridge, tasks::codebase};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let task = std::env::args().nth(1).expect("task: 1E, 1M, or 1H");
    let count = match task.as_str() {
        "1E" => 3,
        "1M" => 5,
        "1H" => 7,
        _ => panic!("unknown task"),
    };
    let agents = "ABCDEFG"
        .chars()
        .take(count)
        .map(|role| format!("DEVELOPER_{role}"))
        .collect::<Vec<_>>();
    let (domain, mut reader) = bridge::open(&agents);
    let result = {
        let run = async {
            match task.as_str() {
                "1E" => codebase::run_1e(&domain).await,
                "1M" => codebase::run_1m(&domain).await,
                "1H" => codebase::run_1h(&domain).await,
                _ => unreachable!(),
            }
            .map_err(|error| format!("{error:?}"))
        };
        tokio::select! {
            result = run => result,
            result = reader.run(&domain) => result,
        }
    };
    domain.report(result).await.expect("write run result");
}
