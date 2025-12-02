//! x86-64 assembly types and output generation.
//!
//! This crate provides types and traits for working with x86-64 assembly code,
//! including register representations, condition codes, and instruction output.
//!
//! # Features
//!
//! - `alloc`: Enables heap allocation support for dynamic collections
//!
//! # Example
//!
//! ```ignore
//! use portal_solutions_asm_x86_64::{X64Arch, ConditionCode, out::WriterCore};
//!
//! let arch = X64Arch::default();
//! ```

#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[doc(hidden)]
pub mod __ {
    pub use core;
}
use core::fmt::Display;

use portal_pc_asm_common::types::mem::MemorySize;

/// 64-bit register names (rax, rcx, rdx, rbx, rsp, rbp, rsi, rdi).
static REG_NAMES: &'static [&'static str; 8] =
    &["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi"];
/// 32-bit register names (eax, ecx, edx, ebx, esp, ebp, esi, edi).
static REG_NAMES_32: &'static [&'static str; 8] =
    &["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];
/// 16-bit register names (ax, cx, dx, bx, sp, bp, si, di).
static REG_NAMES_16: &'static [&'static str; 8] = &["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];
/// 8-bit register names (al, cl, dl, bl, spl, bpl, sil, dil).
static REG_NAMES_8: &'static [&'static str; 8] =
    &["al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil"];
/// XMM register names (xmm0 through xmm15).
static XMM_REG_NAMES: &'static [&'static str; 16] = &[
    "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
    "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15",
];

/// Register class for display formatting.
///
/// Determines whether registers are formatted as general-purpose registers (GPR)
/// or XMM registers for floating-point operations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[non_exhaustive]
pub enum RegisterClass {
    /// General-purpose register (rax, rbx, etc.).
    #[default]
    Gpr,
    /// XMM register for floating-point/SIMD operations (xmm0, xmm1, etc.).
    Xmm,
}

/// Display options for formatting assembly operands.
///
/// This struct combines architecture configuration with register class selection
/// to control how operands are displayed in assembly output.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub struct DisplayOpts {
    /// The target architecture configuration.
    pub arch: X64Arch,
    /// The register class for display.
    pub reg_class: RegisterClass,
}
impl DisplayOpts {
    /// Creates display options with the given architecture and default register class.
    pub fn new(arch: X64Arch) -> Self {
        Self {
            arch,
            reg_class: Default::default(),
        }
    }
    /// Creates display options with the given architecture and register class.
    pub fn with_reg_class(arch: X64Arch, reg_class: RegisterClass) -> Self {
        Self { arch, reg_class }
    }
}
impl Default for DisplayOpts {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
impl From<X64Arch> for DisplayOpts {
    fn from(arch: X64Arch) -> Self {
        Self::new(arch)
    }
}

/// x86-64 architecture configuration.
///
/// This struct holds configuration options for the x86-64 architecture,
/// such as whether APX (Advanced Performance Extensions) is enabled.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct X64Arch {
    /// Whether APX (Advanced Performance Extensions) is enabled.
    /// When enabled, 32 general-purpose registers are available instead of 16.
    pub apx: bool,
}

/// Options for formatting register names.
///
/// Controls how registers are displayed, including the target architecture,
/// the operand size, and the register class (GPR vs XMM).
#[derive(Clone)]
#[non_exhaustive]
pub struct RegFormatOpts {
    /// The target architecture configuration.
    pub arch: X64Arch,
    /// The operand size for register formatting.
    pub size: MemorySize,
    /// The register class for display.
    pub reg_class: RegisterClass,
}
impl RegFormatOpts {
    /// Creates formatting options with the given architecture and default size.
    pub fn default_with_arch(arch: X64Arch) -> Self {
        Self::default_with_arch_and_size(arch, Default::default())
    }
    /// Creates formatting options with the given architecture and size.
    pub fn default_with_arch_and_size(arch: X64Arch, size: MemorySize) -> Self {
        Self {
            arch,
            size,
            reg_class: Default::default(),
        }
    }
    /// Creates formatting options with the given architecture, size, and register class.
    pub fn with_reg_class(arch: X64Arch, size: MemorySize, reg_class: RegisterClass) -> Self {
        Self {
            arch,
            size,
            reg_class,
        }
    }
}
impl Default for RegFormatOpts {
    fn default() -> Self {
        Self::default_with_arch(Default::default())
    }
}

/// Instruction output generation module.
pub mod out;
/// Register handling and formatting module.
pub mod reg;

/// x86-64 condition codes for conditional instructions.
///
/// These codes are used with conditional jumps (jcc), conditional moves (cmovcc),
/// and other conditional instructions. Each code tests specific CPU flags.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum ConditionCode {
    /// Overflow (OF=1).
    O,
    /// No overflow (OF=0).
    NO,
    /// Below / Carry (CF=1). Unsigned less than.
    B,
    /// Not below / No carry (CF=0). Unsigned greater than or equal.
    NB,
    /// Equal / Zero (ZF=1).
    E,
    /// Not equal / Not zero (ZF=0).
    NE,
    /// Not above (CF=1 or ZF=1). Unsigned less than or equal.
    NA,
    /// Above (CF=0 and ZF=0). Unsigned greater than.
    A,
    /// Sign (SF=1). Negative.
    S,
    /// No sign (SF=0). Non-negative.
    NS,
    /// Parity (PF=1). Even parity.
    P,
    /// No parity (PF=0). Odd parity.
    NP,
    /// Less (SF≠OF). Signed less than.
    L,
    /// Not less (SF=OF). Signed greater than or equal.
    NL,
    /// Not greater (ZF=1 or SF≠OF). Signed less than or equal.
    NG,
    /// Greater (ZF=0 and SF=OF). Signed greater than.
    G,
}
impl Display for ConditionCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConditionCode::O => write!(f, "o"),
            ConditionCode::NO => write!(f, "no"),
            ConditionCode::B => write!(f, "b"),
            ConditionCode::NB => write!(f, "nb"),
            ConditionCode::E => write!(f, "e"),
            ConditionCode::NE => write!(f, "ne"),
            ConditionCode::NA => write!(f, "na"),
            ConditionCode::A => write!(f, "a"),
            ConditionCode::S => write!(f, "s"),
            ConditionCode::NS => write!(f, "ns"),
            ConditionCode::P => write!(f, "p"),
            ConditionCode::NP => write!(f, "np"),
            ConditionCode::L => write!(f, "l"),
            ConditionCode::NL => write!(f, "nl"),
            ConditionCode::NG => write!(f, "ng"),
            ConditionCode::G => write!(f, "g"),
        }
    }
}
