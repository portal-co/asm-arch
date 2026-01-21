//! Shim crate containing x86-64 -> arch translation shims for other arch crates.
#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;

pub mod aarch64;
pub mod riscv64;
