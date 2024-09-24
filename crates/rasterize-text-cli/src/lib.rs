#![doc = include_str!("../../../README.md")]

pub mod cli;
pub mod verbosity;

#[doc(inline)]
pub use crate::cli::Cli;
pub use crate::verbosity::Verbosity;