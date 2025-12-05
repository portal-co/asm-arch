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
                
                fn brk(&mut self, _cfg: $crate::AArch64Arch, imm: u16) -> $crate::__::core::result::Result<(),Self::Error>{
                    $crate::__::core::write!(self,"brk #{imm}\n")
                }
                
                fn mov(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mov {dest}, {src}\n")
                }
                
                fn str(&mut self, cfg: $crate::AArch64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let src = src.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"str {src}, {mem}\n")
                }
                
                fn ldr(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"ldr {dest}, {mem}\n")
                }
                
                fn stp(&mut self, cfg: $crate::AArch64Arch, src1: &(dyn $crate::out::arg::MemArg + '_), src2: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let src1 = src1.mem_display(cfg.into());
                    let src2 = src2.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"stp {src1}, {src2}, {mem}\n")
                }
                
                fn ldp(&mut self, cfg: $crate::AArch64Arch, dest1: &(dyn $crate::out::arg::MemArg + '_), dest2: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest1 = dest1.mem_display(cfg.into());
                    let dest2 = dest2.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"ldp {dest1}, {dest2}, {mem}\n")
                }
                
                fn bl(&mut self, cfg: $crate::AArch64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"bl {target}\n")
                }
                
                fn br(&mut self, cfg: $crate::AArch64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"br {target}\n")
                }
                
                fn b(&mut self, cfg: $crate::AArch64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"b {target}\n")
                }
                
                fn cmp(&mut self, cfg: $crate::AArch64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"cmp {a}, {b}\n")
                }
                
                fn csel(&mut self, cfg: $crate::AArch64Arch, cond: $crate::ConditionCode, dest: &(dyn $crate::out::arg::MemArg + '_), true_val: &(dyn $crate::out::arg::MemArg + '_), false_val: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let true_val = true_val.mem_display(cfg.into());
                    let false_val = false_val.mem_display(cfg.into());
                    $crate::__::core::write!(self,"csel {dest}, {true_val}, {false_val}, {cond}\n")
                }
                
                fn bcond(&mut self, cfg: $crate::AArch64Arch, cond: $crate::ConditionCode, target: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"b.{cond} {target}\n")
                }
                
                fn adr(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"adr {dest}, {src}\n")
                }
                
                fn ret(&mut self, _cfg: $crate::AArch64Arch) -> $crate::__::core::result::Result<(), Self::Error>{
                    $crate::__::core::write!(self,"ret\n")
                }
                
                fn mrs_nzcv(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mrs {dest}, nzcv\n")
                }
                
                fn msr_nzcv(&mut self, cfg: $crate::AArch64Arch, src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"msr nzcv, {src}\n")
                }
                
                fn mov_imm(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    // Use movz/movk sequence for 64-bit immediates
                    $crate::__::core::write!(self,"movz {dest}, #{}, lsl #0\n", val & 0xFFFF)?;
                    if (val >> 16) != 0 {
                        $crate::__::core::write!(self,"movk {dest}, #{}, lsl #16\n", (val >> 16) & 0xFFFF)?;
                    }
                    if (val >> 32) != 0 {
                        $crate::__::core::write!(self,"movk {dest}, #{}, lsl #32\n", (val >> 32) & 0xFFFF)?;
                    }
                    if (val >> 48) != 0 {
                        $crate::__::core::write!(self,"movk {dest}, #{}, lsl #48\n", (val >> 48) & 0xFFFF)?;
                    }
                    Ok(())
                }
                
                fn mul(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mul {dest}, {a}, {b}\n")
                }
                
                fn udiv(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"udiv {dest}, {a}, {b}\n")
                }
                
                fn sdiv(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sdiv {dest}, {a}, {b}\n")
                }
                
                fn and(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"and {dest}, {a}, {b}\n")
                }
                
                fn orr(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"orr {dest}, {a}, {b}\n")
                }
                
                fn eor(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"eor {dest}, {a}, {b}\n")
                }
                
                fn lsl(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lsl {dest}, {a}, {b}\n")
                }
                
                fn lsr(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lsr {dest}, {a}, {b}\n")
                }
                
                fn sub(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sub {dest}, {a}, {b}\n")
                }
                
                fn add(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"add {dest}, {a}, {b}\n")
                }
                
                fn sxt(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sxtw {dest}, {src}\n")
                }
                
                fn uxt(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"uxtw {dest}, {src}\n")
                }
                
                fn mvn(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mvn {dest}, {src}\n")
                }
                
                fn fadd(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Simd);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fadd {dest}, {a}, {b}\n")
                }
                
                fn fsub(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Simd);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fsub {dest}, {a}, {b}\n")
                }
                
                fn fmul(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Simd);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fmul {dest}, {a}, {b}\n")
                }
                
                fn fdiv(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Simd);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fdiv {dest}, {a}, {b}\n")
                }
                
                fn fmov(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Simd);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"fmov {dest}, {src}\n")
                }
            }
            
            impl<L: Display> Writer<L> for $ty {
                fn set_label(&mut self, _cfg: $crate::AArch64Arch, s: L) -> $crate::__::core::result::Result<(), Self::Error> {
                    $crate::__::core::write!(self, "{s}:\n")
                }
                
                fn adr_label(&mut self, cfg: $crate::AArch64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: L) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"adr {dest}, {label}\n")
                }
                
                fn b_label(&mut self, _cfg: $crate::AArch64Arch, label: L) -> $crate::__::core::result::Result<(),Self::Error>{
                    $crate::__::core::write!(self,"b {label}\n")
                }
                
                fn bl_label(&mut self, _cfg: $crate::AArch64Arch, label: L) -> $crate::__::core::result::Result<(),Self::Error>{
                    $crate::__::core::write!(self,"bl {label}\n")
                }
            })*
        };
    };
}

writers!(Formatter<'_>, dyn Write + '_);
