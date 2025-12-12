//! Instruction output generation.
//!
//! This module provides traits and implementations for generating x86-64 assembly
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
// use alloc::boxed::Box;

/// Argument types for instruction operands.
pub mod arg;
/// Assembly text output implementations.
pub mod asm;

#[cfg(feature = "iced")]
mod iced;

/// Core trait for writing x86-64 instructions.
///
/// Implementors of this trait can emit individual x86-64 instructions.
/// The trait is designed to be object-safe where possible.
pub trait WriterCore<Context> {
    /// The error type returned by instruction emission methods.
    type Error: Error;

    /// Emits a HLT (halt) instruction.
    #[track_caller]
    fn hlt(&mut self, ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("hlt instruction not implemented")
    }
    
    /// Emits an XCHG (exchange) instruction.
    ///
    /// Exchanges the values in `dest` and `src`.
    #[track_caller]
    fn xchg(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("xchg instruction not implemented")
    }
    
    /// Emits a MOV (move) instruction.
    ///
    /// Copies the value from `src` to `dest`.
    #[track_caller]
    fn mov(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mov instruction not implemented")
    }
    
    /// Emits a SUB (subtract) instruction.
    ///
    /// Subtracts `b` from `a` and stores the result in `a`.
    #[track_caller]
    fn sub(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sub instruction not implemented")
    }
    
    /// Emits an ADD (add) instruction.
    ///
    /// Adds `b` to `a` and stores the result in `a`.
    #[track_caller]
    fn add(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("add instruction not implemented")
    }
    
    /// Emits a MOVSX (move with sign-extend) instruction.
    ///
    /// Copies the value from `src` to `dest` with sign extension.
    #[track_caller]
    fn movsx(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("movsx instruction not implemented")
    }
    
    /// Emits a MOVZX (move with zero-extend) instruction.
    ///
    /// Copies the value from `src` to `dest` with zero extension.
    #[track_caller]
    fn movzx(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("movzx instruction not implemented")
    }
    
    /// Emits a PUSH instruction.
    #[track_caller]
    fn push(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("push instruction not implemented")
    }
    
    /// Emits a POP instruction.
    #[track_caller]
    fn pop(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("pop instruction not implemented")
    }
    
    /// Emits a PUSHF (push flags) instruction.
    #[track_caller]
    fn pushf(&mut self, ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("pushf instruction not implemented")
    }
    
    /// Emits a POPF (pop flags) instruction.
    #[track_caller]
    fn popf(&mut self, ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("popf instruction not implemented")
    }
    
    /// Emits a CALL instruction.
    #[track_caller]
    fn call(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("call instruction not implemented")
    }
    
    /// Emits a JMP (unconditional jump) instruction.
    #[track_caller]
    fn jmp(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("jmp instruction not implemented")
    }
    
    /// Emits a CMP (compare) instruction.
    ///
    /// Compares `a` with `b` by computing `a - b` and setting flags.
    #[track_caller]
    fn cmp(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("cmp instruction not implemented")
    }
    
    /// Emits a CMP (compare with zero) instruction.
    #[track_caller]
    fn cmp0(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("cmp0 instruction not implemented")
    }
    
    /// Emits a CMOVcc (conditional move) instruction for 64-bit operands.
    #[track_caller]
    fn cmovcc64(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _cond: ConditionCode,
        _op: &(dyn MemArg + '_),
        _val: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("cmovcc64 instruction not implemented")
    }
    
    /// Emits a Jcc (conditional jump) instruction.
    #[track_caller]
    fn jcc(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _cond: ConditionCode,
        _op: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("jcc instruction not implemented")
    }
    
    /// Emits an instruction to truncate to 32 bits (AND with 0xffffffff).
    #[track_caller]
    fn u32(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("u32 instruction not implemented")
    }
    
    /// Emits a NOT (bitwise complement) instruction.
    #[track_caller]
    fn not(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("not instruction not implemented")
    }
    
    /// Emits a LEA (load effective address) instruction.
    #[track_caller]
    fn lea(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lea instruction not implemented")
    }

    /// Emits instructions to get the current instruction pointer.
    #[track_caller]
    fn get_ip(&mut self, ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("get_ip instruction not implemented")
    }
    
    /// Emits a RET (return) instruction.
    #[track_caller]
    fn ret(&mut self, ctx: &mut Context, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("ret instruction not implemented")
    }
    
    /// Emits a MOV instruction with a 64-bit immediate value.
    #[track_caller]
    fn mov64(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _r: &(dyn MemArg + '_),
        _val: u64,
    ) -> Result<(), Self::Error> {
        todo!("mov64 instruction not implemented")
    }
    
    /// Emits a MUL (unsigned multiply) instruction.
    #[track_caller]
    fn mul(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mul instruction not implemented")
    }
    
    /// Emits a DIV (unsigned divide) instruction.
    #[track_caller]
    fn div(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("div instruction not implemented")
    }
    
    /// Emits an IDIV (signed divide) instruction.
    #[track_caller]
    fn idiv(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("idiv instruction not implemented")
    }
    
    /// Emits an AND (bitwise AND) instruction.
    #[track_caller]
    fn and(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("and instruction not implemented")
    }
    
    /// Emits an OR (bitwise OR) instruction.
    #[track_caller]
    fn or(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("or instruction not implemented")
    }
    
    /// Emits an XOR (bitwise exclusive OR) instruction.
    #[track_caller]
    fn eor(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("eor instruction not implemented")
    }
    
    /// Emits a SHL (shift left) instruction.
    #[track_caller]
    fn shl(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("shl instruction not implemented")
    }
    
    /// Emits a SHR (shift right) instruction.
    #[track_caller]
    fn shr(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("shr instruction not implemented")
    }
    
    /// Emits a SAR (arithmetic shift right) instruction.
    ///
    /// Shifts `a` right by `b` bits, preserving the sign bit.
    #[track_caller]
    fn sar(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sar instruction not implemented")
    }
    
    /// Emits an ADD instruction for floating point values.
    #[track_caller]
    fn fadd(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fadd instruction not implemented")
    }
    
    /// Emits a SUB instruction for floating point values.
    #[track_caller]
    fn fsub(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fsub instruction not implemented")
    }
    
    /// Emits a MUL instruction for floating point values.
    #[track_caller]
    fn fmul(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmul instruction not implemented")
    }
    
    /// Emits a DIV instruction for floating point values.
    #[track_caller]
    fn fdiv(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fdiv instruction not implemented")
    }
    
    /// Emits a MOV instruction for floating point values.
    #[track_caller]
    fn fmov(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmov instruction not implemented")
    }
    
    /// Emits raw bytes as data.
    ///
    /// Generates a `.byte` directive (or equivalent) for the given bytes.
    #[track_caller]
    fn db(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _bytes: &[u8]) -> Result<(), Self::Error> {
        todo!("db directive not implemented")
    }
}

/// Extended writer trait with label support.
///
/// This trait extends [`WriterCore`] with methods for working with labels,
/// enabling structured control flow in generated code.
pub trait Writer<L, Context>: WriterCore<Context> {
    /// Sets a label at the current position.
    #[track_caller]
    fn set_label(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _s: L) -> Result<(), Self::Error> {
        todo!("set_label not implemented")
    }
    
    /// Emits a LEA instruction that loads the address of a label.
    #[track_caller]
    fn lea_label(&mut self, ctx: &mut Context, _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("lea_label not implemented")
    }

    /// Emits a CALL instruction to a label.
    #[track_caller]
    fn call_label(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _label: L) -> Result<(), Self::Error> {
        todo!("call_label not implemented")
    }

    /// Emits an unconditional jump to a label.
    #[track_caller]
    fn jmp_label(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _label: L) -> Result<(), Self::Error> {
        todo!("jmp_label not implemented")
    }
    
    /// Emits a conditional jump to a label.
    #[track_caller]
    fn jcc_label(&mut self, ctx: &mut Context, _cfg: crate::X64Arch, _cc: crate::ConditionCode, _label: L) -> Result<(), Self::Error> {
        todo!("jcc_label not implemented")
    }
}
#[macro_export]
macro_rules! writer_dispatch {
    ($( [ $($t:tt)* ] [$($u:tt)*] $ty:ty => $e:ty [$l:ty]),*) => {
        const _: () = {
            $(
                impl<$($t)*, $($u)*> $crate::out::WriterCore for $ty{
                    type Error = $e;
                    fn hlt(&mut self, ctx: &mut Context, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::hlt(&mut **self, ctx, cfg)
                    }
                    fn xchg(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::xchg(&mut **self, ctx, cfg, dest, src)
                    }
                    fn push(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::push(&mut **self, ctx, cfg, op)
                    }
                    fn pop(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::pop(&mut **self, ctx, cfg, op)
                    }
                    fn pushf(&mut self, ctx: &mut Context, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::pushf(&mut **self, ctx, cfg)
                    }
                    fn popf(&mut self, ctx: &mut Context, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::popf(&mut **self, ctx, cfg)
                    }
                    fn call(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::call(&mut **self, ctx, cfg,op)
                    }
                    fn jmp(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::jmp(&mut **self, ctx, cfg,op)
                    }
                    fn cmp(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::cmp(&mut **self, ctx, cfg,a,b)
                    }
                    fn cmp0(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::cmp0(&mut **self, ctx, cfg,op)
                    }
                    fn cmovcc64(&mut self, ctx: &mut Context, cfg: $crate::X64Arch,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_),val: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::cmovcc64(&mut **self, ctx, cfg,cc,op,val)
                    }
                    fn jcc(&mut self, ctx: &mut Context, cfg: $crate::X64Arch,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::jcc(&mut **self, ctx, cfg,cc,op)
                    }
                    fn lea(&mut self, ctx: &mut Context, cfg: $crate::X64Arch,
                        dest: &(dyn $crate::out::arg::MemArg + '_),
                        src: &(dyn $crate::out::arg::MemArg + '_),
                  
                    ) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::WriterCore<Context>>::lea(&mut **self, ctx, cfg, dest, src)
                    }

                    fn get_ip(&mut self, ctx: &mut Context, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::get_ip(&mut **self, ctx, cfg)
                    }
                    fn ret(&mut self, ctx: &mut Context, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::ret(&mut **self, ctx, cfg)
                    }
                    fn mov64(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, r: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mov64(&mut **self, ctx, cfg,r,val)
                    }
                    fn mov(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mov(&mut **self, ctx, cfg,dest,src)
                    }
                    fn u32(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::u32(&mut **self, ctx, cfg,op)
                    }
                    fn not(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::not(&mut **self, ctx, cfg,op)
                    }
                    fn mul(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::mul(&mut **self, ctx, cfg,a,b)
                    }
                    fn div(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::div(&mut **self, ctx, cfg,a,b)
                    }
                    fn idiv(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::idiv(&mut **self, ctx, cfg,a,b)
                    }
                    fn and(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::and(&mut **self, ctx, cfg,a,b)
                    }
                    fn or(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::or(&mut **self, ctx, cfg,a,b)
                    }
                    fn eor(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::eor(&mut **self, ctx, cfg,a,b)
                    }
                    fn shl(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::shl(&mut **self, ctx, cfg,a,b)
                    }
                    fn shr(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::shr(&mut **self, ctx, cfg,a,b)
                    }
                    fn sar(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sar(&mut **self, ctx, cfg,a,b)
                    }
                    fn sub(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::sub(&mut **self, ctx, cfg,a,b)
                    }
                    fn add(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::add(&mut **self, ctx, cfg,a,b)
                    }
                    fn movsx(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::movsx(&mut **self, ctx, cfg,dest,src)
                    }
                    fn movzx(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::movzx(&mut **self, ctx, cfg,dest,src)
                    }
                    fn fadd(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fadd(&mut **self, ctx, cfg,dest,src)
                    }
                    fn fsub(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fsub(&mut **self, ctx, cfg,dest,src)
                    }
                    fn fmul(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fmul(&mut **self, ctx, cfg,dest,src)
                    }
                    fn fdiv(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fdiv(&mut **self, ctx, cfg,dest,src)
                    }
                    fn fmov(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::fmov(&mut **self, ctx, cfg,dest,src)
                    }
                    fn db(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, bytes: &[u8]) -> $crate::__::core::result::Result<(), Self::Error>{
                        <$ty as $crate::out::WriterCore<Context>>::db(&mut **self, ctx, cfg,bytes)
                    }
                }
                impl<$($t)*>$crate::out::Writer<$l, Context> for $ty{

                    fn set_label(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, s: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::Writer<$l, Context>>::set_label(&mut **self, ctx, cfg, s)
                    }
                    fn lea_label(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                       <$ty as $crate::out::Writer<$l, Context>>::lea_label(&mut **self, ctx, cfg, dest, label)
                    }
                    fn call_label(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::Writer<$l, Context>>::call_label(&mut **self, ctx, cfg, label)
                    }
                    fn jmp_label(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::Writer<$l, Context>>::jmp_label(&mut **self, ctx, cfg, label)
                    }
                    fn jcc_label(&mut self, ctx: &mut Context, cfg: $crate::X64Arch, cc: $crate::ConditionCode, label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        <$ty as $crate::out::Writer<$l, Context>>::jcc_label(&mut **self, ctx, cfg, cc, label)
                    }

                }
            )*
        };
    };
}
writer_dispatch!(
    [ T: Writer<L, Context> + ?Sized, L ] [T: WriterCore<Context> + ?Sized] &'_ mut T => T::Error [L]
);
#[cfg(feature = "alloc")]
writer_dispatch!(
    [ T: Writer<L, Context> + ?Sized, L ] [T: WriterCore<Context> + ?Sized] ::alloc::boxed::Box<T> => T::Error [L]
);
