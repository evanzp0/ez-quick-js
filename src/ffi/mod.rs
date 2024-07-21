mod static_functions;
mod bindings;

pub(crate) use bindings::*;
pub(crate) use static_functions::*;

pub use bindings::{JSCFunction, JSCFunctionMagic, JSCFunctionData};
