#[path = "../../../src/imp/domain.rs"]
pub mod domain_types;
pub use domain_types::RunError;
mod imp;
#[path = "../../../src/tasks/research_report/mod.rs"]
pub mod research_report;
