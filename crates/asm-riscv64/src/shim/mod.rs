//! x86-64 to RISC-V64 translation shim.
//!
//! This module provides a translation layer for x86-64 instructions to RISC-V64,
//! including a MemArg adapter and stack-based calling convention shim.
//!
//! # Architecture Notes
//!
//! The shim handles conversion between x86-64 and RISC-V64:
//! - **Memory addressing**: RISC-V uses simple base+displacement (12-bit signed)
//! - **Register mapping**: x86-64 has 16 GPRs, RISC-V has 32 (x0-x31)
//! - **Register classes**: Xmm → Fp, Gpr → Gpr
//! - **Calling convention**: Both can use stack-based returns with appropriate mapping
//!
//! # Performance Notes
//!
//! Some x86-64 instructions require multiple RISC-V instructions:
//! - **XCHG**: Multiple instructions (no atomic exchange without A extension)
//! - **Complex addressing**: RISC-V only supports base+imm12, scaled addressing needs extra instructions
//! - **Parity flags**: No direct equivalent

use core::task::Context;

use crate::out::arg::MemArg;
use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};
use portal_solutions_asm_x86_64::{
    ConditionCode as X64ConditionCode, X64Arch,
    out::{Writer as X64Writer, WriterCore as X64WriterCore, arg::MemArg as X64MemArg},
};

/// Label type for shim system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShimLabel(pub usize);

impl core::fmt::Display for ShimLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, ".Lshim_{}", self.0)
    }
}

/// Adapter that converts x86-64 MemArg to RISC-V64 MemArg.
pub struct MemArgAdapter<'a> {
    inner: &'a (dyn X64MemArg + 'a),
    arch: X64Arch,
}

impl<'a> MemArgAdapter<'a> {
    /// Creates a new adapter wrapping an x86-64 MemArg.
    pub fn new(inner: &'a (dyn X64MemArg + 'a), arch: X64Arch) -> Self {
        Self { inner, arch }
    }
}

impl<'a> crate::out::arg::MemArg for MemArgAdapter<'a> {
    fn mem_kind(
        &self,
        go: &mut (dyn FnMut(crate::out::arg::MemArgKind<&'_ (dyn crate::out::arg::Arg + '_)>) + '_),
    ) {
        use crate::out::arg::MemArgKind as RiscVMemArgKind;
        use portal_solutions_asm_x86_64::out::arg::MemArgKind as X64MemArgKind;

        let x64_kind = self.inner.concrete_mem_kind();

        match x64_kind {
            X64MemArgKind::NoMem(arg) => {
                let riscv_arg = convert_arg_kind(arg, self.arch);
                go(RiscVMemArgKind::NoMem(&riscv_arg));
            }
            X64MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => {
                let riscv_base = convert_arg_kind(base, self.arch);
                let riscv_offset =
                    offset.map(|(off, scale)| (convert_arg_kind(off, self.arch), scale));
                let riscv_disp = disp as i32;
                let riscv_reg_class = convert_register_class(reg_class);

                match &riscv_offset {
                    None => {
                        go(RiscVMemArgKind::Mem {
                            base: &riscv_base,
                            offset: None,
                            disp: riscv_disp,
                            size,
                            reg_class: riscv_reg_class,
                        });
                    }
                    Some((off, scale)) => {
                        go(RiscVMemArgKind::Mem {
                            base: &riscv_base,
                            offset: Some((off, *scale)),
                            disp: riscv_disp,
                            size,
                            reg_class: riscv_reg_class,
                        });
                    }
                }
            }
            _ => {
                let riscv_arg = crate::out::arg::ArgKind::Lit(0);
                go(RiscVMemArgKind::NoMem(&riscv_arg));
            }
        }
    }
}

/// Converts x86-64 ArgKind to RISC-V ArgKind with register mapping.
fn convert_arg_kind(
    arg: portal_solutions_asm_x86_64::out::arg::ArgKind,
    arch: X64Arch,
) -> crate::out::arg::ArgKind {
    use crate::out::arg::ArgKind as RiscVArgKind;
    use portal_solutions_asm_x86_64::out::arg::ArgKind as X64ArgKind;

    match arg {
        X64ArgKind::Reg { reg, size } => {
            let riscv_reg = map_x64_register_to_riscv(reg, arch);
            RiscVArgKind::Reg {
                reg: riscv_reg,
                size,
            }
        }
        X64ArgKind::Lit(val) => RiscVArgKind::Lit(val),
        _ => RiscVArgKind::Lit(0),
    }
}

/// Converts x86-64 RegisterClass to RISC-V RegisterClass.
fn convert_register_class(
    reg_class: portal_solutions_asm_x86_64::RegisterClass,
) -> crate::RegisterClass {
    use crate::RegisterClass as RiscVRegClass;
    use portal_solutions_asm_x86_64::RegisterClass as X64RegClass;

    match reg_class {
        X64RegClass::Gpr => RiscVRegClass::Gpr,
        X64RegClass::Xmm => RiscVRegClass::Fp,
        _ => RiscVRegClass::Gpr,
    }
}

/// Maps x86-64 registers to RISC-V registers.
///
/// This function implements the register mapping between x86-64 and RISC-V:
/// - RAX (0) → a0 (10) - argument/return register
/// - RCX (1) → a1 (11) - argument register
/// - RDX (2) → a2 (12) - argument register
/// - RBX (3) → s2 (18) - callee-saved register
/// - RSP (4) → sp (2) - stack pointer
/// - RBP (5) → s0/fp (8) - frame pointer
/// - RSI (6) → a3 (13) - argument register
/// - RDI (7) → a4 (14) - argument register
/// - R8-R15 → a5-a7, t0-t6, s3-s11
pub fn map_x64_register_to_riscv(reg: Reg, arch: X64Arch) -> Reg {
    // Handle APX registers if enabled
    if arch.apx {
        let r = reg.0;
        if r >= 16 {
            // Map APX registers to temporary/saved registers
            return Reg(20 + ((r - 16) % 12));
        }
    }

    match reg.0 {
        0 => Reg(10),  // RAX → a0
        1 => Reg(11),  // RCX → a1
        2 => Reg(12),  // RDX → a2
        3 => Reg(18),  // RBX → s2 (callee-saved)
        4 => Reg(2),   // RSP → sp
        5 => Reg(8),   // RBP → s0/fp (frame pointer)
        6 => Reg(13),  // RSI → a3
        7 => Reg(14),  // RDI → a4
        8 => Reg(15),  // R8 → a5
        9 => Reg(16),  // R9 → a6
        10 => Reg(17), // R10 → a7
        11 => Reg(5),  // R11 → t0
        12 => Reg(6),  // R12 → t1
        13 => Reg(7),  // R13 → t2
        14 => Reg(28), // R14 → t3
        15 => Reg(29), // R15 → t4
        // For SIMD/higher registers, pass through or map to available space
        n => Reg(n),
    }
}

/// Wrapper that translates x86-64 instructions to RISC-V64.
pub struct X64ToRiscV64Shim<W> {
    /// The underlying RISC-V64 writer.
    pub inner: W,
    /// RISC-V64 architecture configuration.
    pub riscv_cfg: crate::RiscV64Arch,
    /// Counter for generating unique shim labels.
    shim_counter: usize,
}

impl<W> X64ToRiscV64Shim<W> {
    /// Creates a new shim wrapping the given RISC-V64 writer.
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            riscv_cfg: crate::RiscV64Arch::rv64imfd(),
            shim_counter: 0,
        }
    }

    /// Creates a new shim with a specific RISC-V64 configuration.
    pub fn with_config(inner: W, riscv_cfg: crate::RiscV64Arch) -> Self {
        Self {
            inner,
            riscv_cfg,
            shim_counter: 0,
        }
    }

    /// Generates a unique shim label.
    fn next_shim_label(&mut self) -> ShimLabel {
        let label = ShimLabel(self.shim_counter);
        self.shim_counter += 1;
        label
    }
}

/// Translates x86-64 condition codes to RISC-V condition codes.
pub fn translate_condition(cc: X64ConditionCode) -> crate::ConditionCode {
    match cc {
        X64ConditionCode::E => crate::ConditionCode::EQ,
        X64ConditionCode::NE => crate::ConditionCode::NE,
        X64ConditionCode::B => crate::ConditionCode::LTU,
        X64ConditionCode::NB => crate::ConditionCode::GEU,
        X64ConditionCode::A => crate::ConditionCode::GTU,
        X64ConditionCode::NA => crate::ConditionCode::LEU,
        X64ConditionCode::L => crate::ConditionCode::LT,
        X64ConditionCode::NL => crate::ConditionCode::GE,
        X64ConditionCode::G => crate::ConditionCode::GT,
        X64ConditionCode::NG => crate::ConditionCode::LE,
        X64ConditionCode::O => crate::ConditionCode::NE, // Approximation
        X64ConditionCode::NO => crate::ConditionCode::EQ, // Approximation
        X64ConditionCode::S => crate::ConditionCode::LT, // Sign bit set
        X64ConditionCode::NS => crate::ConditionCode::GE, // Sign bit clear
        X64ConditionCode::P => crate::ConditionCode::EQ, // No parity equivalent
        X64ConditionCode::NP => crate::ConditionCode::NE, // No parity equivalent
        _ => crate::ConditionCode::EQ,
    }
}

impl<W: crate::out::Writer<ShimLabel, Context>, Context> X64WriterCore<Context>
    for X64ToRiscV64Shim<W>
{
    type Error = W::Error;

    fn hlt(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 HLT → RISC-V EBREAK
        self.inner.ebreak(ctx, self.riscv_cfg)
    }

    fn xchg(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // XCHG using temporary register
        let temp = Reg(30); // t5
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.mv(ctx, self.riscv_cfg, &temp, &dest_adapter)?;
        self.inner
            .mv(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)?;
        self.inner.mv(ctx, self.riscv_cfg, &src_adapter, &temp)?;
        Ok(())
    }

    fn mov(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        use crate::out::arg::MemArgKind;

        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);

        let dest_kind = dest_adapter.concrete_mem_kind();
        let src_kind = src_adapter.concrete_mem_kind();

        match (dest_kind, src_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                self.inner
                    .mv(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                self.inner
                    .ld(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                self.inner
                    .sd(ctx, self.riscv_cfg, &src_adapter, &dest_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                let temp = Reg(30); // t5
                self.inner.ld(ctx, self.riscv_cfg, &temp, &src_adapter)?;
                self.inner.sd(ctx, self.riscv_cfg, &temp, &dest_adapter)
            }
        }
    }

    fn sub(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .sub(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn add(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .add(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn movsx(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // Sign-extend - use load with sign extension or shift sequences
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        // Simplified: just move for now, proper sign-extension would need size info
        self.inner
            .mv(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)
    }

    fn movzx(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // Zero-extend - RISC-V loads are zero-extending by default
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner
            .mv(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)
    }

    fn push(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // PUSH: sp = sp - 8; [sp] = op
        let sp = Reg(2);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.addi(ctx, self.riscv_cfg, &sp, &sp, -8)?;
        self.inner.sd(
            ctx,
            self.riscv_cfg,
            &op_adapter,
            &crate::out::arg::MemArgKind::Mem {
                base: sp,
                offset: None,
                disp: 0,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            },
        )
    }

    fn pop(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // POP: op = [sp]; sp = sp + 8
        let sp = Reg(2);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.ld(
            ctx,
            self.riscv_cfg,
            &op_adapter,
            &crate::out::arg::MemArgKind::Mem {
                base: sp,
                offset: None,
                disp: 0,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            },
        )?;
        self.inner.addi(ctx, self.riscv_cfg, &sp, &sp, 8)
    }

    fn pushf(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // RISC-V doesn't have flags register - skip or use custom solution
        // For now, push zero as placeholder
        let sp = Reg(2);
        let zero = Reg(0);
        self.inner.addi(ctx, self.riscv_cfg, &sp, &sp, -8)?;
        self.inner.sd(
            ctx,
            self.riscv_cfg,
            &zero,
            &crate::out::arg::MemArgKind::Mem {
                base: sp,
                offset: None,
                disp: 0,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            },
        )
    }

    fn popf(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // RISC-V doesn't have flags register - skip or use custom solution
        let sp = Reg(2);
        self.inner.addi(ctx, self.riscv_cfg, &sp, &sp, 8)
    }

    fn call(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.call(ctx, self.riscv_cfg, &op_adapter)
    }

    fn jmp(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.j(ctx, self.riscv_cfg, &op_adapter)
    }

    fn cmp(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // CMP sets flags - RISC-V uses direct comparisons in branches
        // Store comparison result for later branches (not ideal but works)
        let temp = Reg(31); // t6 as comparison result holder
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .sub(ctx, self.riscv_cfg, &temp, &a_adapter, &b_adapter)
    }

    fn cmp0(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let temp = Reg(31); // t6
        let zero = Reg(0);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner
            .sub(ctx, self.riscv_cfg, &temp, &op_adapter, &zero)
    }

    fn cmovcc64(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        cond: X64ConditionCode,
        op: &(dyn X64MemArg + '_),
        val: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // Conditional move - use branch + move sequence
        let skip_label = self.next_shim_label();
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let val_adapter = MemArgAdapter::new(val, _cfg);
        let temp = Reg(30); // t5
        let zero = Reg(0);

        // Compare and branch past move if condition not met
        let riscv_cond = translate_condition(cond);
        // This is simplified - proper implementation needs condition inversion
        self.inner
            .mv(ctx, self.riscv_cfg, &op_adapter, &val_adapter)?;
        Ok(())
    }

    fn jcc(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        cond: X64ConditionCode,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // Conditional jump - needs comparison context from previous CMP
        let temp = Reg(31); // t6 (result from CMP)
        let zero = Reg(0);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let riscv_cond = translate_condition(cond);

        // Use temp register that holds comparison result
        match riscv_cond {
            crate::ConditionCode::EQ => {
                self.inner
                    .beq(ctx, self.riscv_cfg, &temp, &zero, &op_adapter)
            }
            crate::ConditionCode::NE => {
                self.inner
                    .bne(ctx, self.riscv_cfg, &temp, &zero, &op_adapter)
            }
            crate::ConditionCode::LT => {
                self.inner
                    .blt(ctx, self.riscv_cfg, &temp, &zero, &op_adapter)
            }
            crate::ConditionCode::GE => {
                self.inner
                    .bge(ctx, self.riscv_cfg, &temp, &zero, &op_adapter)
            }
            crate::ConditionCode::LTU => {
                self.inner
                    .bltu(ctx, self.riscv_cfg, &temp, &zero, &op_adapter)
            }
            crate::ConditionCode::GEU => {
                self.inner
                    .bgeu(ctx, self.riscv_cfg, &temp, &zero, &op_adapter)
            }
            _ => self
                .inner
                .bne(ctx, self.riscv_cfg, &temp, &zero, &op_adapter),
        }
    }

    fn u32(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // Zero upper 32 bits - use AND with mask or load word
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let temp = Reg(30);
        self.inner.li(ctx, self.riscv_cfg, &temp, 0xFFFFFFFF)?;
        self.inner
            .and(ctx, self.riscv_cfg, &op_adapter, &op_adapter, &temp)
    }

    fn not(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // Bitwise NOT - XOR with all 1s
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let temp = Reg(30);
        self.inner.li(ctx, self.riscv_cfg, &temp, !0u64)?;
        self.inner
            .xor(ctx, self.riscv_cfg, &op_adapter, &op_adapter, &temp)
    }

    fn lea(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // LEA - compute address
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        // Simplified - would need to extract base+disp and use ADDI
        self.inner
            .mv(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)
    }

    fn get_ip(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // Get instruction pointer - use AUIPC
        let ra = Reg(1);
        self.inner.auipc(ctx, self.riscv_cfg, &ra, 0)
    }

    fn ret(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        self.inner.ret(ctx, self.riscv_cfg)
    }

    fn mov64(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        r: &(dyn X64MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        let r_adapter = MemArgAdapter::new(r, _cfg);
        self.inner.li(ctx, self.riscv_cfg, &r_adapter, val)
    }

    fn mul(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .mul(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn div(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .divu(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn idiv(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .div(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn and(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .and(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn or(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .or(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn eor(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .xor(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn shl(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .sll(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn shr(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        self.inner
            .srl(ctx, self.riscv_cfg, &a_adapter, &a_adapter, &b_adapter)
    }

    fn fadd(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fadd_d(
            ctx,
            self.riscv_cfg,
            &dest_adapter,
            &dest_adapter,
            &src_adapter,
        )
    }

    fn fsub(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fsub_d(
            ctx,
            self.riscv_cfg,
            &dest_adapter,
            &dest_adapter,
            &src_adapter,
        )
    }

    fn fmul(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fmul_d(
            ctx,
            self.riscv_cfg,
            &dest_adapter,
            &dest_adapter,
            &src_adapter,
        )
    }

    fn fdiv(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fdiv_d(
            ctx,
            self.riscv_cfg,
            &dest_adapter,
            &dest_adapter,
            &src_adapter,
        )
    }

    fn fmov(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner
            .fmov_d(ctx, self.riscv_cfg, &dest_adapter, &src_adapter)
    }
}

impl<W: crate::out::Writer<ShimLabel, Context>, L, Context> X64Writer<L, Context>
    for X64ToRiscV64Shim<W>
where
    W: crate::out::Writer<L, Context>,
{
    fn set_label(&mut self, ctx: &mut Context, _cfg: X64Arch, s: L) -> Result<(), Self::Error> {
        self.inner.set_label(ctx, self.riscv_cfg, s)
    }

    fn lea_label(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        self.inner.jal_label(ctx, self.riscv_cfg, &Reg(0), label)?;
        Ok(())
    }
}
