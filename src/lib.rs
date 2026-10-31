#![forbid(long_running_const_eval)]

mod imp;
pub mod tasks;

pub use imp::domain::RunError;

pub mod shell;
