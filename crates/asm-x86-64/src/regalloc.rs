//! Register allocation integration for x86-64.
//!
//! This module provides types and functions for integrating with the register
//! allocator, including register kind definitions, Cmd processing, and state
//! initialization.

use crate::{X64Arch, out::WriterCore, stack::StackManager};
use portal_solutions_asm_regalloc::{Cmd, RegAlloc, RegAllocFrame};

/// Register kind for x86-64.
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
/// * `arch` - The x86-64 architecture configuration
/// * `cmd` - The register allocation command to process
/// * `stack_manager` - Optional stack manager for advanced stack operations
///
/// # Returns
/// Result indicating success or a writer error
pub fn process_cmd<Context, E: core::error::Error>(
    writer: &mut (dyn WriterCore<Context, Error = E> + '_),
    ctx: &mut Context,
    arch: X64Arch,
    cmd: &Cmd<RegKind>,
    stack_manager: Option<&mut StackManager>,
) -> Result<(), E> {
    use portal_pc_asm_common::types::reg::Reg;

    match cmd {
        Cmd::Push(target) => {
            let reg = Reg(target.reg);
            match target.kind {
                RegKind::Int => {
                    // If we have a stack manager, use it for RSP-aware operations
                    if let Some(stack_mgr) = stack_manager {
                        stack_mgr.flush_before_rsp_use(writer, ctx, arch)?;
                    }
                    writer.push(ctx, arch, &reg)
                }
                RegKind::Float => {
                    // For float registers, we need to manually push using movsd
                    // This involves RSP operations, so flush any pending stack operations
                    if let Some(stack_mgr) = stack_manager {
                        stack_mgr.flush_before_rsp_use(writer, ctx, arch)?;
                    }

                    // Adjust stack: sub rsp, 8
                    let rsp = Reg(4);
                    let imm8 = crate::out::arg::MemArgKind::NoMem(crate::out::arg::ArgKind::Lit(8));
                    writer.sub(ctx, arch, &rsp, &imm8)?;
                    // Store the XMM register to [rsp]
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: rsp,
                        offset: None,
                        disp: 0,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Xmm,
                    };
                    writer.fmov(ctx, arch, &mem, &reg)
                }
            }
        }
        Cmd::Pop(target) => {
            let reg = Reg(target.reg);
            match target.kind {
                RegKind::Int => {
                    // If we have a stack manager, use it for RSP-aware operations
                    if let Some(stack_mgr) = stack_manager {
                        stack_mgr.flush_before_rsp_use(writer, ctx, arch)?;
                    }
                    writer.pop(ctx, arch, &reg)
                }
                RegKind::Float => {
                    // For float registers, manually pop using movsd
                    // This involves RSP operations, so flush any pending stack operations
                    if let Some(stack_mgr) = stack_manager {
                        stack_mgr.flush_before_rsp_use(writer, ctx, arch)?;
                    }

                    // Load the XMM register from [rsp]
                    let rsp = Reg(4);
                    let mem = crate::out::arg::MemArgKind::Mem {
                        base: rsp,
                        offset: None,
                        disp: 0,
                        size: portal_pc_asm_common::types::mem::MemorySize::_64,
                        reg_class: crate::RegisterClass::Xmm,
                    };
                    writer.fmov(ctx, arch, &reg, &mem)?;
                    // Adjust stack back: add rsp, 8
                    let imm8 = crate::out::arg::MemArgKind::NoMem(crate::out::arg::ArgKind::Lit(8));
                    writer.add(ctx, arch, &rsp, &imm8)
                }
            }
        }
        Cmd::GetLocal { dest, local } => {
            let reg = Reg(dest.reg);
            // Calculate offset from rbp for locals (negative for downward growth)
            let offset = -((*local as i32 + 1) * 8);
            let size = portal_pc_asm_common::types::mem::MemorySize::_64;
            let reg_class = match dest.kind {
                RegKind::Int => crate::RegisterClass::Gpr,
                RegKind::Float => crate::RegisterClass::Xmm,
            };

            // Use stack manager for offset-based access if available
            if let Some(stack_mgr) = stack_manager {
                stack_mgr.access_stack(writer, ctx, arch, offset, size, reg_class, &reg)
            } else {
                // Fallback to direct memory access
                let mem = crate::out::arg::MemArgKind::Mem {
                    base: Reg(5), // rbp for locals
                    offset: None,
                    disp: offset as u32,
                    size,
                    reg_class,
                };
                match dest.kind {
                    RegKind::Int => writer.mov(ctx, arch, &reg, &mem),
                    RegKind::Float => writer.fmov(ctx, arch, &reg, &mem),
                }
            }
        }

        Cmd::SetLocal { src, local } => {
            let reg = Reg(src.reg);
            // Calculate offset from rbp for locals (negative for downward growth)
            let offset = -((*local as i32 + 1) * 8);
            let size = portal_pc_asm_common::types::mem::MemorySize::_64;
            let reg_class = match src.kind {
                RegKind::Int => crate::RegisterClass::Gpr,
                RegKind::Float => crate::RegisterClass::Xmm,
            };

            // Fallback to direct memory access
            let mem = crate::out::arg::MemArgKind::Mem {
                base: Reg(5), // rbp for locals
                offset: None,
                disp: offset as u32,
                size,
                reg_class,
            };
            match src.kind {
                RegKind::Int => writer.mov(ctx, arch, &mem, &reg),
                RegKind::Float => writer.fmov(ctx, arch, &mem, &reg),
            }
        }
    }
}

/// Maps a register index to a physical register kind.
///
/// # Arguments
/// * `_idx` - The logical register index (currently unused)
/// * `is_float` - Whether this is a float/SIMD register
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

/// Initialize register allocation state for x86-64.
///
/// Creates a RegAlloc instance with the specified number of registers per kind,
/// reserving specific registers according to the calling convention.
///
/// # Type Parameters
/// * `N` - Number of registers per kind (typically 16 or 32 with APX)
///
/// # Arguments
/// * `_arch` - The x86-64 architecture configuration (currently unused)
///
/// # Returns
/// A newly initialized RegAlloc instance with appropriate registers reserved
pub fn init_regalloc<const N: usize>(
    _arch: X64Arch,
) -> RegAlloc<RegKind, N, [[RegAllocFrame<RegKind>; N]; 2]> {
    // Initialize integer register frame
    let mut int_frame: [RegAllocFrame<RegKind>; N] = core::array::from_fn(|_| RegAllocFrame::Empty);

    // Reserve rsp (4) and rbp (5) for stack and frame pointer
    if N > 4 {
        int_frame[4] = RegAllocFrame::Reserved;
    }
    if N > 5 {
        int_frame[5] = RegAllocFrame::Reserved;
    }

    // Initialize float register frame
    let float_frame: [RegAllocFrame<RegKind>; N] = core::array::from_fn(|_| RegAllocFrame::Empty);

    // Create frames array directly without MaybeUninit
    let frames = [int_frame, float_frame];

    RegAlloc { frames, tos: None }
}
