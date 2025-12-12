//! iced-x86 backend for assembling machine code.
#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use core::fmt::Display;

use crate::out::{Writer, WriterCore};
use crate::{ConditionCode, X64Arch};

/// Helper functions for translating iced-x86 components to crate types.
#[cfg(feature = "iced")]
mod helpers {
    use super::*;
    use crate::out::arg::{ArgKind, MemArgKind};
    use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

    /// Convert an iced register to our Reg type.
    pub fn iced_register_to_reg(reg: iced_x86::Register) -> Option<Reg> {
        (reg as u32).try_into().ok().map(Reg)
    }

    /// Convert an iced operand to our MemArgKind.
    pub fn iced_operand_to_mem_arg_kind(
        instr: &iced_x86::Instruction,
        op_index: u32,
    ) -> Option<MemArgKind<ArgKind>> {
        if op_index >= instr.op_count() {
            return None;
        }
        match instr.op_kind(op_index) {
            iced_x86::OpKind::Register => {
                let reg = instr.op_register(op_index);
                iced_register_to_reg(reg).map(|r| {
                    let size = MemorySize::_64; // default for registers
                    MemArgKind::NoMem(ArgKind::Reg { reg: r, size })
                })
            }
            iced_x86::OpKind::Immediate8 | iced_x86::OpKind::Immediate8_2nd => {
                Some(MemArgKind::NoMem(ArgKind::Lit(instr.immediate8() as u64)))
            }
            iced_x86::OpKind::Immediate16 => {
                Some(MemArgKind::NoMem(ArgKind::Lit(instr.immediate16() as u64)))
            }
            iced_x86::OpKind::Immediate32 => {
                Some(MemArgKind::NoMem(ArgKind::Lit(instr.immediate32() as u64)))
            }
            iced_x86::OpKind::Immediate64 => {
                Some(MemArgKind::NoMem(ArgKind::Lit(instr.immediate64())))
            }
            iced_x86::OpKind::Immediate8to16
            | iced_x86::OpKind::Immediate8to32
            | iced_x86::OpKind::Immediate8to64 => Some(MemArgKind::NoMem(ArgKind::Lit(
                instr.immediate8to64() as u64,
            ))),
            iced_x86::OpKind::Memory => {
                let base = iced_register_to_reg(instr.memory_base());
                let index = iced_register_to_reg(instr.memory_index());
                let scale = instr.memory_index_scale();
                let disp = instr.memory_displacement64() as u32;
                let size = match instr.memory_size().size() {
                    1 => MemorySize::_8,
                    2 => MemorySize::_16,
                    4 => MemorySize::_32,
                    8 => MemorySize::_64,
                    _ => MemorySize::_64,
                };
                let reg_class = crate::RegisterClass::Gpr; // default, could be Xmm for floats
                let offset = index.map(|idx| {
                    (
                        ArgKind::Reg {
                            reg: idx,
                            size: MemorySize::_64,
                        },
                        scale as u32,
                    )
                });
                Some(MemArgKind::Mem {
                    base: base
                        .map(|r| ArgKind::Reg {
                            reg: r,
                            size: MemorySize::_64,
                        })
                        .unwrap_or(ArgKind::Lit(0)), // dummy if no base
                    offset,
                    disp,
                    size,
                    reg_class,
                })
            }
            _ => None,
        }
    }

    /// Get the first operand as MemArgKind.
    pub fn op0(instr: &iced_x86::Instruction) -> Option<MemArgKind<ArgKind>> {
        iced_operand_to_mem_arg_kind(instr, 0)
    }

    /// Get the second operand as MemArgKind.
    pub fn op1(instr: &iced_x86::Instruction) -> Option<MemArgKind<ArgKind>> {
        iced_operand_to_mem_arg_kind(instr, 1)
    }

    /// Get the third operand as MemArgKind.
    pub fn op2(instr: &iced_x86::Instruction) -> Option<MemArgKind<ArgKind>> {
        iced_operand_to_mem_arg_kind(instr, 2)
    }
}

/// Default instruction handler that translates as many iced instructions as feasible to Writer calls.
#[cfg(feature = "iced")]
pub fn default_instruction_handler<Context, W, L>(
    writer: &mut W,
    ctx: &mut Context,
    arch: &crate::X64Arch,
    labels: &mut alloc::collections::BTreeMap<u64, L>,
    instr: &iced_x86::Instruction,
    ip: u64,
    raw_bytes: &[u8],
) -> Result<(), W::Error>
where
    W: crate::out::Writer<L, Context>,
    L: From<u64> + Clone + core::fmt::Display,
{
    use helpers::*;
    use iced_x86::Mnemonic;
    let dest = op0(instr);
    let dest = dest.as_ref().map(|m| m as &dyn crate::out::arg::MemArg);
    let src = op1(instr);
    let src = src.as_ref().map(|m| m as &dyn crate::out::arg::MemArg);
    let val = op2(instr);
    let val = val.as_ref().map(|m| m as &dyn crate::out::arg::MemArg);

    match instr.mnemonic() {
        Mnemonic::Mov => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.mov(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Add => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.add(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Sub => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.sub(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Cmp => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.cmp(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::And => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.and(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Or => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.or(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Xor => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.eor(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Shl => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.shl(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Shr => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.shr(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Sar => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.sar(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Not => {
            if let Some(op) = dest {
                writer.not(ctx, *arch, op)?;
            }
        }
        Mnemonic::Lea => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.lea(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Movsx => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.movsx(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Movzx => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.movzx(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Push => {
            if let Some(op) = dest {
                writer.push(ctx, *arch, op)?;
            }
        }
        Mnemonic::Pop => {
            if let Some(op) = dest {
                writer.pop(ctx, *arch, op)?;
            }
        }
        Mnemonic::Pushf => {
            writer.pushf(ctx, *arch)?;
        }
        Mnemonic::Popf => {
            writer.popf(ctx, *arch)?;
        }
        Mnemonic::Call => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.call_label(ctx, *arch, label)?;
            } else if let Some(op) = dest {
                writer.call(ctx, *arch, op)?;
            }
        }
        Mnemonic::Jmp => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jmp_label(ctx, *arch, label)?;
            } else if let Some(op) = dest {
                writer.jmp(ctx, *arch, op)?;
            }
        }
        Mnemonic::Ret => {
            writer.ret(ctx, *arch)?;
        }
        Mnemonic::Hlt => {
            writer.hlt(ctx, *arch)?;
        }
        Mnemonic::Xchg => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.xchg(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Imul => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.mul(ctx, *arch, a, b)?;
            }
        }
        Mnemonic::Idiv => {
            if let Some(b) = src {
                writer.idiv(ctx, *arch, dest.unwrap_or(b), b)?;
            }
        }
        Mnemonic::Div => {
            if let Some(b) = src {
                writer.div(ctx, *arch, dest.unwrap_or(b), b)?;
            }
        }
        Mnemonic::Addsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fadd(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Subsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fsub(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Mulsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fmul(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Divsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fdiv(ctx, *arch, d, s)?;
            }
        }
        Mnemonic::Movsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fmov(ctx, *arch, d, s)?;
            }
        }
        // Conditional moves
        Mnemonic::Cmovo => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::O, op, v)?;
            }
        }
        Mnemonic::Cmovno => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NO, op, v)?;
            }
        }
        Mnemonic::Cmovb => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::B, op, v)?;
            }
        }
        Mnemonic::Cmovae => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NB, op, v)?;
            }
        }
        Mnemonic::Cmove => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::E, op, v)?;
            }
        }
        Mnemonic::Cmovne => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NE, op, v)?;
            }
        }
        Mnemonic::Cmovbe => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NA, op, v)?;
            }
        }
        Mnemonic::Cmova => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::A, op, v)?;
            }
        }
        Mnemonic::Cmovs => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::S, op, v)?;
            }
        }
        Mnemonic::Cmovns => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NS, op, v)?;
            }
        }
        Mnemonic::Cmovp => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::P, op, v)?;
            }
        }
        Mnemonic::Cmovnp => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NP, op, v)?;
            }
        }
        Mnemonic::Cmovl => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::L, op, v)?;
            }
        }
        Mnemonic::Cmovge => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NL, op, v)?;
            }
        }
        Mnemonic::Cmovle => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::NG, op, v)?;
            }
        }
        Mnemonic::Cmovg => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(ctx, *arch, crate::ConditionCode::G, op, v)?;
            }
        }
        // Conditional jumps
        Mnemonic::Jo => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::O, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::O, op)?;
            }
        }
        Mnemonic::Jno => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NO, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NO, op)?;
            }
        }
        Mnemonic::Jb => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::B, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::B, op)?;
            }
        }
        Mnemonic::Jae => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NB, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NB, op)?;
            }
        }
        Mnemonic::Je => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::E, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::E, op)?;
            }
        }
        Mnemonic::Jne => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NE, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NE, op)?;
            }
        }
        Mnemonic::Jbe => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NA, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NA, op)?;
            }
        }
        Mnemonic::Ja => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::A, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::A, op)?;
            }
        }
        Mnemonic::Js => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::S, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::S, op)?;
            }
        }
        Mnemonic::Jns => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NS, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NS, op)?;
            }
        }
        Mnemonic::Jp => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::P, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::P, op)?;
            }
        }
        Mnemonic::Jnp => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NP, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NP, op)?;
            }
        }
        Mnemonic::Jl => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::L, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::L, op)?;
            }
        }
        Mnemonic::Jge => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NL, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NL, op)?;
            }
        }
        Mnemonic::Jle => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::NG, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::NG, op)?;
            }
        }
        Mnemonic::Jg => {
            if instr.op0_kind() == iced_x86::OpKind::NearBranch16
                || instr.op0_kind() == iced_x86::OpKind::NearBranch32
                || instr.op0_kind() == iced_x86::OpKind::NearBranch64
            {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(ctx, *arch, crate::ConditionCode::G, label)?;
            } else if let Some(op) = dest {
                writer.jcc(ctx, *arch, crate::ConditionCode::G, op)?;
            }
        }
        // For unsupported instructions, emit as raw bytes
        _ => {
            writer.db(ctx, *arch, raw_bytes)?;
        }
    }
    Ok(())
}

use crate::out::arg::{ArgKind, MemArgKind};
use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

// Helper: map our Reg to iced Register
fn reg_to_register(r: Reg) -> Option<iced_x86::Register> {
    iced_x86::Register::try_from(r.0 as usize).ok()
}

// Helper: map our Reg to iced registers
fn map_gpr64(r: Reg) -> Option<iced_x86::code_asm::AsmRegister64> {
    reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr64::get_gpr64(reg))
}
fn map_gpr32(r: Reg) -> Option<iced_x86::code_asm::AsmRegister32> {
    reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr32::get_gpr32(reg))
}
fn map_gpr16(r: Reg) -> Option<iced_x86::code_asm::AsmRegister16> {
    reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr16::get_gpr16(reg))
}
fn map_gpr8(r: Reg) -> Option<iced_x86::code_asm::AsmRegister8> {
    reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr8::get_gpr8(reg))
}
fn map_xmm(r: Reg) -> Option<iced_x86::code_asm::AsmRegisterXmm> {
    reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::xmm::get_xmm(reg))
}
macro_rules! to_iced_operand {
    ($mem:expr => |$on:pat_param|$body:expr) => {
        match $mem {
            MemArgKind::NoMem(ArgKind::Reg { reg, size }) => match size {
                MemorySize::_64 => {
                    match map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax) {
                        $on => $body,
                    }
                }
                MemorySize::_32 => {
                    match map_gpr32(*reg).unwrap_or(iced_x86::code_asm::registers::gpr32::eax) {
                        $on => $body,
                    }
                }
                MemorySize::_16 => {
                    match map_gpr16(*reg).unwrap_or(iced_x86::code_asm::registers::gpr16::ax) {
                        $on => $body,
                    }
                }
                MemorySize::_8 => {
                    match map_gpr8(*reg).unwrap_or(iced_x86::code_asm::registers::gpr8::al) {
                        $on => $body,
                    }
                }
                _ => match map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax) {
                    $on => $body,
                },
            },
            MemArgKind::NoMem(ArgKind::Lit(v)) => match *v {
                $on => $body,
            },
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => {
                let base_r = if let ArgKind::Reg { reg, .. } = base {
                    map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax)
                } else {
                    iced_x86::code_asm::registers::gpr64::rax
                };
                let mem_operand = if let Some((off, scale)) = offset {
                    if let ArgKind::Reg { reg: off_reg, .. } = off {
                        let off_r = map_gpr64(*off_reg)
                            .unwrap_or(iced_x86::code_asm::registers::gpr64::rax);
                        base_r + off_r * (*scale as i32) + (*disp as i32)
                    } else {
                        base_r + (*disp as i32)
                    }
                } else {
                    base_r + (*disp as i32)
                };
                match (size, reg_class) {
                    (MemorySize::_8, _) => match iced_x86::code_asm::byte_ptr(mem_operand) {
                        $on => $body,
                    },
                    (MemorySize::_16, _) => match iced_x86::code_asm::word_ptr(mem_operand) {
                        $on => $body,
                    },
                    (MemorySize::_32, _) => match iced_x86::code_asm::dword_ptr(mem_operand) {
                        $on => $body,
                    },
                    (MemorySize::_64, _) => match iced_x86::code_asm::qword_ptr(mem_operand) {
                        $on => $body,
                    },
                    (_, &crate::RegisterClass::Xmm) => {
                        match iced_x86::code_asm::xmmword_ptr(mem_operand) {
                            $on => $body,
                        }
                    }
                    _ => match iced_x86::code_asm::qword_ptr(mem_operand) {
                        $on => $body,
                    },
                }
            }
        }
    };
}

#[cfg(feature = "iced")]
/// Minimal, flexible frontend for one‑the‑fly decoding using iced-x86.
///
/// This type wraps a `Writer` and provides a streaming decoder loop. It
/// does not attempt to translate every iced instruction itself — instead it
/// invokes a user-provided handler for each decoded `iced_x86::Instruction`.
/// The handler can then dispatch to the `Writer` trait methods (there is a
/// convenience `handler` example in docs). An optional `inline_data_hook`
/// can claim ranges of bytes to be emitted via `Writer::db` instead of
/// decoding them as instructions.
#[cfg(feature = "iced")]
pub struct IcedFrontend<'a, W, L, H, D, Context>
where
    W: crate::out::Writer<L, Context> + 'a,
{
    /// Underlying writer to dispatch translated instructions to.
    pub writer: &'a mut W,
    /// Architecture configuration passed to writer when needed.
    pub arch: crate::X64Arch,
    /// Map of IP addresses to labels for RIP-relative jumps.
    pub labels: alloc::collections::BTreeMap<u64, L>,
    /// Instruction handler called for each decoded instruction.
    pub handler: H,
    /// Optional hook to detect inline data. Given a slice at current
    /// position and the virtual IP, return `Some(n)` to claim `n` bytes
    /// as data (these bytes will be emitted via `Writer::db`) or `None` to
    /// let the decoder try to decode an instruction.
    pub inline_data_hook: Option<D>,
    phantom: core::marker::PhantomData<Context>,
}

#[cfg(feature = "iced")]
impl<'a, Context, W, L: Ord + Clone + Display + From<u64>, H, D>
    IcedFrontend<'a, W, L, H, D, Context>
where
    W: crate::out::Writer<L, Context> + 'a,
    H: FnMut(
        &mut W,
        &mut Context,
        &crate::X64Arch,
        &mut alloc::collections::BTreeMap<u64, L>,
        &iced_x86::Instruction,
        u64,
        &[u8],
    ) -> Result<(), W::Error>,
    D: FnMut(&[u8], &mut Context, u64) -> Option<usize>,
{
    /// Create a new frontend.
    pub fn new(writer: &'a mut W, arch: crate::X64Arch, handler: H) -> Self {
        Self {
            writer,
            arch,
            labels: alloc::collections::BTreeMap::new(),
            handler,
            inline_data_hook: None,
            phantom: core::marker::PhantomData,
        }
    }

    /// Set or replace the inline-data detection hook.
    pub fn set_inline_hook(&mut self, hook: D) {
        self.inline_data_hook = Some(hook);
    }

    /// Create a new frontend with the default instruction handler.
    ///
    /// The default handler translates as many iced instructions as feasible
    /// to Writer calls, falling back to emitting raw bytes for unsupported instructions.
    pub fn with_default_handler(
        writer: &'a mut W,
        arch: crate::X64Arch,
    ) -> IcedFrontend<
        'a,
        W,
        L,
        impl FnMut(
            &mut W,
            &mut Context,
            &crate::X64Arch,
            &mut alloc::collections::BTreeMap<u64, L>,
            &iced_x86::Instruction,
            u64,
            &[u8],
        ) -> Result<(), W::Error>,
        D,
        Context,
    >
    where
        L: From<u64> + Clone + core::fmt::Display,
    {
        let handler = move |w: &mut W,
                            ctx: &mut Context,
                            a: &crate::X64Arch,
                            labels: &mut alloc::collections::BTreeMap<u64, L>,
                            i: &iced_x86::Instruction,
                            ip: u64,
                            rb: &[u8]| {
            default_instruction_handler(w, ctx, a, labels, i, ip, rb)
        };
        IcedFrontend {
            writer,
            arch,
            labels: alloc::collections::BTreeMap::new(),
            handler,
            inline_data_hook: None,
            phantom: core::marker::PhantomData,
        }
    }

    /// Process the provided bytes starting at the given virtual `base_ip`.
    ///
    /// This decodes instructions sequentially and calls the handler for each
    /// instruction. If an `inline_data_hook` is installed and claims a range
    /// of bytes, those bytes are emitted via `Writer::db` and decoding
    /// resumes after them.
    pub fn process_bytes(
        &mut self,
        ctx: &mut Context,
        base_ip: u64,
        bytes: &[u8],
    ) -> Result<(), W::Error> {
        let mut pos: usize = 0;
        let len = bytes.len();
        while pos < len {
            let current_ip = base_ip + pos as u64;
            // Set label if this IP is a jump target
            if let Some(label) = self.labels.get(&current_ip) {
                self.writer.set_label(ctx, self.arch, label.clone())?;
            }

            // Inline-data hook takes precedence
            if let Some(hook) = &mut self.inline_data_hook {
                if let Some(n) = hook(&bytes[pos..], ctx, current_ip) {
                    let take = core::cmp::min(n, len - pos);
                    // Emit as raw data bytes
                    self.writer.db(ctx, self.arch, &bytes[pos..pos + take])?;
                    pos += take;
                    continue;
                }
            }

            // Decode one instruction starting at bytes[pos..]
            let slice = &bytes[pos..];
            let mut decoder =
                iced_x86::Decoder::with_ip(64, slice, current_ip, iced_x86::DecoderOptions::NONE);
            // Decode one instruction
            let instr = decoder.decode();
            let consumed = decoder.position();
            if consumed == 0 {
                // Defensive: if decoder consumed nothing, treat remaining bytes
                // as data to avoid infinite loop.
                self.writer.db(ctx, self.arch, slice)?;
                break;
            }
            // Conservatively create and set label for this instruction start if not already present
            if !self.labels.contains_key(&current_ip) {
                let label = L::from(current_ip);
                self.labels.insert(current_ip, label.clone());
                self.writer.set_label(ctx,self.arch, label)?;
            }
            // Call handler to translate the iced instruction into Writer calls.
            (self.handler)(
                self.writer,
                ctx,
                &self.arch,
                &mut self.labels,
                &instr,
                current_ip,
                &slice[..consumed],
            )?;
            pos += consumed;
        }
        Ok(())
    }
}
