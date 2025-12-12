//! Register handling and formatting for RISC-V 64-bit.
//!
//! This module provides traits and types for working with RISC-V 64-bit registers,
//! including formatting registers in different sizes and loading from context.

use core::fmt::{Display, Formatter};

use portal_pc_asm_common::types::reg::Reg;

use crate::{
    out::{WriterCore, arg::MemArgKind},
    *,
};

/// Trait for RISC-V 64-bit register operations.
///
/// Provides methods for formatting registers, displaying them with specific options,
/// and loading values from a context structure.
pub trait RiscV64Reg: crate::out::arg::Arg + Sized {
    /// Formats the register to the given formatter with the specified options.
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result;

    /// Creates a displayable representation of the register with the given options.
    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay;

    /// Returns the context handle for loading this register from a context structure.
    ///
    /// Returns a tuple of (base register, base offset, register offset).
    fn context_handle(&self, arch: &RiscV64Arch) -> (Reg, u32, u32);

    /// Loads the register value from a context structure.
    ///
    /// # Arguments
    /// * `arch` - The architecture configuration
    /// * `x` - The writer to emit instructions to
    /// * `xchg` - If true, uses a swap pattern instead of simple load
    fn load_from_context<Context, Error: core::error::Error>(
        &self,
        ctx: &mut Context,
        arch: &RiscV64Arch,
        x: &mut (dyn WriterCore<Context, Error = Error> + '_),
        xchg: bool,
    ) -> Result<(), Error> {
        let (a, b, c) = self.context_handle(arch);
        x.ld(ctx,
            *arch,
            self,
            &MemArgKind::Mem {
                base: a,
                offset: None,
                disp: b as i32,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            },
        )?;
        if xchg {
            // RISC-V doesn't have direct xchg, simulate with load/store sequence
            x.ld(ctx,
                *arch,
                self,
                &MemArgKind::Mem {
                    base: a,
                    offset: None,
                    disp: c as i32,
                    size: MemorySize::_64,
                    reg_class: crate::RegisterClass::Gpr,
                },
            )?;
        } else {
            x.ld(ctx,
                *arch,
                self,
                &MemArgKind::Mem {
                    base: a,
                    offset: None,
                    disp: c as i32,
                    size: MemorySize::_64,
                    reg_class: crate::RegisterClass::Gpr,
                },
            )?;
        }
        Ok(())
    }
}

impl RiscV64Reg for Reg {
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result {
        let idx = (self.0 as usize) % 32;

        match opts.reg_class {
            crate::RegisterClass::Fp => {
                // For floating-point registers, use f registers
                write!(f, "{}", FREG_NAMES[idx])
            }
            crate::RegisterClass::Gpr => {
                // For general-purpose registers, RISC-V doesn't have separate 32/64 names
                write!(f, "{}", REG_NAMES_64[idx])
            }
        }
    }

    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay {
        RegDisplay { reg: *self, opts }
    }

    fn context_handle(&self, _arch: &RiscV64Arch) -> (Reg, u32, u32) {
        // Using register 9 (s1) as context pointer
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
        RiscV64Reg::format(&self.reg, f, &self.opts)
    }
}
