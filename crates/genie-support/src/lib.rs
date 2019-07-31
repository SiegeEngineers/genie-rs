//! Campaign files store multiple scenario files in one easily distributable chunk.
//!
//! genie-cpx can read and write campaign files using the Campaign and CampaignWriter structs,
//! respectively.

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
