use choreofix::{bridge, tasks::api};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    assert_eq!(std::env::args().nth(1).as_deref(), Some("8E"), "task: 8E");
    let agents = ["BACKEND_DEV", "DB_ADMIN", "TESTER"].map(String::from);
    let (domain, mut reader) = bridge::open(&agents);
    let result = tokio::select! {
        result = api::run_8e(&domain) => result.map_err(|e| format!("{e:?}")),
        result = reader.run(&domain) => result,
    };
    domain.report(result).await.expect("write run result");
}
