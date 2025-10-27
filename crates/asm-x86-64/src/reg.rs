use core::fmt::{Display, Formatter};

use portal_pc_asm_common::types::reg::Reg;

use crate::{
    out::{WriterCore, arg::MemArgKind},
    *,
};
pub trait X64Reg: crate::out::arg::Arg + Sized {
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result;
    fn display<'a>(&'a self, opts: RegFormatOpts) -> RegDisplay;
    fn context_handle(&self, arch: &X64Arch) -> (Reg, u32, u32);
    fn load_from_context<Error: core::error::Error>(
        &self,
        arch: &X64Arch,
        x: &mut (dyn WriterCore<Error = Error> + '_),
        xchg: bool,
    ) -> Result<(), Error> {
        let (a, b, c) = self.context_handle(arch);
        x.mov(
            *arch,
            self,
            &MemArgKind::Mem {
                base: a,
                offset: None,
                disp: b,
                size: MemorySize::_64,
            },
            None,
        )?;
        if xchg {
            x.xchg(
                *arch,
                self,
                &MemArgKind::Mem {
                    base: self,
                    offset: None,
                    disp: c,
                    size: MemorySize::_64,
                },
                None,
            )?;
        } else {
            x.mov(
                *arch,
                self,
                &MemArgKind::Mem {
                    base: self,
                    offset: None,
                    disp: c,
                    size: MemorySize::_64,
                },
                None,
            )?;
        }
        Ok(())
    }
}
impl X64Reg for Reg {
    fn format(&self, f: &mut Formatter<'_>, opts: &RegFormatOpts) -> core::fmt::Result {
        let idx = (self.0 as usize) % (if opts.arch.apx { 32 } else { 16 });
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
pub struct RegDisplay {
    reg: Reg,
    opts: RegFormatOpts,
}
impl Display for RegDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        X64Reg::format(&self.reg, f, &self.opts)
    }
}
