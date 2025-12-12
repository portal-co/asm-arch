//! Register allocation integration for RISC-V 64-bit.
//!
//! This module provides types and functions for integrating with the register
//! allocator, including register kind definitions, Cmd processing, and state
//! initialization.

use portal_solutions_asm_regalloc::{Cmd, RegAlloc, RegAllocFrame};
use crate::{out::WriterCore, RiscV64Arch};

/// Register kind for RISC-V 64-bit.
///
/// Distinguishes between integer/general-purpose registers and floating-point registers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RegKind {
    /// Integer/general-purpose register.
    Int = 0,
    /// Floating-point register.
    Float = 1,
}

impl TryFrom<usize> for RegKind {
    type Error = ();
    
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RegKind::Int),
            1 => Ok(RegKind::Float),
            _ => Err(()),
        }
    }
}

/// Process a single register allocation command, emitting assembly instructions.
///
/// # Arguments
/// * `writer` - The instruction writer to emit to
/// * `arch` - The RISC-V 64-bit architecture configuration
/// * `cmd` - The register allocation command to process
///
/// # Returns
/// Result indicating success or a writer error
pub fn process_cmd<Context, E: core::error::Error>(
    writer: &mut (dyn WriterCore<Context, Error = E> + '_),
    ctx: &mut Context,
    arch: RiscV64Arch,
    cmd: &Cmd<RegKind>,
) -> Result<(), E> {
    use portal_pc_asm_common::types::reg::Reg;
    
    match cmd {
        Cmd::Push(target) => {
            let reg = Reg(target.reg);
            let sp = Reg(2); // x2 is stack pointer in RISC-V
            
            // RISC-V doesn't have push instruction
            // We need to: 1) adjust sp, 2) store to [sp]
            // addi sp, sp, -8
            writer.addi(ctx, arch, &sp, &sp, -8)?;
            
            match target.kind {
                RegKind::Int => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: 0,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Gpr,
                    };
                    writer.sd(ctx, arch, &reg, &mem)
                }
                RegKind::Float => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: 0,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Fp,
                    };
                    writer.fsd(ctx, arch, &reg, &mem)
                }
            }
        }
        Cmd::Pop(target) => {
            let reg = Reg(target.reg);
            let sp = Reg(2); // x2 is stack pointer in RISC-V
            
            match target.kind {
                RegKind::Int => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: 0,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Gpr,
                    };
                    writer.ld(ctx, arch, &reg, &mem)?;
                }
                RegKind::Float => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: 0,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Fp,
                    };
                    writer.fld(ctx, arch, &reg, &mem)?;
                }
            }
            
            // RISC-V doesn't have pop instruction
            // After load, adjust sp: addi sp, sp, 8
            writer.addi(ctx, arch, &sp, &sp, 8)
        }
        Cmd::GetLocal { dest, local } => {
            let reg = Reg(dest.reg);
            let fp = Reg(8); // x8/s0 is frame pointer in RISC-V
            let mem = crate::out::arg::MemArgKind::Mem {
                base: fp,
                offset: None,
                disp: -((*local as i32 + 1) * 8), // locals are negative offsets from fp
                size: portal_pc_asm_common::types::mem::MemorySize::_64,
                reg_class: match dest.kind {
                    RegKind::Int => crate::RegisterClass::Gpr,
                    RegKind::Float => crate::RegisterClass::Fp,
                },
            };
            match dest.kind {
                RegKind::Int => writer.ld(ctx, arch, &reg, &mem),
                RegKind::Float => writer.fld(ctx, arch, &reg, &mem),
            }
        }
        Cmd::SetLocal { src, local } => {
            let reg = Reg(src.reg);
            let fp = Reg(8); // x8/s0 is frame pointer in RISC-V
            let mem = crate::out::arg::MemArgKind::Mem {
                base: fp,
                offset: None,
                disp: -((*local as i32 + 1) * 8), // locals are negative offsets from fp
                size: portal_pc_asm_common::types::mem::MemorySize::_64,
                reg_class: match src.kind {
                    RegKind::Int => crate::RegisterClass::Gpr,
                    RegKind::Float => crate::RegisterClass::Fp,
                },
            };
            match src.kind {
                RegKind::Int => writer.sd(ctx, arch, &reg, &mem),
                RegKind::Float => writer.fsd(ctx, arch, &reg, &mem),
            }
        }
    }
}

/// Maps a register index to a physical register kind.
///
/// # Arguments
/// * `idx` - The logical register index
/// * `is_float` - Whether this is a floating-point register
///
/// # Returns
/// The register kind (Int or Float)
pub fn map_index_to_kind(_idx: usize, is_float: bool) -> RegKind {
    if is_float {
        RegKind::Float
    } else {
        RegKind::Int
    }
}

/// Initialize register allocation state for RISC-V 64-bit.
///
/// Creates a RegAlloc instance with the specified number of registers per kind,
/// reserving specific registers according to the RISC-V calling convention.
///
/// # Type Parameters
/// * `N` - Number of registers per kind (typically 32 for RISC-V)
///
/// # Arguments
/// * `_arch` - The RISC-V 64-bit architecture configuration (currently unused)
///
/// # Returns
/// A newly initialized RegAlloc instance with appropriate registers reserved
pub fn init_regalloc<const N: usize>(
    _arch: RiscV64Arch,
) -> RegAlloc<RegKind, N, [[RegAllocFrame<RegKind>; N]; 2]> {
    // Initialize integer register frame
    let mut int_frame: [RegAllocFrame<RegKind>; N] = 
        core::array::from_fn(|_| RegAllocFrame::Empty);
    
    // Reserve RISC-V special registers:
    // x0 (zero) - hardwired to zero
    // x1 (ra) - return address
    // x2 (sp) - stack pointer
    // x3 (gp) - global pointer
    // x4 (tp) - thread pointer
    if N > 0 { int_frame[0] = RegAllocFrame::Reserved; }  // zero
    if N > 1 { int_frame[1] = RegAllocFrame::Reserved; }  // ra
    if N > 2 { int_frame[2] = RegAllocFrame::Reserved; }  // sp
    if N > 3 { int_frame[3] = RegAllocFrame::Reserved; }  // gp
    if N > 4 { int_frame[4] = RegAllocFrame::Reserved; }  // tp
    if N > 8 { int_frame[8] = RegAllocFrame::Reserved; }  // s0/fp - frame pointer
    
    // Initialize float register frame
    let float_frame: [RegAllocFrame<RegKind>; N] = 
        core::array::from_fn(|_| RegAllocFrame::Empty);
    
    // Create frames array directly without MaybeUninit
    let frames = [int_frame, float_frame];
    
    RegAlloc {
        frames,
        tos: None,
    }
}
