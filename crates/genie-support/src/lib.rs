//! Internal utilities for genie-rs modules.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

mod ids;
mod macros;
mod map_into;
mod read;
#[cfg(feature = "strings")]
mod strings;

pub use ids::*;
pub use macros::*;
pub use map_into::*;
pub use read::*;
#[cfg(feature = "strings")]
pub use strings::*;
