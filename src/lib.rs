#![deny(unsafe_op_in_unsafe_fn)]

pub mod durability;
pub mod game;

#[cfg(any(test, target_os = "switch"))]
mod config;

#[cfg(any(test, target_os = "switch"))]
mod rolling_log;

#[cfg(target_os = "switch")]
mod diagnostics;

#[cfg(target_os = "switch")]
mod plugin;

#[cfg(target_os = "switch")]
#[skyline::main(name = "feth_better_durability")]
pub fn skyline_main() {
  plugin::install();
}
