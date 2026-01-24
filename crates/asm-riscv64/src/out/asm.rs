//! Assembly text output implementations for RISC-V 64-bit.
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
            impl<Context> $crate::out::WriterCore<Context> for $ty{
                type Error = $crate::__::core::fmt::Error;

                fn ebreak(&mut self, _ctx: &mut Context, _cfg: $crate::RiscV64Arch) -> Result<(),Self::Error>{
                    $crate::__::core::write!(self,"ebreak\n")
                }

                fn mv(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mv {dest}, {src}\n")
                }

                fn sd(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let src = src.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sd {src}, {mem}\n")
                }

                fn ld(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"ld {dest}, {mem}\n")
                }

                fn lw(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lw {dest}, {mem}\n")
                }

                fn sw(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let src = src.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sw {src}, {mem}\n")
                }

                fn lb(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lb {dest}, {mem}\n")
                }

                fn sb(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let src = src.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sb {src}, {mem}\n")
                }

                fn lh(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lh {dest}, {mem}\n")
                }

                fn sh(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let src = src.mem_display(cfg.into());
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sh {src}, {mem}\n")
                }

                fn jalr(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), base: &(dyn $crate::out::arg::MemArg + '_), offset: i32) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let base = base.mem_display(cfg.into());
                    $crate::__::core::write!(self,"jalr {dest}, {base}, {offset}\n")
                }

                fn jal(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"jal {dest}, {target}\n")
                }

                fn beq(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"beq {a}, {b}, {target}\n")
                }

                fn bne(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"bne {a}, {b}, {target}\n")
                }

                fn blt(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"blt {a}, {b}, {target}\n")
                }

                fn bge(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"bge {a}, {b}, {target}\n")
                }

                fn bltu(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"bltu {a}, {b}, {target}\n")
                }

                fn bgeu(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"bgeu {a}, {b}, {target}\n")
                }

                fn and(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"and {dest}, {a}, {b}\n")
                }

                fn or(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"or {dest}, {a}, {b}\n")
                }

                fn xor(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"xor {dest}, {a}, {b}\n")
                }

                fn sll(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sll {dest}, {a}, {b}\n")
                }

                fn srl(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"srl {dest}, {a}, {b}\n")
                }

                fn sra(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sra {dest}, {a}, {b}\n")
                }

                fn slt(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"slt {dest}, {a}, {b}\n")
                }

                fn sltu(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sltu {dest}, {a}, {b}\n")
                }

                fn sub(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"sub {dest}, {a}, {b}\n")
                }

                fn add(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"add {dest}, {a}, {b}\n")
                }

                fn addi(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_), imm: i32) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let src = src.mem_display(cfg.into());
                    $crate::__::core::write!(self,"addi {dest}, {src}, {imm}\n")
                }

                fn lui(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), imm: u32) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"lui {dest}, {imm}\n")
                }

                fn auipc(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), imm: u32) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"auipc {dest}, {imm}\n")
                }

                fn ret(&mut self, _ctx: &mut Context, _cfg: $crate::RiscV64Arch) -> Result<(), Self::Error>{
                    $crate::__::core::write!(self,"ret\n")
                }

                fn call(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"call {target}\n")
                }

                fn j(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, target: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let target = target.mem_display(cfg.into());
                    $crate::__::core::write!(self,"j {target}\n")
                }

                fn li(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"li {dest}, {val}\n")
                }

                // M extension
                fn mul(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mul {dest}, {a}, {b}\n")
                }

                fn mulh(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"mulh {dest}, {a}, {b}\n")
                }

                fn div(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"div {dest}, {a}, {b}\n")
                }

                fn divu(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"divu {dest}, {a}, {b}\n")
                }

                fn rem(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"rem {dest}, {a}, {b}\n")
                }

                fn remu(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    $crate::__::core::write!(self,"remu {dest}, {a}, {b}\n")
                }

                // F/D extension
                fn fld(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let dest = dest.mem_display(opts);
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"fld {dest}, {mem}\n")
                }

                fn fsd(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, src: &(dyn $crate::out::arg::MemArg + '_), mem: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let src = src.mem_display(opts);
                    let mem = mem.mem_display(cfg.into());
                    $crate::__::core::write!(self,"fsd {src}, {mem}\n")
                }

                fn fadd_d(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fadd.d {dest}, {a}, {b}\n")
                }

                fn fsub_d(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fsub.d {dest}, {a}, {b}\n")
                }

                fn fmul_d(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fmul.d {dest}, {a}, {b}\n")
                }

                fn fdiv_d(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let dest = dest.mem_display(opts);
                    let a = a.mem_display(opts);
                    let b = b.mem_display(opts);
                    $crate::__::core::write!(self,"fdiv.d {dest}, {a}, {b}\n")
                }

                fn fmov_d(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let dest = dest.mem_display(opts);
                    let src = src.mem_display(opts);
                    $crate::__::core::write!(self,"fmv.d {dest}, {src}\n")
                }

                fn fcvt_d_l(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts_fp = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let opts_gpr = $crate::DisplayOpts::new(cfg);
                    let dest = dest.mem_display(opts_fp);
                    let src = src.mem_display(opts_gpr);
                    $crate::__::core::write!(self,"fcvt.d.l {dest}, {src}\n")
                }

                fn fcvt_l_d(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_)) -> Result<(), Self::Error>{
                    let opts_fp = $crate::DisplayOpts::with_reg_class(cfg, $crate::RegisterClass::Fp);
                    let opts_gpr = $crate::DisplayOpts::new(cfg);
                    let dest = dest.mem_display(opts_gpr);
                    let src = src.mem_display(opts_fp);
                    $crate::__::core::write!(self,"fcvt.l.d {dest}, {src}\n")
                }
            }

            impl<L: Display, Context> $crate::out::Writer<L, Context> for $ty {
                fn set_label(&mut self, _ctx: &mut Context, _cfg: $crate::RiscV64Arch, s: L) -> Result<(), Self::Error> {
                    $crate::__::core::write!(self, "{s}:\n")
                }

                fn jal_label(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, dest: &(dyn $crate::out::arg::MemArg + '_), label: L) -> Result<(),Self::Error>{
                    let dest = dest.mem_display(cfg.into());
                    $crate::__::core::write!(self,"jal {dest}, {label}\n")
                }

                fn bcond_label(&mut self, _ctx: &mut Context, cfg: $crate::RiscV64Arch, cond: $crate::ConditionCode, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_), label: L) -> Result<(),Self::Error>{
                    let a = a.mem_display(cfg.into());
                    let b = b.mem_display(cfg.into());
                    match cond {
                        $crate::ConditionCode::EQ => $crate::__::core::write!(self,"beq {a}, {b}, {label}\n"),
                        $crate::ConditionCode::NE => $crate::__::core::write!(self,"bne {a}, {b}, {label}\n"),
                        $crate::ConditionCode::LT => $crate::__::core::write!(self,"blt {a}, {b}, {label}\n"),
                        $crate::ConditionCode::GE => $crate::__::core::write!(self,"bge {a}, {b}, {label}\n"),
                        $crate::ConditionCode::LTU => $crate::__::core::write!(self,"bltu {a}, {b}, {label}\n"),
                        $crate::ConditionCode::GEU => $crate::__::core::write!(self,"bgeu {a}, {b}, {label}\n"),
                        $crate::ConditionCode::GT => $crate::__::core::write!(self,"bgt {a}, {b}, {label}\n"),
                        $crate::ConditionCode::LE => $crate::__::core::write!(self,"ble {a}, {b}, {label}\n"),
                        $crate::ConditionCode::GTU => $crate::__::core::write!(self,"bgtu {a}, {b}, {label}\n"),
                        $crate::ConditionCode::LEU => $crate::__::core::write!(self,"bleu {a}, {b}, {label}\n"),
                    }
                }
            })*
        };
    };
}

writers!(Formatter<'_>, dyn Write + '_);
