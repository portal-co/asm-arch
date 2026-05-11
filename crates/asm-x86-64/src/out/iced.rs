//! iced-x86 backend for assembling machine code.
#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use core::fmt::Display;

use crate::out::{Writer, WriterCore};
use crate::{ConditionCode, X64Arch};

/// Placeholder label type for [`IcedWriter`] when no label tracking is needed.
///
/// This is an uninhabited type — it can never be constructed — so a
/// `BTreeMap<NoLabel, usize>` is always empty and zero-cost.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NoLabel {}

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

fn reg_to_iced(r: Reg) -> iced_x86::Register {
    iced_x86::Register::try_from(r.0 as usize).unwrap_or(iced_x86::Register::RAX)
}

fn mem_kind_to_iced(mk: &MemArgKind<ArgKind>) -> IcedOp {
    match mk {
        MemArgKind::NoMem(ArgKind::Reg { reg, size }) => {
            IcedOp::Reg(reg_to_iced(*reg), *size)
        }
        MemArgKind::NoMem(ArgKind::Lit(v)) => IcedOp::Imm(*v),
        MemArgKind::Mem { base, offset, disp, size, .. } => {
            let base_reg = match base {
                ArgKind::Reg { reg, .. } => reg_to_iced(*reg),
                ArgKind::Lit(_) => iced_x86::Register::None,
            };
            let (idx_reg, scale) = match offset {
                Some((ArgKind::Reg { reg, .. }, s)) => (reg_to_iced(*reg), *s),
                _ => (iced_x86::Register::None, 1),
            };
            IcedOp::Mem(iced_x86::MemoryOperand::with_base_index_scale_displ_size(
                base_reg, idx_reg, scale, *disp as i64, 1,
            ), *size)
        }
    }
}

enum IcedOp {
    Reg(iced_x86::Register, MemorySize),
    Imm(u64),
    Mem(iced_x86::MemoryOperand, MemorySize),
}

/// Binary assembler backend for x86-64 using `iced_x86::Encoder`.
///
/// Implements [`WriterCore`] by encoding each instruction to machine bytes.
/// Does not use `code_asm` — operands are constructed at runtime from
/// [`MemArgKind`] values and encoded via the lower-level `Instruction` API.
///
/// The type parameter `L` is the label type used with [`Writer<L, Context>`].
/// It defaults to [`NoLabel`], which means label tracking is compiled away at
/// zero cost. Specify a concrete `L` (e.g. `u32` or a custom enum) to record
/// label→byte-offset mappings via [`set_label`](Writer::set_label).
#[cfg(feature = "iced")]
pub struct IcedWriter<L = NoLabel> {
    buf: alloc::vec::Vec<u8>,
    ip: u64,
    labels: alloc::collections::BTreeMap<L, usize>,
}

#[cfg(feature = "iced")]
impl<L> IcedWriter<L> {
    /// Create a new writer. `base_ip` is the virtual address the code will run at
    /// (used by the encoder for RIP-relative references).
    pub fn new(base_ip: u64) -> Self {
        Self { buf: alloc::vec::Vec::new(), ip: base_ip, labels: alloc::collections::BTreeMap::new() }
    }

    /// Return the assembled bytes, discarding any recorded label offsets.
    pub fn into_bytes(self) -> alloc::vec::Vec<u8> {
        self.buf
    }

    /// Return the assembled bytes and the recorded label→offset map.
    pub fn into_parts(self) -> (alloc::vec::Vec<u8>, alloc::collections::BTreeMap<L, usize>) {
        (self.buf, self.labels)
    }

    /// Current byte offset (number of bytes assembled so far).
    pub fn offset(&self) -> usize {
        self.buf.len()
    }

    fn encode_instr(&mut self, instr: iced_x86::Instruction) -> Result<(), iced_x86::IcedError> {
        let mut enc = iced_x86::Encoder::new(64);
        let n = enc.encode(&instr, self.ip)?;
        let bytes = enc.take_buffer();
        self.ip += n as u64;
        self.buf.extend_from_slice(&bytes);
        Ok(())
    }

    fn op_to_reg(op: &IcedOp) -> iced_x86::Register {
        match op {
            IcedOp::Reg(r, _) => *r,
            _ => iced_x86::Register::RAX,
        }
    }

    fn op_to_mem(op: &IcedOp) -> iced_x86::MemoryOperand {
        match op {
            IcedOp::Mem(m, _) => m.clone(),
            IcedOp::Reg(r, _) => iced_x86::MemoryOperand::with_base(*r),
            _ => iced_x86::MemoryOperand::with_base(iced_x86::Register::RAX),
        }
    }

    fn size_of(op: &IcedOp) -> MemorySize {
        match op {
            IcedOp::Reg(_, s) | IcedOp::Mem(_, s) => *s,
            IcedOp::Imm(_) => MemorySize::_64,
        }
    }
}

#[cfg(feature = "iced")]
impl<L, Context> crate::out::WriterCore<Context> for IcedWriter<L> {
    type Error = iced_x86::IcedError;

    fn hlt(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        self.encode_instr(iced_x86::Instruction::with(iced_x86::Code::Hlt))
    }

    fn ret(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        self.encode_instr(iced_x86::Instruction::with(iced_x86::Code::Retnq))
    }

    fn pushf(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        self.encode_instr(iced_x86::Instruction::with(iced_x86::Code::Pushfq))
    }

    fn popf(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        self.encode_instr(iced_x86::Instruction::with(iced_x86::Code::Popfq))
    }

    fn mov(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Mov_r64_rm64, *dr, *sr)?
            }
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Mov_r64_imm64, *dr, *v)?
            }
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Mov_r64_rm64, *dr, sm.clone())?
            }
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Mov_rm64_r64, dm.clone(), *sr)?
            }
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Mov_rm64_imm32, dm.clone(), *v as i32)?
            }
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn mov64(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, r: &(dyn crate::out::arg::MemArg + '_), val: u64) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&r.concrete_mem_kind());
        let reg = Self::op_to_reg(&d);
        self.encode_instr(iced_x86::Instruction::with2(iced_x86::Code::Mov_r64_imm64, reg, val)?)
    }

    fn xchg(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Xchg_rm64_r64, *dr, *sr)?
            }
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => {
                iced_x86::Instruction::with2(iced_x86::Code::Xchg_rm64_r64, dm.clone(), *sr)?
            }
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn push(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let o = mem_kind_to_iced(&op.concrete_mem_kind());
        let instr = match &o {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Push_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Push_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn pop(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let o = mem_kind_to_iced(&op.concrete_mem_kind());
        let instr = match &o {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Pop_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Pop_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn call(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let o = mem_kind_to_iced(&op.concrete_mem_kind());
        let instr = match &o {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Call_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Call_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn jmp(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let o = mem_kind_to_iced(&op.concrete_mem_kind());
        let instr = match &o {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Jmp_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Jmp_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn add(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Add_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Add_rm64_imm32, *dr, *v as i32)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Add_r64_rm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Add_rm64_r64, dm.clone(), *sr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Add_rm64_imm32, dm.clone(), *v as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn sub(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Sub_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Sub_rm64_imm32, *dr, *v as i32)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Sub_r64_rm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Sub_rm64_r64, dm.clone(), *sr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Sub_rm64_imm32, dm.clone(), *v as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn and(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::And_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::And_rm64_imm32, *dr, *v as i32)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::And_r64_rm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::And_rm64_r64, dm.clone(), *sr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::And_rm64_imm32, dm.clone(), *v as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn or(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Or_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Or_rm64_imm32, *dr, *v as i32)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Or_r64_rm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Or_rm64_r64, dm.clone(), *sr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Or_rm64_imm32, dm.clone(), *v as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn eor(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Xor_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Xor_rm64_imm32, *dr, *v as i32)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Xor_r64_rm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Xor_rm64_r64, dm.clone(), *sr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Xor_rm64_imm32, dm.clone(), *v as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn shl(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Shl_rm64_imm8, *dr, *v as u32)?,
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) if *sr == iced_x86::Register::CL => iced_x86::Instruction::with1(iced_x86::Code::Shl_rm64_CL, *dr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Shl_rm64_imm8, dm.clone(), *v as u32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn shr(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Shr_rm64_imm8, *dr, *v as u32)?,
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) if *sr == iced_x86::Register::CL => iced_x86::Instruction::with1(iced_x86::Code::Shr_rm64_CL, *dr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Shr_rm64_imm8, dm.clone(), *v as u32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn sar(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Sar_rm64_imm8, *dr, *v as u32)?,
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) if *sr == iced_x86::Register::CL => iced_x86::Instruction::with1(iced_x86::Code::Sar_rm64_CL, *dr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Sar_rm64_imm8, dm.clone(), *v as u32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn cmp(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Cmp_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Cmp_rm64_imm32, *dr, *v as i32)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Cmp_r64_rm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Cmp_rm64_r64, dm.clone(), *sr)?,
            (IcedOp::Mem(dm, _), IcedOp::Imm(v)) => iced_x86::Instruction::with2(iced_x86::Code::Cmp_rm64_imm32, dm.clone(), *v as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn cmp0(&mut self, ctx: &mut Context, cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let zero = MemArgKind::NoMem(ArgKind::Lit(0u64));
        let zero_ref: &dyn crate::out::arg::MemArg = &zero;
        self.cmp(ctx, cfg, op, zero_ref)
    }

    fn not(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let o = mem_kind_to_iced(&op.concrete_mem_kind());
        let instr = match &o {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Not_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Not_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn lea(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let sm = Self::op_to_mem(&s);
        self.encode_instr(iced_x86::Instruction::with2(iced_x86::Code::Lea_r64_m, dr, sm)?)
    }

    fn mul(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&a.concrete_mem_kind());
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Imul_r64_rm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Imul_r64_rm64, *dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn div(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match &s {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Div_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Div_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn idiv(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let s = mem_kind_to_iced(&b.concrete_mem_kind());
        let instr = match &s {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with1(iced_x86::Code::Idiv_rm64, *r)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with1(iced_x86::Code::Idiv_rm64, m.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn movsx(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let code = match Self::size_of(&s) {
            MemorySize::_8 => iced_x86::Code::Movsx_r64_rm8,
            MemorySize::_16 => iced_x86::Code::Movsx_r64_rm16,
            _ => iced_x86::Code::Movsx_r64_rm16,
        };
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(code, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(code, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn movzx(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let code = match Self::size_of(&s) {
            MemorySize::_8 => iced_x86::Code::Movzx_r64_rm8,
            MemorySize::_16 => iced_x86::Code::Movzx_r64_rm16,
            _ => iced_x86::Code::Movzx_r64_rm16,
        };
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(code, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(code, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn u32(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let o = mem_kind_to_iced(&op.concrete_mem_kind());
        let instr = match &o {
            IcedOp::Reg(r, _) => iced_x86::Instruction::with2(iced_x86::Code::And_rm64_imm32, *r, 0xffffffff_u32 as i32)?,
            IcedOp::Mem(m, _) => iced_x86::Instruction::with2(iced_x86::Code::And_rm64_imm32, m.clone(), 0xffffffff_u32 as i32)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn jcc(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, cond: crate::ConditionCode, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        // For indirect jcc we simulate with a conditional jump over a jmp
        // Actually jcc with register target isn't directly encodable; skip
        let _ = (cond, op);
        Ok(())
    }

    fn cmovcc64(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, cond: crate::ConditionCode, op: &(dyn crate::out::arg::MemArg + '_), val: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        use crate::ConditionCode::*;
        let d = mem_kind_to_iced(&op.concrete_mem_kind());
        let s = mem_kind_to_iced(&val.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let code = match cond {
            O => iced_x86::Code::Cmovo_r64_rm64,
            NO => iced_x86::Code::Cmovno_r64_rm64,
            B => iced_x86::Code::Cmovb_r64_rm64,
            NB => iced_x86::Code::Cmovae_r64_rm64,
            E => iced_x86::Code::Cmove_r64_rm64,
            NE => iced_x86::Code::Cmovne_r64_rm64,
            NA => iced_x86::Code::Cmovbe_r64_rm64,
            A => iced_x86::Code::Cmova_r64_rm64,
            S => iced_x86::Code::Cmovs_r64_rm64,
            NS => iced_x86::Code::Cmovns_r64_rm64,
            P => iced_x86::Code::Cmovp_r64_rm64,
            NP => iced_x86::Code::Cmovnp_r64_rm64,
            L => iced_x86::Code::Cmovl_r64_rm64,
            NL => iced_x86::Code::Cmovge_r64_rm64,
            NG => iced_x86::Code::Cmovle_r64_rm64,
            G => iced_x86::Code::Cmovg_r64_rm64,
        };
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(code, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(code, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn fadd(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(iced_x86::Code::Addsd_xmm_xmmm64, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(iced_x86::Code::Addsd_xmm_xmmm64, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn fsub(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(iced_x86::Code::Subsd_xmm_xmmm64, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(iced_x86::Code::Subsd_xmm_xmmm64, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn fmul(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(iced_x86::Code::Mulsd_xmm_xmmm64, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(iced_x86::Code::Mulsd_xmm_xmmm64, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn fdiv(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let dr = Self::op_to_reg(&d);
        let instr = match &s {
            IcedOp::Reg(sr, _) => iced_x86::Instruction::with2(iced_x86::Code::Divsd_xmm_xmmm64, dr, *sr)?,
            IcedOp::Mem(sm, _) => iced_x86::Instruction::with2(iced_x86::Code::Divsd_xmm_xmmm64, dr, sm.clone())?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn fmov(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
        let d = mem_kind_to_iced(&dest.concrete_mem_kind());
        let s = mem_kind_to_iced(&src.concrete_mem_kind());
        let instr = match (&d, &s) {
            (IcedOp::Reg(dr, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Movsd_xmm_xmmm64, *dr, *sr)?,
            (IcedOp::Reg(dr, _), IcedOp::Mem(sm, _)) => iced_x86::Instruction::with2(iced_x86::Code::Movsd_xmm_xmmm64, *dr, sm.clone())?,
            (IcedOp::Mem(dm, _), IcedOp::Reg(sr, _)) => iced_x86::Instruction::with2(iced_x86::Code::Movsd_xmmm64_xmm, dm.clone(), *sr)?,
            _ => return Ok(()),
        };
        self.encode_instr(instr)
    }

    fn db(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch, bytes: &[u8]) -> Result<(), Self::Error> {
        self.buf.extend_from_slice(bytes);
        self.ip += bytes.len() as u64;
        Ok(())
    }

    fn get_ip(&mut self, _ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        // CALL 0; POP rax pattern — caller must handle
        Ok(())
    }
}

#[cfg(feature = "iced")]
impl<L: Ord, Context> crate::out::Writer<L, Context> for IcedWriter<L> {
    fn set_label(
        &mut self,
        _ctx: &mut Context,
        _cfg: crate::X64Arch,
        s: L,
    ) -> Result<(), Self::Error> {
        self.labels.insert(s, self.buf.len());
        Ok(())
    }
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
                self.writer.set_label(ctx, self.arch, label)?;
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "iced"))]
mod tests {
    use super::*;
    use crate::out::Writer as _;

    #[test]
    fn set_label_records_byte_offset() {
        let arch = crate::X64Arch::default();
        let mut ctx = ();
        let mut w: IcedWriter<u32> = IcedWriter::new(0);

        // HLT is 1 byte (0xF4); emit two of them
        w.hlt(&mut ctx, arch).unwrap();
        w.hlt(&mut ctx, arch).unwrap();
        assert_eq!(w.offset(), 2);

        // Record label 99 at offset 2
        w.set_label(&mut ctx, arch, 99u32).unwrap();

        // Emit one more HLT
        w.hlt(&mut ctx, arch).unwrap();

        let (bytes, labels) = w.into_parts();
        assert_eq!(bytes.len(), 3);
        assert_eq!(labels[&99u32], 2);
    }
}
