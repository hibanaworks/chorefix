use choreofix::{bridge, tasks::review};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    assert_eq!(std::env::args().nth(1).as_deref(), Some("6E"), "task: 6E");
    let agents = ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"].map(String::from);
    let (domain, mut reader) = bridge::open(&agents);
    let result = tokio::select! {
        result=review::run_6e(&domain)=>result.map_err(|e|format!("{e:?}")),
        result=reader.run(&domain)=>result,
    };
    domain.report(result).await.expect("write run result");
}
