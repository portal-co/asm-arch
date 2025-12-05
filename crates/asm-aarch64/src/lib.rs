#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[doc(hidden)]
pub mod __ {
    pub use core;
}
#[cfg(feature = "x64_shim")]
pub mod x64_shim;

pub mod out;