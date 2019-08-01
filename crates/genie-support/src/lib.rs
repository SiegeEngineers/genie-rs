//! Some internal utilities for genie-rs modules.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

mod macros;
mod map_into;

pub use macros::*;
pub use map_into::*;
