#![forbid(unsafe_code)]

pub mod condition;
pub mod empire;
pub mod graph;
pub mod render;
pub mod technology;

mod error;

#[cfg(test)]
mod test_support;

pub use error::{Error, Result};
