//! Register handling and formatting for x86-64.
//!
//! This module provides traits and types for working with x86-64 registers,
//! including formatting registers in different sizes and loading from context.

use core::fmt::{Display, Formatter};

use portal_pc_asm_common::types::reg::Reg;

use crate::{
    out::{WriterCore, arg::MemArgKind},
    *,
};

/// Trait for x86-64 register operations.
///
/// Provides methods for formatting registers, displaying them with specific options,
/// and loading values from a context structure.
pub trait X64Reg: crate::out::arg::Arg + Sized {
    /// Formats the register to the given formatter with the specified options.
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result;

    /// Creates a displayable representation of the register with the given options.
    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay;

    /// Returns the context handle for loading this register from a context structure.
    ///
    /// Returns a tuple of (base register, base offset, register offset).
    fn context_handle(&self, arch: &X64Arch) -> (Reg, u32, u32);

    /// Loads the register value from a context structure.
    ///
    /// # Arguments
    /// * `arch` - The architecture configuration
    /// * `x` - The writer to emit instructions to
    /// * `xchg` - If true, uses xchg instead of mov for the final load
    fn load_from_context<Context, Error: core::error::Error>(
        &self,
        ctx: &mut Context,
        arch: &X64Arch,
        x: &mut (dyn WriterCore<Context, Error = Error> + '_),
        xchg: bool,
    ) -> Result<(), Error> {
        let (a, b, c) = self.context_handle(arch);
        x.mov(
            ctx,
            *arch,
            self,
            &MemArgKind::Mem {
                base: a,
                offset: None,
                disp: b,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            },
        )?;
        if xchg {
            x.xchg(
                ctx,
                *arch,
                self,
                &MemArgKind::Mem {
                    base: self,
                    offset: None,
                    disp: c,
                    size: MemorySize::_64,
                    reg_class: crate::RegisterClass::Gpr,
                },
            )?;
        } else {
            x.mov(
                ctx,
                *arch,
                self,
                &MemArgKind::Mem {
                    base: self,
                    offset: None,
                    disp: c,
                    size: MemorySize::_64,
                    reg_class: crate::RegisterClass::Gpr,
                },
            )?;
        }
        Ok(())
    }
}
impl X64Reg for Reg {
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result {
        // Check APX support at the top of the method
        let max_regs = if opts.arch.apx { 32 } else { 16 };
        let idx = (self.0 as usize) % max_regs;

        match opts.reg_class {
            crate::RegisterClass::Xmm => {
                // For XMM/YMM/ZMM registers
                let prefix = if crate::NO_HACK {
                    // Non-hacky code: Proper MemorySize to register mapping (for future use)
                    // Scheme: MemorySize determines register type
                    // - _64 bits (8 bytes) and below -> xmm (128-bit, using scalar operations)
                    // - _128 bits (16 bytes) -> xmm (full 128-bit register) [Future]
                    // - _256 bits (32 bytes) -> ymm (256-bit register) [Future]
                    // - _512 bits (64 bytes) -> zmm (512-bit register) [Future]
                    match &opts.size {
                        MemorySize::_8 | MemorySize::_16 | MemorySize::_32 | MemorySize::_64 => {
                            "xmm"
                        }
                        // Future: Add MemorySize::_128 => "xmm"
                        // Future: Add MemorySize::_256 => "ymm"
                        // Future: Add MemorySize::_512 => "zmm"
                        // Default to xmm for any unknown sizes
                        _ => "xmm",
                    }
                } else {
                    // HACK: Temporary mapping until proper MemorySize variants exist
                    // _8 → 128-bit xmm, _16 → 256-bit ymm, _32 → 512-bit zmm, _64 → unmapped
                    match &opts.size {
                        MemorySize::_8 => "xmm",  // 128-bit XMM register
                        MemorySize::_16 => "ymm", // 256-bit YMM register
                        MemorySize::_32 => "zmm", // 512-bit ZMM register
                        MemorySize::_64 => "xmm", // Unmapped - default to xmm
                        _ => "xmm",
                    }
                };

                // Both regular and APX extended registers use the same format
                write!(f, "{}{}", prefix, idx)
            }
            crate::RegisterClass::Gpr => {
                if idx < 8 {
                    write!(
                        f,
                        "{}",
                        &(match &opts.size {
                            MemorySize::_8 => REG_NAMES_8,
                            MemorySize::_16 => REG_NAMES_16,
                            MemorySize::_32 => REG_NAMES_32,
                            MemorySize::_64 => REG_NAMES,
                        })[idx]
                    )
                } else {
                    write!(
                        f,
                        "r{idx}{}",
                        match &opts.size {
                            MemorySize::_8 => "b",
                            MemorySize::_16 => "w",
                            MemorySize::_32 => "d",
                            MemorySize::_64 => "",
                        }
                    )
                }
            }
        }
    }
    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay {
        RegDisplay { reg: *self, opts }
    }
    fn context_handle(&self, arch: &X64Arch) -> (Reg, u32, u32) {
        (
            Reg(9),
            0x28,
            match (self.0) as u32 % (if arch.apx { 32 } else { 16 }) {
                a => a * 8 + 78,
            },
        )
    }
}

/// A displayable wrapper for a register with formatting options.
///
/// Implements `Display` to format the register according to the specified options.
pub struct RegDisplay {
    reg: Reg,
    opts: RegFormatOpts,
}
impl Display for RegDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        X64Reg::format(&self.reg, f, &self.opts)
    }
}
