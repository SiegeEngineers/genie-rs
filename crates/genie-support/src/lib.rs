//! Internal utilities for genie-rs modules.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(unused)]
#![allow(missing_docs)]

mod ids;
mod macros;
mod map_into;
mod read;
#[cfg(feature = "strings")]
mod strings;
mod versions;

pub use ids::*;
pub use macros::*;
pub use map_into::*;
pub use read::*;
#[cfg(feature = "strings")]
pub use strings::*;
pub use versions::*;
