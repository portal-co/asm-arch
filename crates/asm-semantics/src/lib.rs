#![no_std]
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub extern crate alloc;
#[doc(hidden)]
pub use core;

use alloc::vec::Vec;
use portal_pc_asm_common::types::{Arith, Cmp, reg::Reg};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Arg {
    FixedReg(Reg),
    Slot(u8),
    Previous(u32),
    Lit(u64),
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Semantic {
    pub wrapped: Vec<(Arg, Val)>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Val {
    Bin {
        left: Arg,
        op: Arith,
        right: Arg,
    },
    Jmp {
        target: Arg,
    },
    JmpCompare {
        target: Arg,
        left: Arg,
        op: Cmp,
        right: Arg,
    },
}
