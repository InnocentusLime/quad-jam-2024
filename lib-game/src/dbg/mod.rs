mod console;
#[cfg(not(target_family = "wasm"))]
mod animation_edit;

pub use console::*;
#[cfg(not(target_family = "wasm"))]
pub use animation_edit::*;