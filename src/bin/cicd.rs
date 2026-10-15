use choreofix::{bridge, tasks::cicd};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    assert_eq!(std::env::args().nth(1).as_deref(), Some("16E"), "task: 16E");
    let agents = ["DEVELOPER", "TESTER", "DEPLOYER"].map(String::from);
    let (domain, mut reader) = bridge::open(&agents);
    let result = tokio::select! {
        result = cicd::run_16e(&domain) => result.map_err(|e| format!("{e:?}")),
        result = reader.run(&domain) => result,
    };
    domain.report(result).await.expect("write run result");
}
