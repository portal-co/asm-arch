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

/// Core trait for writing x86-64 instructions.
///
/// Implementors of this trait can emit individual x86-64 instructions.
/// The trait is designed to be object-safe where possible.
pub trait WriterCore {
    /// The error type returned by instruction emission methods.
    type Error: Error;

    /// Emits a HLT (halt) instruction.
    #[track_caller]
    fn hlt(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("hlt instruction not implemented")
    }
    
    /// Emits an XCHG (exchange) instruction.
    ///
    /// Exchanges the values in `dest` and `src`.
    #[track_caller]
    fn xchg(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("xchg instruction not implemented")
    }
    
    /// Emits a MOV (move) instruction.
    ///
    /// Copies the value from `src` to `dest`.
    #[track_caller]
    fn mov(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mov instruction not implemented")
    }
    
    /// Emits a SUB (subtract) instruction.
    ///
    /// Subtracts `b` from `a` and stores the result in `a`.
    #[track_caller]
    fn sub(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("sub instruction not implemented")
    }
    
    /// Emits an ADD (add) instruction.
    ///
    /// Adds `b` to `a` and stores the result in `a`.
    #[track_caller]
    fn add(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("add instruction not implemented")
    }
    
    /// Emits a MOVSX (move with sign-extend) instruction.
    ///
    /// Copies the value from `src` to `dest` with sign extension.
    #[track_caller]
    fn movsx(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("movsx instruction not implemented")
    }
    
    /// Emits a MOVZX (move with zero-extend) instruction.
    ///
    /// Copies the value from `src` to `dest` with zero extension.
    #[track_caller]
    fn movzx(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("movzx instruction not implemented")
    }
    
    /// Emits a PUSH instruction.
    #[track_caller]
    fn push(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("push instruction not implemented")
    }
    
    /// Emits a POP instruction.
    #[track_caller]
    fn pop(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("pop instruction not implemented")
    }
    
    /// Emits a PUSHF (push flags) instruction.
    #[track_caller]
    fn pushf(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("pushf instruction not implemented")
    }
    
    /// Emits a POPF (pop flags) instruction.
    #[track_caller]
    fn popf(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("popf instruction not implemented")
    }
    
    /// Emits a CALL instruction.
    #[track_caller]
    fn call(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("call instruction not implemented")
    }
    
    /// Emits a JMP (unconditional jump) instruction.
    #[track_caller]
    fn jmp(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("jmp instruction not implemented")
    }
    
    /// Emits a CMP (compare) instruction.
    ///
    /// Compares `a` with `b` by computing `a - b` and setting flags.
    #[track_caller]
    fn cmp(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("cmp instruction not implemented")
    }
    
    /// Emits a CMP (compare with zero) instruction.
    #[track_caller]
    fn cmp0(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("cmp0 instruction not implemented")
    }
    
    /// Emits a CMOVcc (conditional move) instruction for 64-bit operands.
    #[track_caller]
    fn cmovcc64(
        &mut self,
        _cfg: crate::X64Arch,
        _cond: ConditionCode,
        _op: &(dyn MemArg + '_),
        _val: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("cmovcc64 instruction not implemented")
    }
    
    /// Emits a Jcc (conditional jump) instruction.
    #[track_caller]
    fn jcc(
        &mut self,
        _cfg: crate::X64Arch,
        _cond: ConditionCode,
        _op: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("jcc instruction not implemented")
    }
    
    /// Emits an instruction to truncate to 32 bits (AND with 0xffffffff).
    #[track_caller]
    fn u32(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("u32 instruction not implemented")
    }
    
    /// Emits a NOT (bitwise complement) instruction.
    #[track_caller]
    fn not(&mut self, _cfg: crate::X64Arch, _op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        todo!("not instruction not implemented")
    }
    
    /// Emits a LEA (load effective address) instruction.
    #[track_caller]
    fn lea(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("lea instruction not implemented")
    }

    /// Emits instructions to get the current instruction pointer.
    #[track_caller]
    fn get_ip(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("get_ip instruction not implemented")
    }
    
    /// Emits a RET (return) instruction.
    #[track_caller]
    fn ret(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> {
        todo!("ret instruction not implemented")
    }
    
    /// Emits a MOV instruction with a 64-bit immediate value.
    #[track_caller]
    fn mov64(
        &mut self,
        _cfg: crate::X64Arch,
        _r: &(dyn MemArg + '_),
        _val: u64,
    ) -> Result<(), Self::Error> {
        todo!("mov64 instruction not implemented")
    }
    
    /// Emits a MUL (unsigned multiply) instruction.
    #[track_caller]
    fn mul(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("mul instruction not implemented")
    }
    
    /// Emits a DIV (unsigned divide) instruction.
    #[track_caller]
    fn div(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("div instruction not implemented")
    }
    
    /// Emits an IDIV (signed divide) instruction.
    #[track_caller]
    fn idiv(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("idiv instruction not implemented")
    }
    
    /// Emits an AND (bitwise AND) instruction.
    #[track_caller]
    fn and(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("and instruction not implemented")
    }
    
    /// Emits an OR (bitwise OR) instruction.
    #[track_caller]
    fn or(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("or instruction not implemented")
    }
    
    /// Emits an XOR (bitwise exclusive OR) instruction.
    #[track_caller]
    fn eor(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("eor instruction not implemented")
    }
    
    /// Emits a SHL (shift left) instruction.
    #[track_caller]
    fn shl(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("shl instruction not implemented")
    }
    
    /// Emits a SHR (shift right) instruction.
    #[track_caller]
    fn shr(
        &mut self,
        _cfg: crate::X64Arch,
        _a: &(dyn MemArg + '_),
        _b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("shr instruction not implemented")
    }
    
    /// Emits an ADD instruction for floating point values.
    #[track_caller]
    fn fadd(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fadd instruction not implemented")
    }
    
    /// Emits a SUB instruction for floating point values.
    #[track_caller]
    fn fsub(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fsub instruction not implemented")
    }
    
    /// Emits a MUL instruction for floating point values.
    #[track_caller]
    fn fmul(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fmul instruction not implemented")
    }
    
    /// Emits a DIV instruction for floating point values.
    #[track_caller]
    fn fdiv(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        todo!("fdiv instruction not implemented")
    }
    
    /// Emits a MOV instruction for floating point values.
    #[track_caller]
    fn fmov(
        &mut self,
        _cfg: crate::X64Arch,
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
pub trait Writer<L>: WriterCore {
    /// Sets a label at the current position.
    #[track_caller]
    fn set_label(&mut self, _cfg: crate::X64Arch, _s: L) -> Result<(), Self::Error> {
        todo!("set_label not implemented")
    }
    
    /// Emits a LEA instruction that loads the address of a label.
    #[track_caller]
    fn lea_label(
        &mut self,
        _cfg: crate::X64Arch,
        _dest: &(dyn MemArg + '_),
        _label: L,
    ) -> Result<(), Self::Error> {
        todo!("lea_label not implemented")
    }
}
#[macro_export]
macro_rules! writer_dispatch {
    ($( [ $($t:tt)* ] [$($u:tt)*] $ty:ty => $e:ty [$l:ty]),*) => {
        const _: () = {
            $(
                impl<$($u)*> $crate::out::WriterCore for $ty{
                    type Error = $e;
                    fn hlt(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::hlt(&mut **self, cfg)
                    }
                    fn xchg(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::xchg(&mut **self, cfg, dest, src)
                    }
                    fn push(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::push(&mut **self, cfg, op)
                    }
                    fn pop(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::pop(&mut **self, cfg, op)
                    }
                    fn pushf(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::pushf(&mut **self, cfg)
                    }
                    fn popf(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::popf(&mut **self, cfg)
                    }
                    fn call(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::call(&mut **self, cfg,op)
                    }
                    fn jmp(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::jmp(&mut **self, cfg,op)
                    }
                    fn cmp(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::cmp(&mut **self, cfg,a,b)
                    }
                    fn cmp0(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::cmp0(&mut **self, cfg,op)
                    }
                    fn cmovcc64(&mut self, cfg: $crate::X64Arch,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_),val: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::cmovcc64(&mut **self, cfg,cc,op,val)
                    }
                    fn jcc(&mut self, cfg: $crate::X64Arch,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::jcc(&mut **self, cfg,cc,op)
                    }
                    fn lea(
                        &mut self, cfg: $crate::X64Arch,
                        dest: &(dyn $crate::out::arg::MemArg + '_),
                        src: &(dyn $crate::out::arg::MemArg + '_),
                  
                    ) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::lea(&mut **self, cfg, dest, src)
                    }

                    fn get_ip(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::get_ip(&mut **self, cfg)
                    }
                    fn ret(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::ret(&mut **self, cfg)
                    }
                    fn mov64(&mut self, cfg: $crate::X64Arch, r: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::mov64(&mut **self, cfg,r,val)
                    }
                    fn mov(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::mov(&mut **self, cfg,dest,src)
                    }
                    fn u32(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::u32(&mut **self, cfg,op)
                    }
                    fn not(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::not(&mut **self, cfg,op)
                    }
                    fn mul(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::mul(&mut **self, cfg,a,b)
                    }
                    fn div(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::div(&mut **self, cfg,a,b)
                    }
                    fn idiv(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::idiv(&mut **self, cfg,a,b)
                    }
                    fn and(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::and(&mut **self, cfg,a,b)
                    }
                    fn or(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::or(&mut **self, cfg,a,b)
                    }
                    fn eor(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::eor(&mut **self, cfg,a,b)
                    }
                    fn shl(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::shl(&mut **self, cfg,a,b)
                    }
                    fn shr(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::shr(&mut **self, cfg,a,b)
                    }
                    fn sub(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::sub(&mut **self, cfg,a,b)
                    }
                    fn movsx(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::movsx(&mut **self, cfg,dest,src)
                    }
                    fn movzx(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::movzx(&mut **self, cfg,dest,src)
                    }
                    fn fadd(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::fadd(&mut **self, cfg,dest,src)
                    }
                    fn fsub(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::fsub(&mut **self, cfg,dest,src)
                    }
                    fn fmul(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::fmul(&mut **self, cfg,dest,src)
                    }
                    fn fdiv(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::fdiv(&mut **self, cfg,dest,src)
                    }
                    fn fmov(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::fmov(&mut **self, cfg,dest,src)
                    }
                }
                impl<$($t)*>$crate::out:: Writer<$l> for $ty{

                    fn set_label(&mut self, cfg: $crate::X64Arch, s: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::Writer::set_label(&mut **self, cfg, s)
                    }
                    fn lea_label(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                       $crate::out:: Writer::lea_label(&mut **self, cfg, dest, label)
                    }

                }
            )*
        };
    };
}
writer_dispatch!(
    [ T: Writer<L> + ?Sized,L ] [T: WriterCore + ?Sized] &'_ mut T => T::Error [L]
    // [ T: Writer<L> + ?Sized,L ] Box<T> => T::Error [L]
);
#[cfg(feature = "alloc")]
writer_dispatch!(
    [ T: Writer<L> + ?Sized,L ] [T: WriterCore + ?Sized] ::alloc::boxed::Box<T> => T::Error [L]
);
