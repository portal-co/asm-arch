//! RISC-V 64-bit (RV64) assembly types and output generation.
//!
//! This crate provides types and traits for working with RISC-V 64-bit assembly code,
//! including register representations, condition codes, and instruction output.
//!
//! # Features
//!
//! - `alloc`: Enables heap allocation support for dynamic collections
//! - `x64_shim`: Enables x86-64 to RISC-V64 translation shim
//!
//! # Example
//!
//! ```ignore
//! use portal_solutions_asm_riscv64::{RiscV64Arch, out::WriterCore};
//!
//! let arch = RiscV64Arch::default();
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

/// 64-bit general-purpose register names (x0-x31).
/// x0 is hardwired to zero, x1 is return address, x2 is stack pointer.
static REG_NAMES_64: &'static [&'static str; 32] = &[
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2",
    "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5",
    "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
    "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

/// 32-bit general-purpose register names (same as 64-bit in RISC-V).
#[allow(dead_code)]
static REG_NAMES_32: &'static [&'static str; 32] = &[
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2",
    "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5",
    "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
    "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

/// Floating-point register names (f0-f31).
static FREG_NAMES: &'static [&'static str; 32] = &[
    "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7",
    "fs0", "fs1", "fa0", "fa1", "fa2", "fa3", "fa4", "fa5",
    "fa6", "fa7", "fs2", "fs3", "fs4", "fs5", "fs6", "fs7",
    "fs8", "fs9", "fs10", "fs11", "ft8", "ft9", "ft10", "ft11",
];

/// Register class for display formatting.
///
/// Determines whether registers are formatted as general-purpose registers (GPR)
/// or floating-point registers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[non_exhaustive]
pub enum RegisterClass {
    /// General-purpose register (x0-x31, or their ABI names).
    #[default]
    Gpr,
    /// Floating-point register (f0-f31).
    Fp,
}

/// Display options for formatting assembly operands.
///
/// This struct combines architecture configuration with register class selection
/// to control how operands are displayed in assembly output.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub struct DisplayOpts {
    /// The target architecture configuration.
    pub arch: RiscV64Arch,
    /// The register class for display.
    pub reg_class: RegisterClass,
}

impl DisplayOpts {
    /// Creates display options with the given architecture and default register class.
    pub fn new(arch: RiscV64Arch) -> Self {
        Self {
            arch,
            reg_class: Default::default(),
        }
    }
    /// Creates display options with the given architecture and register class.
    pub fn with_reg_class(arch: RiscV64Arch, reg_class: RegisterClass) -> Self {
        Self { arch, reg_class }
    }
}

impl Default for DisplayOpts {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl From<RiscV64Arch> for DisplayOpts {
    fn from(arch: RiscV64Arch) -> Self {
        Self::new(arch)
    }
}

/// RISC-V 64-bit architecture configuration.
///
/// This struct holds configuration options for the RISC-V 64-bit architecture,
/// such as available extensions (I, M, A, F, D, C, etc.).
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct RiscV64Arch {
    /// Whether the M extension (integer multiplication/division) is enabled.
    pub m_extension: bool,
    /// Whether the A extension (atomic operations) is enabled.
    pub a_extension: bool,
    /// Whether the F extension (single-precision floating-point) is enabled.
    pub f_extension: bool,
    /// Whether the D extension (double-precision floating-point) is enabled.
    pub d_extension: bool,
    /// Whether the C extension (compressed instructions) is enabled.
    pub c_extension: bool,
}

impl RiscV64Arch {
    /// Creates a configuration with I, M, F, D extensions (RV64IMFD).
    pub fn rv64imfd() -> Self {
        Self {
            m_extension: true,
            a_extension: false,
            f_extension: true,
            d_extension: true,
            c_extension: false,
        }
    }
    
    /// Creates a configuration with all common extensions (RV64GC = IMAFD + C).
    pub fn rv64gc() -> Self {
        Self {
            m_extension: true,
            a_extension: true,
            f_extension: true,
            d_extension: true,
            c_extension: true,
        }
    }
}

/// Options for formatting register names.
///
/// Controls how registers are displayed, including the target architecture,
/// the operand size, and the register class (GPR vs FP).
#[derive(Clone)]
#[non_exhaustive]
pub struct RegFormatOpts {
    /// The target architecture configuration.
    pub arch: RiscV64Arch,
    /// The operand size for register formatting.
    pub size: MemorySize,
    /// The register class for display.
    pub reg_class: RegisterClass,
}

impl RegFormatOpts {
    /// Creates formatting options with the given architecture and default size.
    pub fn default_with_arch(arch: RiscV64Arch) -> Self {
        Self::default_with_arch_and_size(arch, Default::default())
    }
    /// Creates formatting options with the given architecture and size.
    pub fn default_with_arch_and_size(arch: RiscV64Arch, size: MemorySize) -> Self {
        Self {
            arch,
            size,
            reg_class: Default::default(),
        }
    }
    /// Creates formatting options with the given architecture, size, and register class.
    pub fn with_reg_class(arch: RiscV64Arch, size: MemorySize, reg_class: RegisterClass) -> Self {
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
/// Desugaring wrapper for complex memory operands.
pub mod desugar;

#[cfg(feature = "x64_shim")]
pub mod shim;

/// Register allocation integration module (gated by `regalloc-integration` feature).
#[cfg(feature = "regalloc-integration")]
pub mod regalloc;

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use portal_pc_asm_common::types::reg::Reg;
    use crate::reg::RiscV64Reg;
    extern crate alloc;
    use alloc::string::String;
    use alloc::format;
    
    #[test]
    fn test_register_display() {
        let cfg = RiscV64Arch::default();
        let reg0 = Reg(0);
        let reg1 = Reg(1);
        let reg2 = Reg(2);
        let reg10 = Reg(10);
        
        // Test GPR display (64-bit)
        let gpr_opts = RegFormatOpts::default_with_arch(cfg);
        let zero = format!("{}", RiscV64Reg::display(&reg0, gpr_opts.clone()));
        let ra = format!("{}", RiscV64Reg::display(&reg1, gpr_opts.clone()));
        let sp = format!("{}", RiscV64Reg::display(&reg2, gpr_opts.clone()));
        let a0 = format!("{}", RiscV64Reg::display(&reg10, gpr_opts));
        
        assert_eq!(zero, "zero");
        assert_eq!(ra, "ra");
        assert_eq!(sp, "sp");
        assert_eq!(a0, "a0");
        
        // Test FP display
        let fp_opts = RegFormatOpts::with_reg_class(cfg, MemorySize::_64, RegisterClass::Fp);
        let ft0 = format!("{}", RiscV64Reg::display(&reg0, fp_opts.clone()));
        let fa0 = format!("{}", RiscV64Reg::display(&reg10, fp_opts));
        
        assert_eq!(ft0, "ft0");
        assert_eq!(fa0, "fa0");
    }
}

/// RISC-V condition codes for conditional branches.
///
/// RISC-V uses separate branch instructions for different conditions,
/// unlike x86 which uses condition codes with flags. This enum represents
/// the comparison type for branch instructions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum ConditionCode {
    /// Equal (BEQ).
    EQ,
    /// Not equal (BNE).
    NE,
    /// Less than, signed (BLT).
    LT,
    /// Greater than or equal, signed (BGE).
    GE,
    /// Less than, unsigned (BLTU).
    LTU,
    /// Greater than or equal, unsigned (BGEU).
    GEU,
    /// Greater than, signed (BGT - pseudo-instruction).
    GT,
    /// Less than or equal, signed (BLE - pseudo-instruction).
    LE,
    /// Greater than, unsigned (BGTU - pseudo-instruction).
    GTU,
    /// Less than or equal, unsigned (BLEU - pseudo-instruction).
    LEU,
}

impl Display for ConditionCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConditionCode::EQ => write!(f, "eq"),
            ConditionCode::NE => write!(f, "ne"),
            ConditionCode::LT => write!(f, "lt"),
            ConditionCode::GE => write!(f, "ge"),
            ConditionCode::LTU => write!(f, "ltu"),
            ConditionCode::GEU => write!(f, "geu"),
            ConditionCode::GT => write!(f, "gt"),
            ConditionCode::LE => write!(f, "le"),
            ConditionCode::GTU => write!(f, "gtu"),
            ConditionCode::LEU => write!(f, "leu"),
        }
    }
}
