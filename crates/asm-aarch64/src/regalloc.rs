//! Register allocation integration for AArch64.
//!
//! This module provides types and functions for integrating with the register
//! allocator, including register kind definitions, Cmd processing, and state
//! initialization.

use portal_solutions_asm_regalloc::{Cmd, RegAlloc, RegAllocFrame, Target};
use crate::{out::WriterCore, AArch64Arch};

/// Register kind for AArch64.
///
/// Distinguishes between integer/general-purpose registers and floating-point/SIMD registers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RegKind {
    /// Integer/general-purpose register.
    Int = 0,
    /// Floating-point/SIMD register.
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
/// * `arch` - The AArch64 architecture configuration
/// * `cmd` - The register allocation command to process
///
/// # Returns
/// Result indicating success or a writer error
pub fn process_cmd<E: core::error::Error>(
    writer: &mut (dyn WriterCore<Error = E> + '_),
    arch: AArch64Arch,
    cmd: &Cmd<RegKind>,
) -> Result<(), E> {
    use portal_pc_asm_common::types::reg::Reg;
    
    match cmd {
        Cmd::Push(target) => {
            let reg = Reg(target.reg);
            // AArch64 doesn't have push instruction, use str with pre-indexed addressing
            // [sp, #-8]! means: sp = sp - 8, then str to [sp]
            let sp = Reg(31); // sp register
            match target.kind {
                RegKind::Int => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: -8,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Gpr,
                        mode: crate::out::arg::AddressingMode::PreIndex,
                    };
                    writer.str(arch, &reg, &mem)
                }
                RegKind::Float => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: -8,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Simd,
                        mode: crate::out::arg::AddressingMode::PreIndex,
                    };
                    writer.str(arch, &reg, &mem)
                }
            }
        }
        Cmd::Pop(target) => {
            let reg = Reg(target.reg);
            // AArch64 doesn't have pop instruction, use ldr with post-indexed addressing
            // [sp], #8 means: ldr from [sp], then sp = sp + 8
            let sp = Reg(31); // sp register
            match target.kind {
                RegKind::Int => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: 8,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Gpr,
                        mode: crate::out::arg::AddressingMode::PostIndex,
                    };
                    writer.ldr(arch, &reg, &mem)
                }
                RegKind::Float => {
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: sp,
                        offset: None,
                        disp: 8,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Simd,
                        mode: crate::out::arg::AddressingMode::PostIndex,
                    };
                    writer.ldr(arch, &reg, &mem)
                }
            }
        }
        Cmd::GetLocal { dest, local } => {
            let reg = Reg(dest.reg);
            let fp = Reg(29); // x29 is frame pointer
            let mem = crate::out::arg::MemArgKind::Mem {
                base: fp,
                offset: None,
                disp: -((*local as i32 + 1) * 8), // locals are negative offsets from fp
                size: portal_pc_asm_common::types::mem::MemorySize::_64,
                reg_class: match dest.kind {
                    RegKind::Int => crate::RegisterClass::Gpr,
                    RegKind::Float => crate::RegisterClass::Simd,
                },
                mode: crate::out::arg::AddressingMode::Offset,
            };
            writer.ldr(arch, &reg, &mem)
        }
        Cmd::SetLocal { src, local } => {
            let reg = Reg(src.reg);
            let fp = Reg(29); // x29 is frame pointer
            let mem = crate::out::arg::MemArgKind::Mem {
                base: fp,
                offset: None,
                disp: -((*local as i32 + 1) * 8), // locals are negative offsets from fp
                size: portal_pc_asm_common::types::mem::MemorySize::_64,
                reg_class: match src.kind {
                    RegKind::Int => crate::RegisterClass::Gpr,
                    RegKind::Float => crate::RegisterClass::Simd,
                },
                mode: crate::out::arg::AddressingMode::Offset,
            };
            writer.str(arch, &reg, &mem)
        }
    }
}

/// Maps a register index to a physical register kind.
///
/// # Arguments
/// * `idx` - The logical register index
/// * `is_float` - Whether this is a float/SIMD register
///
/// # Returns
/// The register kind (Int or Float)
pub fn map_index_to_kind(idx: usize, is_float: bool) -> RegKind {
    if is_float {
        RegKind::Float
    } else {
        RegKind::Int
    }
}

/// Initialize register allocation state for AArch64.
///
/// Creates a RegAlloc instance with the specified number of registers per kind,
/// reserving specific registers according to the calling convention.
///
/// # Type Parameters
/// * `N` - Number of registers per kind (typically 32 for AArch64)
///
/// # Returns
/// A newly initialized RegAlloc instance with appropriate registers reserved
pub fn init_regalloc<const N: usize>(
    arch: AArch64Arch,
) -> RegAlloc<RegKind, N, [core::mem::MaybeUninit<[RegAllocFrame<RegKind>; N]>; 2]> {
    use core::mem::MaybeUninit;
    
    let mut frames: [MaybeUninit<[RegAllocFrame<RegKind>; N]>; 2] = 
        [MaybeUninit::uninit(), MaybeUninit::uninit()];
    
    // Initialize integer register frame
    let mut int_frame: [RegAllocFrame<RegKind>; N] = 
        core::array::from_fn(|_| RegAllocFrame::Empty);
    
    // Reserve x29 (frame pointer), x30 (link register), and x31 (stack pointer)
    if N > 29 { int_frame[29] = RegAllocFrame::Reserved; }
    if N > 30 { int_frame[30] = RegAllocFrame::Reserved; }
    if N > 31 { int_frame[31] = RegAllocFrame::Reserved; }
    
    frames[0] = MaybeUninit::new(int_frame);
    
    // Initialize float register frame
    let float_frame: [RegAllocFrame<RegKind>; N] = 
        core::array::from_fn(|_| RegAllocFrame::Empty);
    
    frames[1] = MaybeUninit::new(float_frame);
    
    // Safety: We've initialized both frames
    let frames = unsafe {
        core::mem::transmute::<
            [MaybeUninit<[RegAllocFrame<RegKind>; N]>; 2],
            [core::mem::MaybeUninit<[RegAllocFrame<RegKind>; N]>; 2]
        >(frames)
    };
    
    RegAlloc {
        frames,
        tos: None,
    }
}
