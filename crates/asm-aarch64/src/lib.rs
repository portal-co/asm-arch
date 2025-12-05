//! AArch64 (ARM64) assembly types and output generation.
//!
//! This crate provides types and traits for working with AArch64 assembly code,
//! including register representations, condition codes, and instruction output.
//!
//! # Features
//!
//! - `alloc`: Enables heap allocation support for dynamic collections
//! - `x64_shim`: Enables x86-64 to AArch64 translation shim
//!
//! # Example
//!
//! ```ignore
//! use portal_solutions_asm_aarch64::{AArch64Arch, ConditionCode, out::WriterCore};
//!
//! let arch = AArch64Arch::default();
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

/// 64-bit general-purpose register names (x0-x30, sp, xzr).
static REG_NAMES_64: &'static [&'static str; 32] = &[
    "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7",
    "x8", "x9", "x10", "x11", "x12", "x13", "x14", "x15",
    "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23",
    "x24", "x25", "x26", "x27", "x28", "x29", "x30", "sp",
];

/// 32-bit general-purpose register names (w0-w30, wsp, wzr).
static REG_NAMES_32: &'static [&'static str; 32] = &[
    "w0", "w1", "w2", "w3", "w4", "w5", "w6", "w7",
    "w8", "w9", "w10", "w11", "w12", "w13", "w14", "w15",
    "w16", "w17", "w18", "w19", "w20", "w21", "w22", "w23",
    "w24", "w25", "w26", "w27", "w28", "w29", "w30", "wsp",
];

/// SIMD/FP register names (v0-v31).
static VREG_NAMES: &'static [&'static str; 32] = &[
    "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7",
    "v8", "v9", "v10", "v11", "v12", "v13", "v14", "v15",
    "v16", "v17", "v18", "v19", "v20", "v21", "v22", "v23",
    "v24", "v25", "v26", "v27", "v28", "v29", "v30", "v31",
];

/// Register class for display formatting.
///
/// Determines whether registers are formatted as general-purpose registers (GPR)
/// or SIMD/FP registers for floating-point operations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[non_exhaustive]
pub enum RegisterClass {
    /// General-purpose register (x0, x1, etc.).
    #[default]
    Gpr,
    /// SIMD/FP register for floating-point/SIMD operations (v0, v1, etc.).
    Simd,
}

/// Display options for formatting assembly operands.
///
/// This struct combines architecture configuration with register class selection
/// to control how operands are displayed in assembly output.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub struct DisplayOpts {
    /// The target architecture configuration.
    pub arch: AArch64Arch,
    /// The register class for display.
    pub reg_class: RegisterClass,
}

impl DisplayOpts {
    /// Creates display options with the given architecture and default register class.
    pub fn new(arch: AArch64Arch) -> Self {
        Self {
            arch,
            reg_class: Default::default(),
        }
    }
    /// Creates display options with the given architecture and register class.
    pub fn with_reg_class(arch: AArch64Arch, reg_class: RegisterClass) -> Self {
        Self { arch, reg_class }
    }
}

impl Default for DisplayOpts {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl From<AArch64Arch> for DisplayOpts {
    fn from(arch: AArch64Arch) -> Self {
        Self::new(arch)
    }
}

/// AArch64 architecture configuration.
///
/// This struct holds configuration options for the AArch64 architecture,
/// such as available extensions.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct AArch64Arch {
    /// Placeholder for future architecture-specific options.
    _reserved: (),
}

/// Options for formatting register names.
///
/// Controls how registers are displayed, including the target architecture,
/// the operand size, and the register class (GPR vs SIMD).
#[derive(Clone)]
#[non_exhaustive]
pub struct RegFormatOpts {
    /// The target architecture configuration.
    pub arch: AArch64Arch,
    /// The operand size for register formatting.
    pub size: MemorySize,
    /// The register class for display.
    pub reg_class: RegisterClass,
}

impl RegFormatOpts {
    /// Creates formatting options with the given architecture and default size.
    pub fn default_with_arch(arch: AArch64Arch) -> Self {
        Self::default_with_arch_and_size(arch, Default::default())
    }
    /// Creates formatting options with the given architecture and size.
    pub fn default_with_arch_and_size(arch: AArch64Arch, size: MemorySize) -> Self {
        Self {
            arch,
            size,
            reg_class: Default::default(),
        }
    }
    /// Creates formatting options with the given architecture, size, and register class.
    pub fn with_reg_class(arch: AArch64Arch, size: MemorySize, reg_class: RegisterClass) -> Self {
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

#[cfg(feature = "x64_shim")]
pub mod shim;

/// Register allocation integration module (gated by `regalloc-integration` feature).
#[cfg(feature = "regalloc-integration")]
pub mod regalloc;

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use portal_pc_asm_common::types::reg::Reg;
    use crate::reg::AArch64Reg;
    extern crate alloc;
    use alloc::string::String;
    use alloc::format;
    
    #[test]
    fn test_register_display() {
        let cfg = AArch64Arch::default();
        let reg0 = Reg(0);
        let reg1 = Reg(1);
        let reg30 = Reg(30);
        let reg31 = Reg(31);
        
        // Test GPR display (64-bit)
        let gpr_opts = RegFormatOpts::default_with_arch(cfg);
        let x0 = format!("{}", AArch64Reg::display(&reg0, gpr_opts.clone()));
        let x1 = format!("{}", AArch64Reg::display(&reg1, gpr_opts.clone()));
        let x30 = format!("{}", AArch64Reg::display(&reg30, gpr_opts.clone()));
        let sp = format!("{}", AArch64Reg::display(&reg31, gpr_opts));
        
        assert_eq!(x0, "x0");
        assert_eq!(x1, "x1");
        assert_eq!(x30, "x30");
        assert_eq!(sp, "sp");
        
        // Test GPR display (32-bit)
        let gpr32_opts = RegFormatOpts::default_with_arch_and_size(cfg, MemorySize::_32);
        let w0 = format!("{}", AArch64Reg::display(&reg0, gpr32_opts.clone()));
        let w1 = format!("{}", AArch64Reg::display(&reg1, gpr32_opts));
        
        assert_eq!(w0, "w0");
        assert_eq!(w1, "w1");
        
        // Test SIMD display
        let simd_opts = RegFormatOpts::with_reg_class(cfg, MemorySize::_64, RegisterClass::Simd);
        let v0 = format!("{}", AArch64Reg::display(&reg0, simd_opts.clone()));
        let v1 = format!("{}", AArch64Reg::display(&reg1, simd_opts));
        
        assert_eq!(v0, "v0.d");
        assert_eq!(v1, "v1.d");
    }
    
    #[test]
    fn test_condition_codes() {
        use crate::ConditionCode;
        
        assert_eq!(format!("{}", ConditionCode::EQ), "eq");
        assert_eq!(format!("{}", ConditionCode::NE), "ne");
        assert_eq!(format!("{}", ConditionCode::HI), "hi");
        assert_eq!(format!("{}", ConditionCode::LS), "ls");
        assert_eq!(format!("{}", ConditionCode::GE), "ge");
        assert_eq!(format!("{}", ConditionCode::LT), "lt");
        assert_eq!(format!("{}", ConditionCode::GT), "gt");
        assert_eq!(format!("{}", ConditionCode::LE), "le");
    }
    
    #[test]
    #[cfg(feature = "x64_shim")]
    fn test_condition_translation() {
        use portal_solutions_asm_x86_64::ConditionCode as X64CC;
        use crate::shim::translate_condition;
        use crate::ConditionCode as AArch64CC;
        
        assert_eq!(translate_condition(X64CC::E), AArch64CC::EQ);
        assert_eq!(translate_condition(X64CC::NE), AArch64CC::NE);
        assert_eq!(translate_condition(X64CC::B), AArch64CC::LO);
        assert_eq!(translate_condition(X64CC::A), AArch64CC::HI);
        assert_eq!(translate_condition(X64CC::L), AArch64CC::LT);
        assert_eq!(translate_condition(X64CC::G), AArch64CC::GT);
    }
    
    #[test]
    #[cfg(feature = "x64_shim")]
    fn test_shim_basic_operations() {
        use portal_solutions_asm_x86_64::{X64Arch, out::WriterCore as X64WriterCore};
        use crate::shim::X64ToAArch64Shim;
        use alloc::string::String;
        use core::fmt::Write;
        
        let mut output = String::new();
        {
            let writer: &mut dyn Write = &mut output;
            let mut shim = X64ToAArch64Shim::new(writer);
            let cfg = X64Arch::default();
            
            // Test HLT instruction
            X64WriterCore::hlt(&mut shim, cfg).unwrap();
        }
        assert!(output.contains("brk"));
        
        // Test RET instruction
        output.clear();
        {
            let writer: &mut dyn Write = &mut output;
            let mut shim = X64ToAArch64Shim::new(writer);
            let cfg = X64Arch::default();
            X64WriterCore::ret(&mut shim, cfg).unwrap();
        }
        assert!(output.contains("ret"));
    }
    
    #[test]
    #[cfg(feature = "x64_shim")]
    fn test_memarg_adapter() {
        use portal_pc_asm_common::types::reg::Reg;
        use crate::{shim::MemArgAdapter, out::arg::MemArg as AArch64MemArg};
        
        // Test with a simple register
        let x64_reg = Reg(0);
        let adapter = MemArgAdapter::new(&x64_reg);
        
        // Verify the adapter implements the AArch64 MemArg trait
        let kind = adapter.concrete_mem_kind();
        
        // Should convert to a NoMem variant
        match kind {
            crate::out::arg::MemArgKind::NoMem(arg) => {
                match arg {
                    crate::out::arg::ArgKind::Reg { reg, .. } => {
                        assert_eq!(reg.0, 0);
                    }
                    _ => panic!("Expected register argument"),
                }
            }
            _ => panic!("Expected NoMem variant"),
        }
    }
}

/// AArch64 condition codes for conditional instructions.
///
/// These codes are used with conditional branches (b.cond), conditional select (csel),
/// and other conditional instructions. Each code tests specific CPU flags (NZCV).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum ConditionCode {
    /// Equal (Z == 1).
    EQ = 0b0000,
    /// Not equal (Z == 0).
    NE = 0b0001,
    /// Carry set / Unsigned higher or same (C == 1).
    HS = 0b0010,
    /// Carry clear / Unsigned lower (C == 0).
    LO = 0b0011,
    /// Minus / Negative (N == 1).
    MI = 0b0100,
    /// Plus / Positive or zero (N == 0).
    PL = 0b0101,
    /// Overflow set (V == 1).
    VS = 0b0110,
    /// Overflow clear (V == 0).
    VC = 0b0111,
    /// Unsigned higher (C == 1 && Z == 0).
    HI = 0b1000,
    /// Unsigned lower or same (C == 0 || Z == 1).
    LS = 0b1001,
    /// Signed greater than or equal (N == V).
    GE = 0b1010,
    /// Signed less than (N != V).
    LT = 0b1011,
    /// Signed greater than (Z == 0 && N == V).
    GT = 0b1100,
    /// Signed less than or equal (Z == 1 || N != V).
    LE = 0b1101,
    /// Always (unconditional).
    AL = 0b1110,
    /// Never (reserved, behaves as always in most contexts).
    NV = 0b1111,
}

impl Display for ConditionCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConditionCode::EQ => write!(f, "eq"),
            ConditionCode::NE => write!(f, "ne"),
            ConditionCode::HS => write!(f, "hs"),
            ConditionCode::LO => write!(f, "lo"),
            ConditionCode::MI => write!(f, "mi"),
            ConditionCode::PL => write!(f, "pl"),
            ConditionCode::VS => write!(f, "vs"),
            ConditionCode::VC => write!(f, "vc"),
            ConditionCode::HI => write!(f, "hi"),
            ConditionCode::LS => write!(f, "ls"),
            ConditionCode::GE => write!(f, "ge"),
            ConditionCode::LT => write!(f, "lt"),
            ConditionCode::GT => write!(f, "gt"),
            ConditionCode::LE => write!(f, "le"),
            ConditionCode::AL => write!(f, "al"),
            ConditionCode::NV => write!(f, "nv"),
        }
    }
}