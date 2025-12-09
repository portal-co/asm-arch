//! iced-x86 backend for assembling machine code.
#![allow(unused)]

#[cfg(feature = "iced")]
mod _inner {
    extern crate alloc;
    use alloc::collections::BTreeMap;
    use core::fmt::Display;

    use crate::out::{Writer, WriterCore};
    use crate::{ConditionCode, X64Arch};

    pub struct IcedX86Writer<L> {
        pub asm: iced_x86::code_asm::CodeAssembler,
        pub labels: BTreeMap<L, iced_x86::code_asm::CodeLabel>,
    }
}

/// Helper functions for translating iced-x86 components to crate types.
#[cfg(feature = "iced")]
mod helpers {
    use super::*;
    use crate::out::arg::{ArgKind, MemArgKind};
    use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

    /// Convert an iced register to our Reg type.
    pub fn iced_register_to_reg(reg: iced_x86::Register) -> Option<Reg> {
        reg.raw().try_into().ok().map(Reg)
    }

    /// Convert an iced operand to our MemArgKind.
    pub fn iced_operand_to_mem_arg_kind(
        instr: &iced_x86::Instruction,
        op_index: usize,
    ) -> Option<MemArgKind<ArgKind>> {
        if op_index >= instr.op_count() {
            return None;
        }
        match instr.op_kind(op_index) {
            iced_x86::OpKind::Register => {
                let reg = instr.op_register(op_index);
                iced_register_to_reg(reg).map(|r| {
                    let size = match reg.size() {
                        1 => MemorySize::_8,
                        2 => MemorySize::_16,
                        4 => MemorySize::_32,
                        8 => MemorySize::_64,
                        _ => MemorySize::_64, // default
                    };
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
                let base = instr.memory_base().and_then(iced_register_to_reg);
                let index = instr.memory_index().and_then(iced_register_to_reg);
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
                let offset = index.map(|idx| (idx, scale as u32));
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
pub fn default_instruction_handler<W, L>(
    writer: &mut W,
    arch: &crate::X64Arch,
    labels: &mut alloc::collections::BTreeMap<u64, L>,
    instr: &iced_x86::Instruction,
    ip: u64,
    raw_bytes: &[u8],
) -> Result<(), W::Error>
where
    W: crate::out::Writer<L>,
    L: From<u64> + Clone + core::fmt::Display,
{
    use helpers::*;
    use iced_x86::Mnemonic;

    let dest = op0(instr)
        .as_ref()
        .map(|m| m as &dyn crate::out::arg::MemArg);
    let src = op1(instr)
        .as_ref()
        .map(|m| m as &dyn crate::out::arg::MemArg);
    let val = op2(instr)
        .as_ref()
        .map(|m| m as &dyn crate::out::arg::MemArg);

    match instr.mnemonic() {
        Mnemonic::Mov => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.mov(*arch, d, s)?;
            }
        }
        Mnemonic::Add => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.add(*arch, a, b)?;
            }
        }
        Mnemonic::Sub => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.sub(*arch, a, b)?;
            }
        }
        Mnemonic::Cmp => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.cmp(*arch, a, b)?;
            }
        }
        Mnemonic::And => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.and(*arch, a, b)?;
            }
        }
        Mnemonic::Or => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.or(*arch, a, b)?;
            }
        }
        Mnemonic::Xor => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.eor(*arch, a, b)?;
            }
        }
        Mnemonic::Shl => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.shl(*arch, a, b)?;
            }
        }
        Mnemonic::Shr => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.shr(*arch, a, b)?;
            }
        }
        Mnemonic::Sar => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.sar(*arch, a, b)?;
            }
        }
        Mnemonic::Not => {
            if let Some(op) = dest {
                writer.not(*arch, op)?;
            }
        }
        Mnemonic::Lea => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.lea(*arch, d, s)?;
            }
        }
        Mnemonic::Movsx => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.movsx(*arch, d, s)?;
            }
        }
        Mnemonic::Movzx => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.movzx(*arch, d, s)?;
            }
        }
        Mnemonic::Push => {
            if let Some(op) = dest {
                writer.push(*arch, op)?;
            }
        }
        Mnemonic::Pop => {
            if let Some(op) = dest {
                writer.pop(*arch, op)?;
            }
        }
        Mnemonic::Pushf => {
            writer.pushf(*arch)?;
        }
        Mnemonic::Popf => {
            writer.popf(*arch)?;
        }
        Mnemonic::Call => {
            if instr.is_call_near() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.call_label(*arch, label)?;
            } else if let Some(op) = dest {
                writer.call(*arch, op)?;
            }
        }
        Mnemonic::Jmp => {
            if instr.is_jmp_near() || instr.is_jmp_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jmp_label(*arch, label)?;
            } else if let Some(op) = dest {
                writer.jmp(*arch, op)?;
            }
        }
        Mnemonic::Ret => {
            writer.ret(*arch)?;
        }
        Mnemonic::Hlt => {
            writer.hlt(*arch)?;
        }
        Mnemonic::Xchg => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.xchg(*arch, d, s)?;
            }
        }
        Mnemonic::Imul => {
            if let (Some(a), Some(b)) = (dest, src) {
                writer.mul(*arch, a, b)?;
            }
        }
        Mnemonic::Idiv => {
            if let Some(b) = src {
                writer.idiv(*arch, dest.unwrap_or(b), b)?;
            }
        }
        Mnemonic::Div => {
            if let Some(b) = src {
                writer.div(*arch, dest.unwrap_or(b), b)?;
            }
        }
        Mnemonic::Addsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fadd(*arch, d, s)?;
            }
        }
        Mnemonic::Subsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fsub(*arch, d, s)?;
            }
        }
        Mnemonic::Mulsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fmul(*arch, d, s)?;
            }
        }
        Mnemonic::Divsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fdiv(*arch, d, s)?;
            }
        }
        Mnemonic::Movsd => {
            if let (Some(d), Some(s)) = (dest, src) {
                writer.fmov(*arch, d, s)?;
            }
        }
        // Conditional moves
        Mnemonic::Cmovo => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::O, op, v)?;
            }
        }
        Mnemonic::Cmovno => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NO, op, v)?;
            }
        }
        Mnemonic::Cmovb => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::B, op, v)?;
            }
        }
        Mnemonic::Cmovae => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NB, op, v)?;
            }
        }
        Mnemonic::Cmove => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::E, op, v)?;
            }
        }
        Mnemonic::Cmovne => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NE, op, v)?;
            }
        }
        Mnemonic::Cmovbe => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NA, op, v)?;
            }
        }
        Mnemonic::Cmova => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::A, op, v)?;
            }
        }
        Mnemonic::Cmovs => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::S, op, v)?;
            }
        }
        Mnemonic::Cmovns => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NS, op, v)?;
            }
        }
        Mnemonic::Cmovp => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::P, op, v)?;
            }
        }
        Mnemonic::Cmovnp => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NP, op, v)?;
            }
        }
        Mnemonic::Cmovl => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::L, op, v)?;
            }
        }
        Mnemonic::Cmovge => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NL, op, v)?;
            }
        }
        Mnemonic::Cmovle => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::NG, op, v)?;
            }
        }
        Mnemonic::Cmovg => {
            if let (Some(op), Some(v)) = (dest, src) {
                writer.cmovcc64(*arch, crate::ConditionCode::G, op, v)?;
            }
        }
        // Conditional jumps
        Mnemonic::Jo => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::O, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::O, op)?;
            }
        }
        Mnemonic::Jno => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NO, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NO, op)?;
            }
        }
        Mnemonic::Jb => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::B, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::B, op)?;
            }
        }
        Mnemonic::Jae => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NB, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NB, op)?;
            }
        }
        Mnemonic::Je => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::E, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::E, op)?;
            }
        }
        Mnemonic::Jne => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NE, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NE, op)?;
            }
        }
        Mnemonic::Jbe => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NA, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NA, op)?;
            }
        }
        Mnemonic::Ja => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::A, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::A, op)?;
            }
        }
        Mnemonic::Js => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::S, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::S, op)?;
            }
        }
        Mnemonic::Jns => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NS, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NS, op)?;
            }
        }
        Mnemonic::Jp => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::P, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::P, op)?;
            }
        }
        Mnemonic::Jnp => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NP, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NP, op)?;
            }
        }
        Mnemonic::Jl => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::L, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::L, op)?;
            }
        }
        Mnemonic::Jge => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NL, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NL, op)?;
            }
        }
        Mnemonic::Jle => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::NG, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::NG, op)?;
            }
        }
        Mnemonic::Jg => {
            if instr.is_jcc_near() || instr.is_jcc_short() {
                let target = instr.near_branch64();
                let label = labels
                    .entry(target)
                    .or_insert_with(|| L::from(target))
                    .clone();
                writer.jcc_label(*arch, crate::ConditionCode::G, label)?;
            } else if let Some(op) = dest {
                writer.jcc(*arch, crate::ConditionCode::G, op)?;
            }
        }
        // For unsupported instructions, emit as raw bytes
        _ => {
            writer.db(*arch, raw_bytes)?;
        }
    }
    Ok(())
}

use crate::out::arg::{ArgKind, MemArgKind};
use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

// Helper: map our Reg to iced Register
fn reg_to_register(r: Reg) -> Option<iced_x86::Register> {
    iced_x86::Register::try_from(r.0 as u32).ok()
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

// Helper to convert our MemArgKind to iced Operand
fn to_iced_operand(mem: &MemArgKind<ArgKind>) -> iced_x86::code_asm::Operand {
    match mem {
        MemArgKind::NoMem(ArgKind::Reg { reg, size }) => match size {
            MemorySize::_64 => map_gpr64(*reg)
                .unwrap_or(iced_x86::code_asm::registers::gpr64::rax())
                .into(),
            MemorySize::_32 => map_gpr32(*reg)
                .unwrap_or(iced_x86::code_asm::registers::gpr32::eax())
                .into(),
            MemorySize::_16 => map_gpr16(*reg)
                .unwrap_or(iced_x86::code_asm::registers::gpr16::ax())
                .into(),
            MemorySize::_8 => map_gpr8(*reg)
                .unwrap_or(iced_x86::code_asm::registers::gpr8::al())
                .into(),
            _ => map_gpr64(*reg)
                .unwrap_or(iced_x86::code_asm::registers::gpr64::rax())
                .into(),
        },
        MemArgKind::NoMem(ArgKind::Lit(v)) => (*v).into(),
        MemArgKind::Mem {
            base,
            offset,
            disp,
            size,
            reg_class,
        } => {
            let base_r = if let ArgKind::Reg { reg, .. } = base {
                map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax())
            } else {
                iced_x86::code_asm::registers::gpr64::rax()
            };
            let mem_operand = if let Some((off, scale)) = offset {
                if let ArgKind::Reg { reg: off_reg, .. } = off {
                    let off_r =
                        map_gpr64(*off_reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax());
                    base_r + off_r * (*scale as i32) + (*disp as i32)
                } else {
                    base_r + (*disp as i32)
                }
            } else {
                base_r + (*disp as i32)
            };
            match (size, reg_class) {
                (MemorySize::_8, _) => iced_x86::code_asm::byte_ptr(mem_operand).into(),
                (MemorySize::_16, _) => iced_x86::code_asm::word_ptr(mem_operand).into(),
                (MemorySize::_32, _) => iced_x86::code_asm::dword_ptr(mem_operand).into(),
                (MemorySize::_64, _) => iced_x86::code_asm::qword_ptr(mem_operand).into(),
                (_, &crate::RegisterClass::Xmm) => {
                    iced_x86::code_asm::xmmword_ptr(mem_operand).into()
                }
                _ => iced_x86::code_asm::qword_ptr(mem_operand).into(),
            }
        }
    }
}

impl<L: Ord + Clone + Display> Writer<L> for IcedX86Writer<L> {
    type Error = iced_x86::IcedError;
    fn set_label(&mut self, _cfg: X64Arch, s: L) -> Result<(), Self::Error> {
        let mut lbl = self.asm.create_label();
        self.labels.insert(s, lbl);
        self.asm.set_label(&mut lbl);
        Ok(())
    }

    fn lea_label(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let mem = dest.concrete_mem_kind();
        let iced_dest = to_iced_operand(&mem);
        if let Some(&lbl) = self.labels.get(&label) {
            self.asm.lea(iced_dest, lbl)?;
        }
        Ok(())
    }

    fn call_label(&mut self, _cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        if let Some(&lbl) = self.labels.get(&label) {
            self.asm.call(lbl)?;
        }
        Ok(())
    }

    fn jmp_label(&mut self, _cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        if let Some(&lbl) = self.labels.get(&label) {
            self.asm.jmp(lbl)?;
        }
        Ok(())
    }

    fn jcc_label(
        &mut self,
        _cfg: X64Arch,
        cc: crate::ConditionCode,
        label: L,
    ) -> Result<(), Self::Error> {
        use crate::ConditionCode as CC;
        if let Some(&lbl) = self.labels.get(&label) {
            match cc {
                CC::E => self.asm.je(lbl)?,
                CC::NE => self.asm.jne(lbl)?,
                CC::B => self.asm.jb(lbl)?,
                CC::NB => self.asm.jnb(lbl)?,
                CC::A => self.asm.ja(lbl)?,
                CC::NA => self.asm.jna(lbl)?,
                CC::L => self.asm.jl(lbl)?,
                CC::NL => self.asm.jnl(lbl)?,
                CC::G => self.asm.jg(lbl)?,
                CC::NG => self.asm.jng(lbl)?,
                CC::O => self.asm.jo(lbl)?,
                CC::NO => self.asm.jno(lbl)?,
                CC::S => self.asm.js(lbl)?,
                CC::NS => self.asm.jns(lbl)?,
                CC::P => self.asm.jp(lbl)?,
                CC::NP => self.asm.jnp(lbl)?,
                _ => self.asm.jmp(lbl)?,
            }
        }
        Ok(())
    }
}

// -- WriterCore instruction implementations --
impl<L: Ord + Clone + Display> WriterCore for IcedX86Writer<L> {
    type Error = iced_x86::IcedError;

    fn hlt(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        self.asm.hlt()?;
        Ok(())
    }

    fn xchg(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.xchg(iced_d, iced_s)?;
        Ok(())
    }

    fn mov(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.mov(iced_d, iced_s)?;
        Ok(())
    }

    fn sub(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.sub(iced_a, iced_b)?;
        Ok(())
    }

    fn add(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.add(iced_a, iced_b)?;
        Ok(())
    }

    fn movsx(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.movsx(iced_d, iced_s)?;
        Ok(())
    }

    fn movzx(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.movzx(iced_d, iced_s)?;
        Ok(())
    }

    fn push(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.push(iced_o)?;
        Ok(())
    }

    fn pop(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.pop(iced_o)?;
        Ok(())
    }

    fn pushf(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        self.asm.pushf()?;
        Ok(())
    }

    fn popf(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        self.asm.popf()?;
        Ok(())
    }

    fn call(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.call(iced_o)?;
        Ok(())
    }

    fn jmp(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.jmp(iced_o)?;
        Ok(())
    }

    fn cmp(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.cmp(iced_a, iced_b)?;
        Ok(())
    }

    fn cmp0(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.cmp(iced_o, 0u64)?;
        Ok(())
    }

    fn cmovcc64(
        &mut self,
        _cfg: X64Arch,
        cond: crate::ConditionCode,
        op: &(dyn crate::out::arg::MemArg + '_),
        val: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a = op.concrete_mem_kind();
        let b = val.concrete_mem_kind();
        let iced_a = to_iced_operand(&a);
        let iced_b = to_iced_operand(&b);
        // map cond to conditional move - use cmov<conds>
        use crate::ConditionCode as CC;
        match cond {
            CC::E => self.asm.cmove(iced_a, iced_b)?,
            CC::NE => self.asm.cmovne(iced_a, iced_b)?,
            CC::B => self.asm.cmovb(iced_a, iced_b)?,
            CC::NB => self.asm.cmovnb(iced_a, iced_b)?,
            CC::A => self.asm.cmova(iced_a, iced_b)?,
            CC::NA => self.asm.cmovna(iced_a, iced_b)?,
            CC::L => self.asm.cmovl(iced_a, iced_b)?,
            CC::NL => self.asm.cmovnl(iced_a, iced_b)?,
            CC::G => self.asm.cmovg(iced_a, iced_b)?,
            CC::NG => self.asm.cmovng(iced_a, iced_b)?,
            CC::O => self.asm.cmovo(iced_a, iced_b)?,
            CC::NO => self.asm.cmovno(iced_a, iced_b)?,
            CC::S => self.asm.cmovs(iced_a, iced_b)?,
            CC::NS => self.asm.cmovns(iced_a, iced_b)?,
            CC::P => self.asm.cmovp(iced_a, iced_b)?,
            CC::NP => self.asm.cmovnp(iced_a, iced_b)?,
        }
        Ok(())
    }

    fn jcc(
        &mut self,
        _cfg: X64Arch,
        cond: crate::ConditionCode,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        use crate::ConditionCode as CC;
        match cond {
            CC::E => self.asm.je(iced_o)?,
            CC::NE => self.asm.jne(iced_o)?,
            CC::B => self.asm.jb(iced_o)?,
            CC::NB => self.asm.jnb(iced_o)?,
            CC::A => self.asm.ja(iced_o)?,
            CC::NA => self.asm.jna(iced_o)?,
            CC::L => self.asm.jl(iced_o)?,
            CC::NL => self.asm.jnl(iced_o)?,
            CC::G => self.asm.jg(iced_o)?,
            CC::NG => self.asm.jng(iced_o)?,
            CC::O => self.asm.jo(iced_o)?,
            CC::NO => self.asm.jno(iced_o)?,
            CC::S => self.asm.js(iced_o)?,
            CC::NS => self.asm.jns(iced_o)?,
            CC::P => self.asm.jp(iced_o)?,
            CC::NP => self.asm.jnp(iced_o)?,
        }
        Ok(())
    }

    fn u32(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        // and op, 0xffffffff
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.and(iced_o, iced_o, 0xffffffffu64)?;
        Ok(())
    }

    fn not(
        &mut self,
        _cfg: X64Arch,
        op: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let o = op.concrete_mem_kind();
        let iced_o = to_iced_operand(&o);
        self.asm.not(iced_o)?;
        Ok(())
    }

    fn lea(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.lea(iced_d, iced_s)?;
        Ok(())
    }

    fn get_ip(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        // use call/pop trick: create label and lea into reg? For simplicity, emit call 1f; 1: ; but CodeAssembler supports call with label
        let mut lbl = self.asm.create_label();
        self.asm.call(lbl)?;
        self.asm.set_label(&mut lbl);
        Ok(())
    }

    fn ret(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        self.asm.ret()?;
        Ok(())
    }

    fn mov64(
        &mut self,
        _cfg: X64Arch,
        r: &(dyn crate::out::arg::MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        let reg_kind = r.concrete_mem_kind();
        let iced_r = to_iced_operand(&reg_kind);
        self.asm.mov(iced_r, val)?;
        Ok(())
    }

    fn mul(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.imul(iced_a, iced_b)?;
        Ok(())
    }

    fn div(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.idiv(iced_b)?;
        Ok(())
    }

    fn idiv(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.idiv(iced_b)?;
        Ok(())
    }

    fn and(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.and(iced_a, iced_a, iced_b)?;
        Ok(())
    }

    fn or(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.or(iced_a, iced_a, iced_b)?;
        Ok(())
    }

    fn eor(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.xor(iced_a, iced_a, iced_b)?;
        Ok(())
    }

    fn shl(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.shl(iced_a, iced_b)?;
        Ok(())
    }

    fn shr(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.shr(iced_a, iced_b)?;
        Ok(())
    }

    fn sar(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn crate::out::arg::MemArg + '_),
        b: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let A = a.concrete_mem_kind();
        let B = b.concrete_mem_kind();
        let iced_a = to_iced_operand(&A);
        let iced_b = to_iced_operand(&B);
        self.asm.sar(iced_a, iced_b)?;
        Ok(())
    }

    fn fadd(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.addsd(iced_d, iced_s)?;
        Ok(())
    }

    fn fsub(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.subsd(iced_d, iced_s)?;
        Ok(())
    }

    fn fmul(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.mulsd(iced_d, iced_s)?;
        Ok(())
    }

    fn fdiv(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.divsd(iced_d, iced_s)?;
        Ok(())
    }

    fn fmov(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), Self::Error> {
        let d = dest.concrete_mem_kind();
        let s = src.concrete_mem_kind();
        let iced_d = to_iced_operand(&d);
        let iced_s = to_iced_operand(&s);
        self.asm.movsd(iced_d, iced_s)?;
        Ok(())
    }

    fn db(&mut self, _cfg: X64Arch, bytes: &[u8]) -> Result<(), Self::Error> {
        for &b in bytes {
            self.asm.byte(b)?;
        }
        Ok(())
    }
}

#[cfg(feature = "iced")]
pub use _inner::IcedX86Writer;

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
pub struct IcedFrontend<'a, W, L, H, D>
where
    W: crate::out::Writer<L> + 'a,
    H: FnMut(
        &mut W,
        &crate::X64Arch,
        &mut alloc::collections::BTreeMap<u64, L>,
        &iced_x86::Instruction,
        u64,
        &[u8],
    ) -> Result<(), W::Error>,
    D: FnMut(&[u8], u64) -> Option<usize>,
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
}

#[cfg(feature = "iced")]
impl<'a, W, L, H, D> IcedFrontend<'a, W, L, H, D>
where
    W: crate::out::Writer<L> + 'a,
    H: FnMut(
        &mut W,
        &crate::X64Arch,
        &mut alloc::collections::BTreeMap<u64, L>,
        &iced_x86::Instruction,
        u64,
        &[u8],
    ) -> Result<(), W::Error>,
    D: FnMut(&[u8], u64) -> Option<usize>,
{
    /// Create a new frontend.
    pub fn new(writer: &'a mut W, arch: crate::X64Arch, handler: H) -> Self {
        Self {
            writer,
            arch,
            labels: alloc::collections::BTreeMap::new(),
            handler,
            inline_data_hook: None,
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
            &crate::X64Arch,
            &mut alloc::collections::BTreeMap<u64, L>,
            &iced_x86::Instruction,
            u64,
            &[u8],
        ) -> Result<(), W::Error>,
        D,
    >
    where
        L: From<u64> + Clone + core::fmt::Display,
    {
        let handler =
            move |w: &mut W,
                  a: &crate::X64Arch,
                  labels: &mut alloc::collections::BTreeMap<u64, L>,
                  i: &iced_x86::Instruction,
                  ip: u64,
                  rb: &[u8]| { default_instruction_handler(w, a, labels, i, ip, rb) };
        IcedFrontend {
            writer,
            arch,
            labels: alloc::collections::BTreeMap::new(),
            handler,
            inline_data_hook: None,
        }
    }

    /// Process the provided bytes starting at the given virtual `base_ip`.
    ///
    /// This decodes instructions sequentially and calls the handler for each
    /// instruction. If an `inline_data_hook` is installed and claims a range
    /// of bytes, those bytes are emitted via `Writer::db` and decoding
    /// resumes after them.
    pub fn process_bytes(&mut self, base_ip: u64, bytes: &[u8]) -> Result<(), W::Error> {
        let mut pos: usize = 0;
        let len = bytes.len();
        while pos < len {
            let current_ip = base_ip + pos as u64;
            // Set label if this IP is a jump target
            if let Some(label) = self.labels.get(&current_ip) {
                self.writer.set_label(self.arch, label.clone())?;
            }

            // Inline-data hook takes precedence
            if let Some(hook) = &mut self.inline_data_hook {
                if let Some(n) = hook(&bytes[pos..], current_ip) {
                    let take = core::cmp::min(n, len - pos);
                    // Emit as raw data bytes
                    self.writer.db(self.arch, &bytes[pos..pos + take])?;
                    pos += take;
                    continue;
                }
            }

            // Decode one instruction starting at bytes[pos..]
            let slice = &bytes[pos..];
            let mut decoder =
                iced_x86::Decoder::with_ip(64, slice, current_ip, iced_x86::DecoderOptions::NONE);
            // If there are no bytes left for decoder (shouldn't happen), break
            if decoder.can_decode() == false {
                break;
            }
            // Decode one instruction
            let instr = decoder.decode();
            let consumed = decoder.position();
            if consumed == 0 {
                // Defensive: if decoder consumed nothing, treat remaining bytes
                // as data to avoid infinite loop.
                self.writer.db(self.arch, slice)?;
                break;
            }
            // Conservatively create and set label for this instruction start if not already present
            if !self.labels.contains_key(&current_ip) {
                let label = L::from(current_ip);
                self.labels.insert(current_ip, label.clone());
                self.writer.set_label(self.arch, label)?;
            }
            // Call handler to translate the iced instruction into Writer calls.
            (self.handler)(
                self.writer,
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
