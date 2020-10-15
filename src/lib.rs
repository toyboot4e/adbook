//! adbook

// Globally importing `info!`, `warn!`, etc.
#[macro_use]
extern crate log;

// Globally importing `anyhow!`, `bail!` and `ensure!`
#[macro_use]
extern crate anyhow;

pub mod book;
pub mod cli;
pub mod config;
