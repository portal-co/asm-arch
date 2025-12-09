//! Desugaring wrapper for RISC-V memory operands.
//!
//! This module provides a wrapper around WriterCore implementations that automatically
//! desugars complex memory operands (with scaled offsets or large displacements) into
//! valid RISC-V instruction sequences.
//!
//! # Overview
//!
//! RISC-V memory instructions only support `base + disp` addressing where disp is a
//! 12-bit signed immediate (-2048 to 2047). Complex addressing modes from the x86_64
//! shim (like `base + offset*scale + disp`) need to be desugared into multiple
//! instructions:
//!
//! 1. Calculate effective address in a temporary register
//! 2. Use the temporary register as the base with displacement 0 (or small offset)
//!
//! # Usage
//!
//! Wrap any WriterCore implementation with DesugaringWriter to automatically handle
//! complex memory operands:
//!
//! ```ignore
//! use portal_solutions_asm_riscv64::{
//!     desugar::DesugaringWriter,
//!     out::asm::AsmWriter,
//!     RiscV64Arch,
//! };
//! use portal_pc_asm_common::types::reg::Reg;
//!
//! let mut output = String::new();
//! let mut writer = AsmWriter::new(&mut output);
//! let mut desugar = DesugaringWriter::new(&mut writer);
//!
//! let cfg = RiscV64Arch::default();
//! let dest = Reg(10); // a0
//!
//! // Complex memory operand with scaled offset
//! let mem = MemArgKind::Mem {
//!     base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
//!     offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 3)),
//!     disp: 8,
//!     size: MemorySize::_64,
//!     reg_class: RegisterClass::Gpr,
//! };
//!
//! // This automatically desugars to multiple instructions
//! desugar.ld(cfg, &dest, &mem)?;
//! ```
//!
//! # Desugaring Examples
//!
//! ## Scaled Offset
//!
//! ```text
//! Input:  ld x10, mem[base=x5, offset=x6, scale=3, disp=100]
//! Output: li   t3, 3          // Load shift amount
//!         sll  t6, x6, t3     // t6 = x6 << 3
//!         add  t6, x5, t6     // t6 = x5 + t6
//!         ld   x10, 100(t6)   // x10 = mem[t6 + 100]
//! ```
//!
//! ## Large Displacement
//!
//! ```text
//! Input:  ld x10, mem[base=x5, disp=4096]
//! Output: li   t6, 4096       // Load large displacement
//!         add  t6, x5, t6     // t6 = x5 + 4096
//!         ld   x10, 0(t6)     // x10 = mem[t6 + 0]
//! ```
//!
//! # Configuration
//!
//! By default, the desugaring wrapper uses t6 (x31) as the temporary register.
//! You can customize this with DesugarConfig:
//!
//! ```ignore
//! let config = DesugarConfig {
//!     temp_reg: Reg(28), // Use t3 instead
//! };
//! let mut desugar = DesugaringWriter::with_config(&mut writer, config);
//! ```

use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

use crate::{
    out::{
        arg::{ArgKind, MemArg, MemArgKind},
        WriterCore,
    },
    RiscV64Arch,
};

/// Configuration for the desugaring wrapper.
#[derive(Clone, Copy, Debug)]
pub struct DesugarConfig {
    /// Primary temporary register to use for address calculations.
    /// Default: x31 (t6) - last temporary register.
    pub temp_reg: Reg,
    /// Secondary temporary register to use when primary is in use.
    /// Default: x28 (t3) - another temporary register.
    pub temp_reg2: Reg,
}

impl Default for DesugarConfig {
    fn default() -> Self {
        Self {
            temp_reg: Reg(31),  // t6
            temp_reg2: Reg(28), // t3
        }
    }
}

/// Wrapper around WriterCore that desugars complex memory operands.
///
/// This wrapper intercepts memory instructions and desugars complex addressing
/// modes into valid RISC-V instruction sequences.
pub struct DesugaringWriter<'a, W: WriterCore + ?Sized> {
    /// The underlying writer.
    writer: &'a mut W,
    /// Configuration for desugaring.
    config: DesugarConfig,
}

impl<'a, W: WriterCore + ?Sized> DesugaringWriter<'a, W> {
    /// Creates a new desugaring wrapper with default configuration.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            config: DesugarConfig::default(),
        }
    }

    /// Creates a new desugaring wrapper with custom configuration.
    pub fn with_config(writer: &'a mut W, config: DesugarConfig) -> Self {
        Self { writer, config }
    }

    /// Checks if a displacement fits in 12 bits (RISC-V immediate range).
    fn fits_in_12_bits(disp: i32) -> bool {
        disp >= -2048 && disp < 2048
    }

    /// Desugars a memory operand into a simple base+disp form.
    ///
    /// Returns (base_reg, displacement) where base_reg might be the temp register
    /// if address calculation was needed.
    fn desugar_mem_operand(
        &mut self,
        arch: RiscV64Arch,
        mem: &MemArgKind<ArgKind>,
    ) -> Result<(Reg, i32), W::Error> {
        match mem {
            MemArgKind::NoMem(_) => {
                // This shouldn't be called for non-memory operands
                panic!("desugar_mem_operand called with NoMem variant")
            }
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size: _,
                reg_class: _,
            } => {
                // Extract base register
                let base_reg = match base {
                    ArgKind::Reg { reg, .. } => *reg,
                    ArgKind::Lit(val) => {
                        // Load literal into temp register
                        let temp = self.config.temp_reg;
                        self.writer.li(arch, &temp, *val)?;
                        temp
                    }
                };

                // Handle offset if present
                let effective_base = if let Some((offset_arg, scale)) = offset {
                    let temp = self.config.temp_reg;

                    // Get offset register
                    let offset_reg = match offset_arg {
                        ArgKind::Reg { reg, .. } => *reg,
                        ArgKind::Lit(val) => {
                            // Load literal offset into temp
                            self.writer.li(arch, &temp, *val)?;
                            temp
                        }
                    };

                    // Calculate: temp = offset_reg << scale
                    if *scale > 0 {
                        // Use sll with immediate - need to create a temp register with the shift amount
                        let shift_amount = self.config.temp_reg2;
                        self.writer.li(arch, &shift_amount, *scale as u64)?;
                        self.writer.sll(arch, &temp, &offset_reg, &shift_amount)?;
                    } else {
                        // No scaling, just move
                        self.writer.mv(arch, &temp, &offset_reg)?;
                    }

                    // Add base: temp = base + temp
                    self.writer.add(arch, &temp, &base_reg, &temp)?;

                    temp
                } else {
                    base_reg
                };

                // Handle large displacement
                if Self::fits_in_12_bits(*disp) {
                    Ok((effective_base, *disp))
                } else {
                    // Displacement too large, need to add it to the base
                    let temp = self.config.temp_reg;

                    // Load displacement into temp (or use existing effective_base)
                    if effective_base == temp {
                        // temp already has the effective address, add displacement using li + add
                        let temp2 = self.config.temp_reg2;
                        self.writer.li(arch, &temp2, (*disp as i64) as u64)?;
                        self.writer.add(arch, &temp, &temp, &temp2)?;
                    } else {
                        // Load displacement and add base
                        self.writer.li(arch, &temp, (*disp as i64) as u64)?;
                        self.writer.add(arch, &temp, &effective_base, &temp)?;
                    }

                    Ok((temp, 0))
                }
            }
        }
    }

    /// Helper to create a simple memory operand from base and displacement.
    fn simple_mem(base: Reg, disp: i32, size: MemorySize, reg_class: crate::RegisterClass) -> MemArgKind<ArgKind> {
        MemArgKind::Mem {
            base: ArgKind::Reg { reg: base, size },
            offset: None,
            disp,
            size,
            reg_class,
        }
    }

    /// Desugars a memory argument if needed.
    fn desugar_mem_arg(
        &mut self,
        arch: RiscV64Arch,
        mem_arg: &(dyn MemArg + '_),
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = mem_arg.concrete_mem_kind();

        match &concrete {
            MemArgKind::NoMem(_) => Ok(concrete),
            MemArgKind::Mem {
                offset: Some(_), // Has scaled offset - needs desugaring
                disp: _,
                size,
                reg_class,
                ..
            } => {
                // Has scaled offset - needs desugaring
                let (base, new_disp) = self.desugar_mem_operand(arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, *size, *reg_class))
            }
            MemArgKind::Mem {
                offset: None,
                disp,
                size,
                reg_class,
                ..
            } if !Self::fits_in_12_bits(*disp) => {
                // Large displacement - needs desugaring
                let (base, new_disp) = self.desugar_mem_operand(arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, *size, *reg_class))
            }
            _ => Ok(concrete), // Simple case - no desugaring needed
        }
    }
}

// Implement WriterCore for DesugaringWriter
// We forward most methods and only intercept memory operations
impl<'a, W: WriterCore + ?Sized> WriterCore for DesugaringWriter<'a, W> {
    type Error = W::Error;

    // Memory load/store instructions that need desugaring

    fn ld(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.ld(cfg, dest, &desugared_mem)
    }

    fn sd(
        &mut self,
        cfg: RiscV64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.sd(cfg, src, &desugared_mem)
    }

    fn lw(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.lw(cfg, dest, &desugared_mem)
    }

    fn sw(
        &mut self,
        cfg: RiscV64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.sw(cfg, src, &desugared_mem)
    }

    fn lb(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.lb(cfg, dest, &desugared_mem)
    }

    fn sb(
        &mut self,
        cfg: RiscV64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.sb(cfg, src, &desugared_mem)
    }

    fn lh(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.lh(cfg, dest, &desugared_mem)
    }

    fn sh(
        &mut self,
        cfg: RiscV64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.sh(cfg, src, &desugared_mem)
    }

    fn fld(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.fld(cfg, dest, &desugared_mem)
    }

    fn fsd(
        &mut self,
        cfg: RiscV64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.fsd(cfg, src, &desugared_mem)
    }

    // Forward all non-memory instructions directly to the underlying writer
    // (We only need to implement the trait - the default implementations will forward via todo!())
    
    fn ebreak(&mut self, cfg: RiscV64Arch) -> Result<(), Self::Error> {
        self.writer.ebreak(cfg)
    }

    fn mv(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.mv(cfg, dest, src)
    }

    fn add(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.add(cfg, dest, a, b)
    }

    fn sub(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.sub(cfg, dest, a, b)
    }

    fn addi(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
        imm: i32,
    ) -> Result<(), Self::Error> {
        self.writer.addi(cfg, dest, src, imm)
    }

    fn li(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        self.writer.li(cfg, dest, val)
    }

    fn sll(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.sll(cfg, dest, a, b)
    }

    // Arithmetic operations - M extension
    
    fn mul(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.mul(cfg, dest, a, b)
    }

    fn mulh(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.mulh(cfg, dest, a, b)
    }

    fn div(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.div(cfg, dest, a, b)
    }

    fn divu(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.divu(cfg, dest, a, b)
    }

    fn rem(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.rem(cfg, dest, a, b)
    }

    fn remu(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.remu(cfg, dest, a, b)
    }

    // Bitwise operations
    
    fn and(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.and(cfg, dest, a, b)
    }

    fn or(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.or(cfg, dest, a, b)
    }

    fn xor(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.xor(cfg, dest, a, b)
    }

    fn srl(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.srl(cfg, dest, a, b)
    }

    fn sra(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.sra(cfg, dest, a, b)
    }

    fn slt(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.slt(cfg, dest, a, b)
    }

    fn sltu(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.sltu(cfg, dest, a, b)
    }

    // Control flow operations
    
    fn jalr(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        base: &(dyn MemArg + '_),
        offset: i32,
    ) -> Result<(), Self::Error> {
        self.writer.jalr(cfg, dest, base, offset)
    }

    fn jal(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.jal(cfg, dest, target)
    }

    fn beq(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.beq(cfg, a, b, target)
    }

    fn bne(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.bne(cfg, a, b, target)
    }

    fn blt(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.blt(cfg, a, b, target)
    }

    fn bge(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.bge(cfg, a, b, target)
    }

    fn bltu(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.bltu(cfg, a, b, target)
    }

    fn bgeu(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.bgeu(cfg, a, b, target)
    }

    fn ret(&mut self, cfg: RiscV64Arch) -> Result<(), Self::Error> {
        self.writer.ret(cfg)
    }

    fn call(&mut self, cfg: RiscV64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.writer.call(cfg, target)
    }

    fn j(&mut self, cfg: RiscV64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.writer.j(cfg, target)
    }

    // Special operations
    
    fn lui(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        imm: u32,
    ) -> Result<(), Self::Error> {
        self.writer.lui(cfg, dest, imm)
    }

    fn auipc(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        imm: u32,
    ) -> Result<(), Self::Error> {
        self.writer.auipc(cfg, dest, imm)
    }

    // Floating-point operations
    
    fn fadd_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fadd_d(cfg, dest, a, b)
    }

    fn fsub_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fsub_d(cfg, dest, a, b)
    }

    fn fmul_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fmul_d(cfg, dest, a, b)
    }

    fn fdiv_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fdiv_d(cfg, dest, a, b)
    }

    fn fmov_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fmov_d(cfg, dest, src)
    }

    fn fcvt_d_l(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fcvt_d_l(cfg, dest, src)
    }

    fn fcvt_l_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.fcvt_l_d(cfg, dest, src)
    }
}

// Implement Writer trait for DesugaringWriter
// This enables label support - we simply forward to the underlying writer
impl<'a, W, L> crate::out::Writer<L> for DesugaringWriter<'a, W>
where
    W: crate::out::Writer<L> + ?Sized,
{
    fn set_label(&mut self, cfg: RiscV64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.set_label(cfg, label)
    }

    fn jal_label(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.jal_label(cfg, dest, label)
    }

    fn bcond_label(
        &mut self,
        cfg: RiscV64Arch,
        cond: crate::ConditionCode,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.bcond_label(cfg, cond, a, b, label)
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::String;
    use core::fmt::Write;

    #[test]
    fn test_desugar_scaled_offset() {
        let mut output = String::new();
        // Use a writer that implements WriterCore (via the writers! macro)
        // We need to use the output String through a mutable reference
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);
            
            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0
            
            // Memory operand: base=x5, offset=x6, scale=3, disp=8
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 3)),
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };
            
            // This should desugar to:
            // li   t3, 3
            // sll  t6, x6, t3
            // add  t6, x5, t6
            // ld   a0, 8(t6)
            let _ = desugar.ld(cfg, &dest, &mem);
        }
        
        // Check that output contains desugaring instructions
        assert!(output.contains("sll") || output.contains("slli"));
        assert!(output.contains("add"));
        assert!(output.contains("ld"));
    }

    #[test]
    fn test_desugar_large_displacement() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);
            
            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0
            
            // Memory operand with large displacement (>12 bits)
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: None,
                disp: 4096, // Too large for 12-bit immediate
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };
            
            // This should desugar to address calculation
            let _ = desugar.ld(cfg, &dest, &mem);
        }
        
        // Check that output contains desugaring instructions
        assert!(output.contains("li") || output.contains("addi"));
    }
}
