//! Instruction output generation.
//!
//! This module provides traits and implementations for generating AArch64 assembly
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
    out::arg::{Arg, MemArg},
    *,
};

/// Argument types for instruction operands.
pub mod arg;
/// Assembly text output implementations.
pub mod asm;

/// Core trait for writing AArch64 instructions.
///
/// Implementors of this trait can emit individual AArch64 instructions.
/// The trait is designed to be object-safe where possible.
pub trait WriterCore<Context> {
    /// The error type returned by instruction emission methods.
    type Error: Error;

    /// Emits a BRK (breakpoint) instruction.
    #[track_caller]
    fn brk(&mut self, ctx: &mut Context, _cfg: crate::AArch64Arch, _imm: u16) -> Result<(), Self::Error> {
        todo!("brk instruction not implemented")
    }
    
    /// Emits a MOV (move) instruction.
    ///
    /// Copies the value from `src` to `dest`.
    #[track_caller]
    fn mov(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mov instruction not implemented")
    }
    
    /// Emits a SUB (subtract) instruction.
    ///
    /// Subtracts `b` from `a` and stores the result in `dest`.
    #[track_caller]
    fn sub(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sub instruction not implemented")
    }
    
    /// Emits an ADD (add) instruction.
    ///
    /// Adds `a` and `b`, stores the result in `dest`.
    #[track_caller]
    fn add(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("add instruction not implemented")
    }
    
    /// Emits a SXTB/SXTH/SXTW (sign-extend) instruction.
    #[track_caller]
    fn sxt(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sxt instruction not implemented")
    }
    
    /// Emits a UXTB/UXTH (zero-extend) instruction.
    #[track_caller]
    fn uxt(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("uxt instruction not implemented")
    }
    
    /// Emits a STR (store register) instruction.
    #[track_caller]
    fn str(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _src: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("str instruction not implemented")
    }
    
    /// Emits a LDR (load register) instruction.
    #[track_caller]
    fn ldr(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("ldr instruction not implemented")
    }
    
    /// Emits a STP (store pair) instruction.
    #[track_caller]
    fn stp(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _src1: &(dyn MemArg + '_),
        _src2: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("stp instruction not implemented")
    }
    
    /// Emits a LDP (load pair) instruction.
    #[track_caller]
    fn ldp(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest1: &(dyn MemArg + '_),
        _dest2: &(dyn MemArg + '_),
        _mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("ldp instruction not implemented")
    }
    
    /// Emits a BL (branch with link) instruction.
    #[track_caller]
    fn bl(&mut self, ctx: &mut Context, _cfg: crate::AArch64Arch, _target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("bl instruction not implemented")
    }
    
    /// Emits a BR (branch to register) instruction.
    #[track_caller]
    fn br(&mut self, ctx: &mut Context, _cfg: crate::AArch64Arch, _target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("br instruction not implemented")
    }
    
    /// Emits a B (unconditional branch) instruction.
    #[track_caller]
    fn b(&mut self, ctx: &mut Context, _cfg: crate::AArch64Arch, _target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("b instruction not implemented")
    }
    
    /// Emits a CMP (compare) instruction.
    ///
    /// Compares `a` with `b` by computing `a - b` and setting flags.
    #[track_caller]
    fn cmp(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("cmp instruction not implemented")
    }
    
    /// Emits a CSEL (conditional select) instruction.
    #[track_caller]
    fn csel(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _cond: ConditionCode,
        _dest: &(dyn MemArg + '_),
        _true_val: &(dyn MemArg + '_),
        _false_val: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("csel instruction not implemented")
    }
    
    /// Emits a B.cond (conditional branch) instruction.
    #[track_caller]
    fn bcond(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _cond: ConditionCode,
        _target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("bcond instruction not implemented")
    }
    
    /// Emits an AND (bitwise AND) instruction.
    #[track_caller]
    fn and(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("and instruction not implemented")
    }
    
    /// Emits an ORR (bitwise OR) instruction.
    #[track_caller]
    fn orr(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("orr instruction not implemented")
    }
    
    /// Emits an EOR (bitwise exclusive OR) instruction.
    #[track_caller]
    fn eor(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("eor instruction not implemented")
    }
    
    /// Emits a LSL (logical shift left) instruction.
    #[track_caller]
    fn lsl(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lsl instruction not implemented")
    }
    
    /// Emits a LSR (logical shift right) instruction.
    #[track_caller]
    fn lsr(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lsr instruction not implemented")
    }
    
    /// Emits an MVN (bitwise NOT) instruction.
    #[track_caller]
    fn mvn(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mvn instruction not implemented")
    }
    
    /// Emits an ADR (address of label) instruction.
    #[track_caller]
    fn adr(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("adr instruction not implemented")
    }
    
    /// Emits a RET (return) instruction.
    #[track_caller]
    fn ret(&mut self, ctx: &mut Context, _cfg: crate::AArch64Arch) -> Result<(), Self::Error> {
        todo!("ret instruction not implemented")
    }
    
    /// Emits an MRS NZCV (move from NZCV flags to register) instruction.
    #[track_caller]
    fn mrs_nzcv(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mrs_nzcv instruction not implemented")
    }
    
    /// Emits an MSR NZCV (move to NZCV flags from register) instruction.
    #[track_caller]
    fn msr_nzcv(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("msr_nzcv instruction not implemented")
    }
    
    /// Emits a MOVZ/MOVK sequence to load a 64-bit immediate.
    #[track_caller]
    fn mov_imm(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _val: u64,
    ) -> Result<(), Self::Error> {
        todo!("mov_imm instruction not implemented")
    }
    
    /// Emits a MUL (multiply) instruction.
    #[track_caller]
    fn mul(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mul instruction not implemented")
    }
    
    /// Emits a UDIV (unsigned divide) instruction.
    #[track_caller]
    fn udiv(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("udiv instruction not implemented")
    }
    
    /// Emits a SDIV (signed divide) instruction.
    #[track_caller]
    fn sdiv(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sdiv instruction not implemented")
    }
    
    /// Emits a FADD (floating-point add) instruction.
    #[track_caller]
    fn fadd(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fadd instruction not implemented")
    }
    
    /// Emits a FSUB (floating-point subtract) instruction.
    #[track_caller]
    fn fsub(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fsub instruction not implemented")
    }
    
    /// Emits a FMUL (floating-point multiply) instruction.
    #[track_caller]
    fn fmul(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmul instruction not implemented")
    }
    
    /// Emits a FDIV (floating-point divide) instruction.
    #[track_caller]
    fn fdiv(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fdiv instruction not implemented")
    }
    
    /// Emits a FMOV (floating-point move) instruction.
    #[track_caller]
    fn fmov(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmov instruction not implemented")
    }
}

/// Extended writer trait with label support.
///
/// This trait extends [`WriterCore`] with methods for working with labels,
/// enabling structured control flow in generated code.
pub trait Writer<L, Context>: WriterCore<Context> {
    /// Sets a label at the current position.
    #[track_caller]
    fn set_label(&mut self, ctx: &mut Context, _cfg: crate::AArch64Arch, _s: L) -> Result<(), Self::Error> {
        todo!("set_label not implemented")
    }
    
    /// Emits an ADR instruction that loads the address of a label.
    #[track_caller]
    fn adr_label(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _dest: &(dyn MemArg + '_),
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("adr_label not implemented")
    }
    
    /// Emits a B (branch) instruction to a label.
    #[track_caller]
    fn b_label(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("b_label not implemented")
    }
    
    /// Emits a BL (branch with link) instruction to a label.
    #[track_caller]
    fn bl_label(&mut self, ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("bl_label not implemented")
    }
}

#[macro_export]
macro_rules! writer_dispatch {
    ($( [ $($t:tt)* ] [$($u:tt)*] $ty:ty => $e:ty [$l:ty]),*) => {
        const _: () = {
            $(
                impl<$($t)*> $crate::out::WriterCore<Context> for $ty{
                    type Error = $e;
                    fn brk(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, imm: u16) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::brk(&mut **self, ctx, cfg, imm)
                    }
                    fn mov(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::mov(&mut **self, ctx, cfg, dest, src)
                    }
                    fn str(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::str(&mut **self, ctx, cfg, src, mem)
                    }
                    fn ldr(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::ldr(&mut **self, ctx, cfg, dest, mem)
                    }
                    fn stp(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, src1: &(dyn $crate::out::arg::MemArg + '_), src2: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::stp(&mut **self, ctx, cfg, src1, src2, mem)
                    }
                    fn ldp(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest1: &(dyn $crate::out::arg::MemArg + '_), dest2: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::ldp(&mut **self, ctx, cfg, dest1, dest2, mem)
                    }
                    fn bl(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::bl(&mut **self, ctx, cfg, target)
                    }
                    fn br(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::br(&mut **self, ctx, cfg, target)
                    }
                    fn b(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::b(&mut **self, ctx, cfg, target)
                    }
                    fn cmp(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::cmp(&mut **self, ctx, cfg, a, b)
                    }
                    fn csel(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, cond: $crate::ConditionCode, dest: &(dyn $crate::out::arg::MemArg + '_), true_val: &(dyn $crate::out::arg::MemArg + '_), false_val: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::csel(&mut **self, ctx, cfg, cond, dest, true_val, false_val)
                    }
                    fn bcond(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, cond: $crate::ConditionCode, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::bcond(&mut **self, ctx, cfg, cond, target)
                    }
                    fn adr(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::adr(&mut **self, ctx, cfg, dest, src)
                    }
                    fn ret(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::ret(&mut **self, ctx, cfg)
                    }
                    fn mov_imm(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mov_imm(&mut **self, ctx, cfg, dest, val)
                    }
                    fn mul(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mul(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn udiv(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::udiv(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn sdiv(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sdiv(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn and(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::and(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn orr(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::orr(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn eor(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::eor(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn lsl(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::lsl(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn lsr(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::lsr(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn sub(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sub(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn add(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::add(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn sxt(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sxt(&mut **self, ctx, cfg, dest, src)
                    }
                    fn uxt(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::uxt(&mut **self, ctx, cfg, dest, src)
                    }
                    fn mvn(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mvn(&mut **self, ctx, cfg, dest, src)
                    }
                    fn fadd(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fadd(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn fsub(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fsub(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn fmul(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fmul(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn fdiv(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fdiv(&mut **self, ctx, cfg, dest, a, b)
                    }
                    fn fmov(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fmov(&mut **self, ctx, cfg, dest, src)
                    }
                }
                impl<$($u)*>$crate::out::Writer<$l, Context> for $ty{
                    fn set_label(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, s: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::Writer<$l, Context>>::set_label(&mut **self, ctx, cfg, s)
                    }
                    fn adr_label(&mut self, ctx: &mut Context, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                       <$ty as $crate::out::Writer<$l, Context>>::adr_label(&mut **self, ctx, cfg, dest, label)
                    }
                }
            )*
        };
    };
}

writer_dispatch!(
    [ T: Writer<L, Context> + WriterCore<Context> + ?Sized, L, Context ] [] &'_ mut T => T::Error [L]
);

#[cfg(feature = "alloc")]
writer_dispatch!(
    [ T: Writer<L, Context> + WriterCore<Context> + ?Sized, L, Context ] [] ::alloc::boxed::Box<T> => T::Error [L]
);
