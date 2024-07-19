//! FFI Bindings for [quickjs](https://bellard.org/quickjs/),
//! a Javascript engine.
//! See the [quickjs](https://crates.io/crates/quickjs) crate for a high-level
//! wrapper.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(unused)]

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
// include!("static-functions.rs");

pub mod common;
mod context;
mod data;
pub mod ffi;
pub mod function;
mod runtime;

pub use context::*;
pub use data::*;
pub use runtime::*;

