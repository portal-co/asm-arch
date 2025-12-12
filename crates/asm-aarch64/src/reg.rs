//! Register handling and formatting for AArch64.
//!
//! This module provides traits and types for working with AArch64 registers,
//! including formatting registers in different sizes and loading from context.

use core::fmt::{Display, Formatter};

use portal_pc_asm_common::types::reg::Reg;

use crate::{
    out::{WriterCore, arg::MemArgKind},
    *,
};

/// Trait for AArch64 register operations.
///
/// Provides methods for formatting registers, displaying them with specific options,
/// and loading values from a context structure.
pub trait AArch64Reg: crate::out::arg::Arg + Sized {
    /// Formats the register to the given formatter with the specified options.
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result;
    
    /// Creates a displayable representation of the register with the given options.
    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay;
    
    /// Returns the context handle for loading this register from a context structure.
    ///
    /// Returns a tuple of (base register, base offset, register offset).
    fn context_handle(&self, arch: &AArch64Arch) -> (Reg, u32, u32);
    
    /// Loads the register value from a context structure.
    ///
    /// # Arguments
    /// * `arch` - The architecture configuration
    /// * `x` - The writer to emit instructions to
    /// * `xchg` - If true, uses a swap pattern instead of simple load
    fn load_from_context<Context, Error: core::error::Error>(
        &self,
        arch: &AArch64Arch,
        x: &mut (dyn WriterCore<Context, Error = Error> + '_),
        ctx: &mut Context,
        xchg: bool,
    ) -> Result<(), Error> {
        let (a, b, c) = self.context_handle(arch);
        x.ldr(
            ctx,
            *arch,
            self,
            &MemArgKind::Mem {
                base: a,
                offset: None,
                disp: b as i32,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::Offset,
            },
        )?;
        if xchg {
            // AArch64 doesn't have direct xchg, simulate with load/store sequence
            x.ldr(
                ctx,
                *arch,
                self,
                &MemArgKind::Mem {
                    base: a,
                    offset: None,
                    disp: c as i32,
                    size: MemorySize::_64,
                    reg_class: crate::RegisterClass::Gpr,
                    mode: crate::out::arg::AddressingMode::Offset,
                },
            )?;
        } else {
            x.ldr(
                ctx,
                *arch,
                self,
                &MemArgKind::Mem {
                    base: a,
                    offset: None,
                    disp: c as i32,
                    size: MemorySize::_64,
                    reg_class: crate::RegisterClass::Gpr,
                    mode: crate::out::arg::AddressingMode::Offset,
                },
            )?;
        }
        Ok(())
    }
}

impl AArch64Reg for Reg {
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result {
        let idx = (self.0 as usize) % 32;
        
        match opts.reg_class {
            crate::RegisterClass::Simd => {
                // For SIMD/FP registers, use v registers with element size qualifiers
                let suffix = match &opts.size {
                    MemorySize::_8 => ".b",   // byte element
                    MemorySize::_16 => ".h",  // halfword element
                    MemorySize::_32 => ".s",  // single precision
                    MemorySize::_64 => ".d",  // double precision
                    _ => ".d",  // default to double
                };
                write!(f, "{}{}", VREG_NAMES[idx], suffix)
            }
            crate::RegisterClass::Gpr => {
                // For general-purpose registers
                match &opts.size {
                    MemorySize::_32 => write!(f, "{}", REG_NAMES_32[idx]),
                    MemorySize::_64 | _ => write!(f, "{}", REG_NAMES_64[idx]),
                }
            }
        }
    }
    
    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay {
        RegDisplay { reg: *self, opts }
    }
    
    fn context_handle(&self, _arch: &AArch64Arch) -> (Reg, u32, u32) {
        // Similar to x86-64, using register 9 as context pointer
        (
            Reg(9),
            0x28,
            match (self.0) as u32 % 32 {
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
        AArch64Reg::format(&self.reg, f, &self.opts)
    }
}
