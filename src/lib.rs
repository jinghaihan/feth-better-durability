#![deny(unsafe_op_in_unsafe_fn)]

pub mod durability;
pub mod game;

#[cfg(target_os = "switch")]
mod plugin;

#[cfg(target_os = "switch")]
#[skyline::main(name = "feth_infinite_weapon_durability")]
pub fn skyline_main() {
  plugin::install();
}
