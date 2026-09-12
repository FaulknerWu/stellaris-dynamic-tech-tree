#![forbid(unsafe_code)]

mod clausewitz;
mod error;

pub mod analysis;
pub mod game_data;
pub mod load_order;
pub mod localisation;
pub mod output;
pub mod paths;
pub mod save;

pub use error::{Error, Result};
