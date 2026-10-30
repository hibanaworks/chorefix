#![forbid(long_running_const_eval)]

mod imp;
mod resource;
pub mod tasks;

pub use imp::domain::{Domain, RunError, Work, WorkRequest};

pub mod shell;
