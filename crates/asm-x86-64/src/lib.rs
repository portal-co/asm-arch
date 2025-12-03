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

/// **TEMPORARY HACK**: Controls whether to use the YMM/ZMM register naming hack.
///
/// When `false` (current value), smaller `MemorySize` values are mapped to larger SIMD registers:
/// - `MemorySize::_8` → xmm (128-bit)
/// - `MemorySize::_16` → ymm (256-bit)
/// - `MemorySize::_32` → zmm (512-bit)
/// - `MemorySize::_64` → unmapped (defaults to xmm)
///
/// This is a workaround until proper `MemorySize` variants (_128/_256/_512) are added to `portal-pc-asm-common`.
/// When `true`, the proper (non-hacky) implementation will be used once those variants exist.
///
/// **Consumers can check this value** to determine if the hack is active and adjust their code accordingly.
pub const NO_HACK: bool = false;

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

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use portal_pc_asm_common::types::reg::Reg;
    use crate::reg::X64Reg;
    extern crate alloc;
    use alloc::string::String;
    use alloc::format;
    
    #[test]
    fn test_xmm_register_display() {
        let cfg = X64Arch::default();
        let reg0 = Reg(0);
        let reg1 = Reg(1);
        let reg7 = Reg(7);
        let reg8 = Reg(8);
        
        // Test GPR display
        let gpr_opts = RegFormatOpts::default_with_arch(cfg);
        let gpr0 = format!("{}", X64Reg::display(&reg0, gpr_opts.clone()));
        let gpr1 = format!("{}", X64Reg::display(&reg1, gpr_opts.clone()));
        let gpr7 = format!("{}", X64Reg::display(&reg7, gpr_opts.clone()));
        let gpr8 = format!("{}", X64Reg::display(&reg8, gpr_opts));
        
        assert_eq!(gpr0, "rax");
        assert_eq!(gpr1, "rcx");
        assert_eq!(gpr7, "rdi");
        assert_eq!(gpr8, "r8");
        
        // Test XMM display
        let xmm_opts = RegFormatOpts::with_reg_class(cfg, Default::default(), RegisterClass::Xmm);
        let xmm0 = format!("{}", X64Reg::display(&reg0, xmm_opts.clone()));
        let xmm1 = format!("{}", X64Reg::display(&reg1, xmm_opts.clone()));
        let xmm7 = format!("{}", X64Reg::display(&reg7, xmm_opts.clone()));
        let xmm8 = format!("{}", X64Reg::display(&reg8, xmm_opts));
        
        assert_eq!(xmm0, "xmm0");
        assert_eq!(xmm1, "xmm1");
        assert_eq!(xmm7, "xmm7");
        assert_eq!(xmm8, "xmm8");
    }
    
    #[test]
    fn test_apx_xmm_registers() {
        let cfg_apx = X64Arch { apx: true };
        let reg16 = Reg(16);
        let reg31 = Reg(31);
        
        // Test APX XMM registers (xmm16-xmm31)
        let xmm_opts = RegFormatOpts::with_reg_class(cfg_apx, Default::default(), RegisterClass::Xmm);
        let xmm16 = format!("{}", X64Reg::display(&reg16, xmm_opts.clone()));
        let xmm31 = format!("{}", X64Reg::display(&reg31, xmm_opts));
        
        assert_eq!(xmm16, "xmm16", "Register 16 should display as xmm16 with APX and XMM register class");
        assert_eq!(xmm31, "xmm31", "Register 31 should display as xmm31 with APX and XMM register class");
        
        // Test APX GPR registers for comparison
        let gpr_opts = RegFormatOpts::default_with_arch(cfg_apx);
        let r16 = format!("{}", X64Reg::display(&reg16, gpr_opts.clone()));
        let r31 = format!("{}", X64Reg::display(&reg31, gpr_opts));
        
        assert_eq!(r16, "r16", "Register 16 should display as r16 with APX and GPR register class");
        assert_eq!(r31, "r31", "Register 31 should display as r31 with APX and GPR register class");
    }
    
    #[test]
    fn test_ymm_zmm_register_hack() {
        // Test the temporary hack for YMM/ZMM register naming
        // _8 → xmm (128-bit), _16 → ymm (256-bit), _32 → zmm (512-bit)
        let cfg = X64Arch::default();
        let reg0 = Reg(0);
        let reg1 = Reg(1);
        
        // Test XMM with _8 (hack: 128-bit xmm)
        let xmm_opts = RegFormatOpts::with_reg_class(cfg, MemorySize::_8, RegisterClass::Xmm);
        let xmm0 = format!("{}", X64Reg::display(&reg0, xmm_opts.clone()));
        let xmm1 = format!("{}", X64Reg::display(&reg1, xmm_opts));
        assert_eq!(xmm0, "xmm0", "MemorySize::_8 should map to xmm with hack");
        assert_eq!(xmm1, "xmm1", "MemorySize::_8 should map to xmm with hack");
        
        // Test YMM with _16 (hack: 256-bit ymm)
        let ymm_opts = RegFormatOpts::with_reg_class(cfg, MemorySize::_16, RegisterClass::Xmm);
        let ymm0 = format!("{}", X64Reg::display(&reg0, ymm_opts.clone()));
        let ymm1 = format!("{}", X64Reg::display(&reg1, ymm_opts));
        assert_eq!(ymm0, "ymm0", "MemorySize::_16 should map to ymm with hack");
        assert_eq!(ymm1, "ymm1", "MemorySize::_16 should map to ymm with hack");
        
        // Test ZMM with _32 (hack: 512-bit zmm)
        let zmm_opts = RegFormatOpts::with_reg_class(cfg, MemorySize::_32, RegisterClass::Xmm);
        let zmm0 = format!("{}", X64Reg::display(&reg0, zmm_opts.clone()));
        let zmm1 = format!("{}", X64Reg::display(&reg1, zmm_opts));
        assert_eq!(zmm0, "zmm0", "MemorySize::_32 should map to zmm with hack");
        assert_eq!(zmm1, "zmm1", "MemorySize::_32 should map to zmm with hack");
    }
    
    #[test]
    fn test_float_instruction_with_xmm_registers() {
        use crate::out::arg::Arg;
        
        let cfg = X64Arch::default();
        let reg0 = Reg(0);
        let reg1 = Reg(1);
        
        // Test that registers display as XMM when using DisplayOpts with Xmm register class
        let xmm_opts = DisplayOpts::with_reg_class(cfg, RegisterClass::Xmm);
        let reg0_xmm = format!("{}", Arg::display(&reg0, xmm_opts));
        let reg1_xmm = format!("{}", Arg::display(&reg1, xmm_opts));
        
        assert_eq!(reg0_xmm, "xmm0", "Register 0 should display as xmm0 with XMM register class");
        assert_eq!(reg1_xmm, "xmm1", "Register 1 should display as xmm1 with XMM register class");
        
        // Test that registers still display as GPR by default
        let gpr_opts = DisplayOpts::new(cfg);
        let reg0_gpr = format!("{}", Arg::display(&reg0, gpr_opts));
        let reg1_gpr = format!("{}", Arg::display(&reg1, gpr_opts));
        
        assert_eq!(reg0_gpr, "rax", "Register 0 should display as rax with default (GPR) register class");
        assert_eq!(reg1_gpr, "rcx", "Register 1 should display as rcx with default (GPR) register class");
    }
    
    #[test]
    fn test_pushf_popf_instructions() {
        use crate::out::WriterCore;
        use alloc::string::String;
        use core::fmt::Write;
        
        let cfg = X64Arch::default();
        let mut output = String::new();
        
        // Test pushf instruction
        let writer: &mut dyn Write = &mut output;
        WriterCore::pushf(writer, cfg).expect("pushf should succeed");
        assert_eq!(output, "pushfq\n", "pushf should emit 'pushfq\\n'");
        
        // Test popf instruction
        output.clear();
        let writer: &mut dyn Write = &mut output;
        WriterCore::popf(writer, cfg).expect("popf should succeed");
        assert_eq!(output, "popfq\n", "popf should emit 'popfq\\n'");
    }
}

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
