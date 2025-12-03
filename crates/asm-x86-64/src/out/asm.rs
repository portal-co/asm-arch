//! Assembly text output implementations.
//!
//! This module provides macro-based implementations of [`WriterCore`] and [`Writer`]
//! for types that implement [`core::fmt::Write`], enabling assembly code to be
//! written as text.

use super::*;
use core::fmt::{Display, Formatter, Write};

/// Implements [`WriterCore`] and [`Writer`] for the specified types.
///
/// This macro generates implementations that emit assembly instructions as text.
#[macro_export]
macro_rules! writers {
    ($($ty:ty),*) => {
        const _: () = {
            $(
            impl $crate::out::WriterCore for $ty{
                type Error = $crate::__::core::fmt::Error;
                fn hlt(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(),Self::Error>{
                    $crate::__::core::write!(self,"hlt\n")
                }
                fn xchg(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"xchg {dest}, {src}\n")
                }
                fn push(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"push {op}\n")
                }
                fn pop(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"pop {op}\n")
                }
                fn pushf(&mut self, _cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                    $crate::__::core::write!(self,"pushfq\n")
                }
                fn popf(&mut self, _cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                    $crate::__::core::write!(self,"popfq\n")
                }
                fn call(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"call {op}\n")
                }
                 fn jmp(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"jmp {op}\n")
                }
                fn cmp0(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"cmp {op}, 0\n")
                }
                fn cmovcc64(&mut self, cfg: $crate::X64Arch,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_),val:&(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                     let op = op.mem_display(cfg.into());
                     let val = val.mem_display(cfg.into());
                    $crate::__::core::write!(self,"cmov{cc} {op}, {val}\n")
                }
                fn jcc(&mut self, cfg: $crate::X64Arch,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"j{cc} {op}\n")
                }
                fn u32(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"and {op}, 0xffffffff\n")
                }
                fn lea(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lea {dest}, {src}")
                }
                fn mov(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                     let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mov {dest}, {src}\n")
                }

                fn get_ip(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(),Self::Error>{
                //   let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"call 1f\n1:\n")
                }
                fn ret(&mut self, cfg: $crate::X64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                    $crate::__::core::write!(self,"ret\n")
                }
                fn mov64(&mut self, cfg: $crate::X64Arch, r: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                    let r = r.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mov {r}, {val}\n")
                }
                fn not(&mut self, cfg: $crate::X64Arch, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display(cfg.into());
                    $crate::__::core::write!(self,"not {op}\n")
                }
                fn mul(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mul {a},{b}\n")
                }
                fn div(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"div {a},{b}\n")
                }
                fn idiv(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"idiv {a},{b}\n")
                }
                fn and(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"and {a},{b}\n")
                }
                fn or(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"or {a},{b}\n")
                }
                fn eor(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"eor {a},{b}\n")
                }
                fn shl(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"shl {a},{b}\n")
                }
                fn shr(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"shr {a},{b}\n")
                }
                fn sub(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sub {a},{b}\n")
                }
                fn movsx(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"movsx {dest},{src}\n")
                }
                fn movzx(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"movzx {dest},{src}\n")
                }
                fn fadd(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Xmm);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"addsd {dest},{src}\n")
                }
                fn fsub(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Xmm);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"subsd {dest},{src}\n")
                }
                fn fmul(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Xmm);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"mulsd {dest},{src}\n")
                }
                fn fdiv(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Xmm);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"divsd {dest},{src}\n")
                }
                fn fmov(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Xmm);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"movsd {dest},{src}\n")
                }
            }
            impl<L: Display> Writer<L> for $ty {
                 fn set_label(&mut self, cfg: $crate::X64Arch, s: L) -> $crate::__::core::result::Result<(), Self::Error> {
                    $crate::__::core::write!(self, "{s}:\n")
                }
                 fn lea_label(&mut self, cfg: $crate::X64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: L) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lea {dest}, {label}\n")
                }

            })*
        };
    };
}
writers!(Formatter<'_>, (dyn Write + '_));
