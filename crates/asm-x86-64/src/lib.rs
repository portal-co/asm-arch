#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[doc(hidden)]
pub mod __{
    pub use core;
}
use portal_pc_asm_common::types::mem::MemorySize;

static REG_NAMES: &'static [&'static str; 8] =
    &["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi"];
static REG_NAMES_32: &'static [&'static str; 8] =
    &["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];
static REG_NAMES_16: &'static [&'static str; 8] = &["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];
static REG_NAMES_8: &'static [&'static str; 8] =
    &["al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil"];
    #[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct X64Arch {
    pub apx: bool,
}
#[derive(Clone)]
#[non_exhaustive]
pub struct RegFormatOpts {
    pub arch: X64Arch,
    pub size: MemorySize,
}
impl RegFormatOpts {
    pub fn default_with_arch(arch: X64Arch) -> Self {
        Self::default_with_arch_and_size(arch, Default::default())
    }
    pub fn default_with_arch_and_size(arch: X64Arch, size: MemorySize) -> Self {
        Self { arch, size }
    }
}
impl Default for RegFormatOpts {
    fn default() -> Self {
        Self::default_with_arch(Default::default())
    }
}
pub mod out;
pub mod reg;