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
    fn hlt(&mut self, cfg: crate::X64Arch) -> Result<(), Self::Error>;
    
    /// Emits an XCHG (exchange) instruction.
    ///
    /// Exchanges the values in `dest` and `src`.
    fn xchg(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a MOV (move) instruction.
    ///
    /// Copies the value from `src` to `dest`.
    fn mov(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a SUB (subtract) instruction.
    ///
    /// Subtracts `b` from `a` and stores the result in `a`.
    fn sub(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a MOVSX (move with sign-extend) instruction.
    ///
    /// Copies the value from `src` to `dest` with sign extension.
    fn movsx(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a MOVZX (move with zero-extend) instruction.
    ///
    /// Copies the value from `src` to `dest` with zero extension.
    fn movzx(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a PUSH instruction.
    fn push(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a POP instruction.
    fn pop(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a CALL instruction.
    fn call(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a JMP (unconditional jump) instruction.
    fn jmp(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a CMP (compare with zero) instruction.
    fn cmp0(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a CMOVcc (conditional move) instruction for 64-bit operands.
    fn cmovcc64(
        &mut self,
        cfg: crate::X64Arch,
        cond: ConditionCode,
        op: &(dyn MemArg + '_),
        val: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a Jcc (conditional jump) instruction.
    fn jcc(
        &mut self,
        cfg: crate::X64Arch,
        cond: ConditionCode,
        op: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits an instruction to truncate to 32 bits (AND with 0xffffffff).
    fn u32(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a NOT (bitwise complement) instruction.
    fn not(&mut self, cfg: crate::X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    
    /// Emits a LEA (load effective address) instruction.
    fn lea(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;

    /// Emits instructions to get the current instruction pointer.
    fn get_ip(&mut self, cfg: crate::X64Arch) -> Result<(), Self::Error>;
    
    /// Emits a RET (return) instruction.
    fn ret(&mut self, cfg: crate::X64Arch) -> Result<(), Self::Error>;
    
    /// Emits a MOV instruction with a 64-bit immediate value.
    fn mov64(
        &mut self,
        cfg: crate::X64Arch,
        r: &(dyn MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error>;
    
    /// Emits a MUL (unsigned multiply) instruction.
    fn mul(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a DIV (unsigned divide) instruction.
    fn div(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits an IDIV (signed divide) instruction.
    fn idiv(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits an AND (bitwise AND) instruction.
    fn and(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits an OR (bitwise OR) instruction.
    fn or(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits an XOR (bitwise exclusive OR) instruction.
    fn eor(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a SHL (shift left) instruction.
    fn shl(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a SHR (shift right) instruction.
    fn shr(
        &mut self,
        cfg: crate::X64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits an ADD instruction for floating point values.
    fn fadd(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a SUB instruction for floating point values.
    fn fsub(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a MUL instruction for floating point values.
    fn fmul(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a DIV instruction for floating point values.
    fn fdiv(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
    
    /// Emits a MOV instruction for floating point values.
    fn fmov(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error>;
}

/// Extended writer trait with label support.
///
/// This trait extends [`WriterCore`] with methods for working with labels,
/// enabling structured control flow in generated code.
pub trait Writer<L>: WriterCore {
    /// Sets a label at the current position.
    fn set_label(&mut self, cfg: crate::X64Arch, s: L) -> Result<(), Self::Error>;
    
    /// Emits a LEA instruction that loads the address of a label.
    fn lea_label(
        &mut self,
        cfg: crate::X64Arch,
        dest: &(dyn MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error>;
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
                    fn call(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::call(&mut **self, cfg,op)
                    }
                    fn jmp(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::jmp(&mut **self, cfg,op)
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
