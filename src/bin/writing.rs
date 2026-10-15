use hibana_tracefix::{bridge, tasks::writing};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let task = std::env::args().nth(1).expect("task: 2E, 2M, or 2H");
    let count = match task.as_str() {
        "2E" => 3,
        "2M" => 5,
        "2H" => 7,
        _ => panic!("unknown task"),
    };
    let agents = "ABCDEFG"
        .chars()
        .take(count)
        .map(|role| format!("RESEARCHER_{role}"))
        .collect::<Vec<_>>();
    let (domain, mut reader) = bridge::open(&agents);
    let result = {
        let run = async {
            match task.as_str() {
                "2E" => writing::run_2e(&domain).await,
                "2M" => writing::run_2m(&domain).await,
                "2H" => writing::run_2h(&domain).await,
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
