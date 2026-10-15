use futures::future::{Either, select};
use hibana_tracefix::{bridge, tasks::philosophers};

fn main() {
    let task = std::env::args().nth(1).expect("task: 9E, 9M, or 9H");
    let count = match task.as_str() {
        "9E" => 3,
        "9M" => 5,
        "9H" => 7,
        _ => panic!("unknown task"),
    };
    let agents = "ABCDEFG"
        .chars()
        .take(count)
        .map(|role| format!("PHILOSOPHER_{role}"))
        .collect::<Vec<_>>();
    let (domain, mut reader) = bridge::open(&agents);
    let result = futures::executor::block_on(async {
        let run = async {
            match task.as_str() {
                "9E" => philosophers::run_9e(&domain).await,
                "9M" => philosophers::run_9m(&domain).await,
                "9H" => philosophers::run_9h(&domain).await,
                _ => unreachable!(),
            }
            .map_err(|error| format!("{error:?}"))
        };
        match select(Box::pin(run), Box::pin(reader.run(&domain))).await {
            Either::Left((result, _)) | Either::Right((result, _)) => result,
        }
    });
    bridge::report(result).expect("write run result");
}
