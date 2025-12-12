//! Instruction output generation for RISC-V 64-bit.
//!
//! This module provides traits and implementations for generating RISC-V 64-bit assembly
//! instructions. The core abstractions are:
//!
//! - [`WriterCore`]: Core trait for emitting individual instructions
//! - [`Writer`]: Extended trait that adds label support
//!
//! # Modules
//!
//! - [`arg`]: Argument and memory operand types
//! - [`asm`]: Assembly text output implementations

use core::error::Error;

use crate::{
    out::arg::MemArg,
    *,
};

/// Argument types for instruction operands.
pub mod arg;
/// Assembly text output implementations.
pub mod asm;

/// Core trait for writing RISC-V 64-bit instructions.
///
/// Implementors of this trait can emit individual RISC-V instructions.
/// The trait is designed to be object-safe where possible.
pub trait WriterCore<Context> {
    /// The error type returned by instruction emission methods.
    type Error: Error;

    /// Emits an EBREAK (breakpoint) instruction.
    #[track_caller]
    fn ebreak(&mut self, _cfg: crate::RiscV64Arch) -> Result<(), Self::Error> {
        todo!("ebreak instruction not implemented")
    }
    
    /// Emits a MV (move/copy register) pseudo-instruction.
    /// Implemented as: ADDI dest, src, 0
    #[track_caller]
    fn mv(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mv instruction not implemented")
    }
    
    /// Emits a SUB (subtract) instruction.
    #[track_caller]
    fn sub(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sub instruction not implemented")
    }
    
    /// Emits an ADD (add) instruction.
    #[track_caller]
    fn add(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("add instruction not implemented")
    }
    
    /// Emits an ADDI (add immediate) instruction.
    #[track_caller]
    fn addi(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
        _imm: i32,
    ) -> Result<(), Self::Error> {
        todo!("addi instruction not implemented")
    }
    
    /// Emits a SD (store doubleword) instruction.
    #[track_caller]
    fn sd(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _src: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sd instruction not implemented")
    }
    
    /// Emits an LD (load doubleword) instruction.
    #[track_caller]
    fn ld(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("ld instruction not implemented")
    }
    
    /// Emits a LW (load word) instruction.
    #[track_caller]
    fn lw(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lw instruction not implemented")
    }
    
    /// Emits a SW (store word) instruction.
    #[track_caller]
    fn sw(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _src: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sw instruction not implemented")
    }
    
    /// Emits an LB (load byte) instruction.
    #[track_caller]
    fn lb(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lb instruction not implemented")
    }
    
    /// Emits a SB (store byte) instruction.
    #[track_caller]
    fn sb(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _src: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sb instruction not implemented")
    }
    
    /// Emits an LH (load halfword) instruction.
    #[track_caller]
    fn lh(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lh instruction not implemented")
    }
    
    /// Emits a SH (store halfword) instruction.
    #[track_caller]
    fn sh(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _src: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sh instruction not implemented")
    }
    
    /// Emits a JALR (jump and link register) instruction.
    #[track_caller]
    fn jalr(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _base: &(dyn MemArg + '_),
        _offset: i32,
    ) -> Result<(), Self::Error> {
        todo!("jalr instruction not implemented")
    }
    
    /// Emits a JAL (jump and link) instruction.
    #[track_caller]
    fn jal(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("jal instruction not implemented")
    }
    
    /// Emits a BEQ (branch if equal) instruction.
    #[track_caller]
    fn beq(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("beq instruction not implemented")
    }
    
    /// Emits a BNE (branch if not equal) instruction.
    #[track_caller]
    fn bne(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("bne instruction not implemented")
    }
    
    /// Emits a BLT (branch if less than, signed) instruction.
    #[track_caller]
    fn blt(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("blt instruction not implemented")
    }
    
    /// Emits a BGE (branch if greater than or equal, signed) instruction.
    #[track_caller]
    fn bge(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("bge instruction not implemented")
    }
    
    /// Emits a BLTU (branch if less than, unsigned) instruction.
    #[track_caller]
    fn bltu(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("bltu instruction not implemented")
    }
    
    /// Emits a BGEU (branch if greater than or equal, unsigned) instruction.
    #[track_caller]
    fn bgeu(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("bgeu instruction not implemented")
    }
    
    /// Emits an AND (bitwise AND) instruction.
    #[track_caller]
    fn and(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("and instruction not implemented")
    }
    
    /// Emits an OR (bitwise OR) instruction.
    #[track_caller]
    fn or(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("or instruction not implemented")
    }
    
    /// Emits an XOR (bitwise exclusive OR) instruction.
    #[track_caller]
    fn xor(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("xor instruction not implemented")
    }
    
    /// Emits a SLL (shift left logical) instruction.
    #[track_caller]
    fn sll(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sll instruction not implemented")
    }
    
    /// Emits a SRL (shift right logical) instruction.
    #[track_caller]
    fn srl(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("srl instruction not implemented")
    }
    
    /// Emits a SRA (shift right arithmetic) instruction.
    #[track_caller]
    fn sra(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sra instruction not implemented")
    }
    
    /// Emits a SLT (set less than, signed) instruction.
    #[track_caller]
    fn slt(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("slt instruction not implemented")
    }
    
    /// Emits a SLTU (set less than, unsigned) instruction.
    #[track_caller]
    fn sltu(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sltu instruction not implemented")
    }
    
    /// Emits a LUI (load upper immediate) instruction.
    #[track_caller]
    fn lui(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _imm: u32,
    ) -> Result<(), Self::Error> {
        todo!("lui instruction not implemented")
    }
    
    /// Emits an AUIPC (add upper immediate to PC) instruction.
    #[track_caller]
    fn auipc(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _imm: u32,
    ) -> Result<(), Self::Error> {
        todo!("auipc instruction not implemented")
    }
    
    /// Emits a LI (load immediate) pseudo-instruction.
    /// Uses LUI + ADDI for large immediates.
    #[track_caller]
    fn li(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _val: u64,
    ) -> Result<(), Self::Error> {
        todo!("li pseudo-instruction not implemented")
    }
    
    /// Emits a RET (return) pseudo-instruction.
    /// Implemented as: JALR x0, x1, 0
    #[track_caller]
    fn ret(&mut self, _cfg: crate::RiscV64Arch) -> Result<(), Self::Error> {
        todo!("ret instruction not implemented")
    }
    
    /// Emits a CALL pseudo-instruction (for function calls).
    /// Implemented as: AUIPC + JALR sequence.
    #[track_caller]
    fn call(&mut self, _cfg: crate::RiscV64Arch, _target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("call pseudo-instruction not implemented")
    }
    
    /// Emits a J (jump) pseudo-instruction.
    /// Implemented as: JAL x0, target
    #[track_caller]
    fn j(&mut self, _cfg: crate::RiscV64Arch, _target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("j pseudo-instruction not implemented")
    }
    
    // M extension instructions (multiplication/division)
    
    /// Emits a MUL (multiply) instruction.
    #[track_caller]
    fn mul(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mul instruction not implemented")
    }
    
    /// Emits a MULH (multiply high, signed×signed) instruction.
    #[track_caller]
    fn mulh(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mulh instruction not implemented")
    }
    
    /// Emits a DIV (divide, signed) instruction.
    #[track_caller]
    fn div(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("div instruction not implemented")
    }
    
    /// Emits a DIVU (divide, unsigned) instruction.
    #[track_caller]
    fn divu(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("divu instruction not implemented")
    }
    
    /// Emits a REM (remainder, signed) instruction.
    #[track_caller]
    fn rem(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("rem instruction not implemented")
    }
    
    /// Emits a REMU (remainder, unsigned) instruction.
    #[track_caller]
    fn remu(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("remu instruction not implemented")
    }
    
    // F/D extension instructions (floating-point)
    
    /// Emits a FLD (floating-point load double) instruction.
    #[track_caller]
    fn fld(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fld instruction not implemented")
    }
    
    /// Emits a FSD (floating-point store double) instruction.
    #[track_caller]
    fn fsd(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _src: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fsd instruction not implemented")
    }
    
    /// Emits a FADD.D (floating-point add double) instruction.
    #[track_caller]
    fn fadd_d(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fadd.d instruction not implemented")
    }
    
    /// Emits a FSUB.D (floating-point subtract double) instruction.
    #[track_caller]
    fn fsub_d(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fsub.d instruction not implemented")
    }
    
    /// Emits a FMUL.D (floating-point multiply double) instruction.
    #[track_caller]
    fn fmul_d(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmul.d instruction not implemented")
    }
    
    /// Emits a FDIV.D (floating-point divide double) instruction.
    #[track_caller]
    fn fdiv_d(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fdiv.d instruction not implemented")
    }
    
    /// Emits a FMOV.D (floating-point move double) pseudo-instruction.
    /// Implemented as: FSGNJ.D dest, src, src
    #[track_caller]
    fn fmov_d(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmov.d pseudo-instruction not implemented")
    }
    
    /// Emits a FCVT.D.L (convert signed 64-bit integer to double) instruction.
    #[track_caller]
    fn fcvt_d_l(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fcvt.d.l instruction not implemented")
    }
    
    /// Emits a FCVT.L.D (convert double to signed 64-bit integer) instruction.
    #[track_caller]
    fn fcvt_l_d(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fcvt.l.d instruction not implemented")
    }
}

/// Extended writer trait with label support.
///
/// This trait extends [`WriterCore`] with methods for working with labels,
/// enabling structured control flow in generated code.
pub trait Writer<L, Context>: WriterCore<Context> {
    /// Sets a label at the current position.
    #[track_caller]
    fn set_label(&mut self, _cfg: crate::RiscV64Arch, _s: L) -> Result<(), Self::Error> {
        todo!("set_label not implemented")
    }
    
    /// Emits a JAL instruction to a label.
    #[track_caller]
    fn jal_label(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _dest: &(dyn MemArg + '_),
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("jal_label not implemented")
    }
    
    /// Emits a branch instruction to a label based on condition code.
    #[track_caller]
    fn bcond_label(
        &mut self,
        _cfg: crate::RiscV64Arch,
        _cond: ConditionCode,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("bcond_label not implemented")
    }
}

// Macro to forward WriterCore methods through Box/&mut T
#[macro_export]
macro_rules! writer_dispatch {
    ($( [ $($t:tt)* ] [$($u:tt)*] $ty:ty => $e:ty [$l:ty]),*) => {
        const _: () = {
            $(
                impl<$($u)*> $crate::out::WriterCore for $ty{
                    type Error = $e;
                    fn ebreak(&mut self, cfg: $crate::RiscV64Arch) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::ebreak(&mut **self, cfg)
                    }
                    fn mv(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::mv(&mut **self, cfg, dest, src)
                    }
                    fn sd(&mut self, cfg: $crate::RiscV64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::sd(&mut **self, cfg, src, mem)
                    }
                    fn ld(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::ld(&mut **self, cfg, dest, mem)
                    }
                    fn add(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::add(&mut **self, cfg, dest, a, b)
                    }
                    fn sub(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sub(&mut **self, cfg, dest, a, b)
                    }
                    fn mul(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mul(&mut **self, cfg, dest, a, b)
                    }
                    fn div(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::div(&mut **self, cfg, dest, a, b)
                    }
                    fn and(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::and(&mut **self, cfg, dest, a, b)
                    }
                    fn or(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::or(&mut **self, cfg, dest, a, b)
                    }
                    fn xor(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::xor(&mut **self, cfg, dest, a, b)
                    }
                    fn sll(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sll(&mut **self, cfg, dest, a, b)
                    }
                    fn srl(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::srl(&mut **self, cfg, dest, a, b)
                    }
                    fn ret(&mut self, cfg: $crate::RiscV64Arch) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::ret(&mut **self, cfg)
                    }
                    fn li(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::li(&mut **self, cfg, dest, val)
                    }
                }
                impl<$($t)*>$crate::out::Writer<$l> for $ty{
                    fn set_label(&mut self, cfg: $crate::RiscV64Arch, s: $l) -> Result<(), Self::Error> {
                        <$ty as $crate::out::Writer<$l, Context>>::set_label(&mut **self, cfg, s)
                    }
                    fn jal_label(&mut self, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: $l) -> Result<(), Self::Error> {
                       <$ty as $crate::out::Writer<$l, Context>>::jal_label(&mut **self, cfg, dest, label)
                    }
                }
            )*
        };
    };
}

writer_dispatch!(
    [ T: Writer<L> + ?Sized,L ] [T: WriterCore<Context> + ?Sized] &'_ mut T => T::Error [L]
);

#[cfg(feature = "alloc")]
writer_dispatch!(
    [ T: Writer<L> + ?Sized,L ] [T: WriterCore<Context> + ?Sized] ::alloc::boxed::Box<T> => T::Error [L]
);
