#![allow(unused_variables)]

mod error;
pub mod state;

pub use crate::error::ContractError;

#[cfg(test)]
mod tests;
