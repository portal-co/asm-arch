//! Desugaring wrapper for AArch64 memory operands and instruction constraints.
//!
//! This module provides a wrapper around WriterCore implementations that automatically
//! desugars invalid memory operands and operands that violate AArch64 instruction constraints
//! into valid AArch64 instruction sequences.
//!
//! # Overview
//!
//! AArch64 has several constraints that require desugaring:
//!
//! 1. **Memory addressing**: AArch64 supports various addressing modes, but complex
//!    addressing with large displacements or scaled offsets may need desugaring.
//!
//! 2. **Immediate limits**: AArch64 instructions have limits on immediate operand sizes:
//!    - ADD/SUB immediates are limited to 12 bits (0-4095)
//!    - Some other instructions have similar constraints
//!
//! 3. **Computational instructions**: AArch64 computational instructions cannot take
//!    memory operands directly - they only operate on registers. Memory operands
//!    must be loaded into registers first.
//!
//! 4. **Literal operands**: Computational instructions cannot take literal operands
//!    directly - literals must be loaded into registers first.
//!
//! # Desugaring Examples
//!
//! ## Large Immediates in ADD
//!
//! ```text
//! Input:  add x10, x5, #5000  // 5000 > 4095 (12-bit limit)
//! Output: mov x16, #5000      // Load large immediate
//!         add x10, x5, x16    // Perform addition
//! ```
//!
//! ## Memory Operands in Arithmetic
//!
//! ```text
//! Input:  add x10, x5, [x6, #8]
//! Output: ldr x16, [x6, #8]   // Load memory operand
//!         add x10, x5, x16    // Perform addition
//! ```
//!
//! ## Literal Operands in Computational Instructions
//!
//! ```text
//! Input:  add x10, x5, #42    // Literal operand in add
//! Output: mov x16, #42        // Load literal
//!         add x10, x5, x16    // Perform addition
//! ```
//!
//! # Usage
//!
//! Wrap any WriterCore implementation with DesugaringWriter to automatically handle
//! all invalid operands:
//!
//! ```ignore
//! use portal_solutions_asm_aarch64::{
//!     desugar::DesugaringWriter,
//!     out::asm::AsmWriter,
//!     AArch64Arch,
//! };
//! use portal_pc_asm_common::types::reg::Reg;
//!
//! let mut output = String::new();
//! let mut writer = AsmWriter::new(&mut output);
//! let mut desugar = DesugaringWriter::new(&mut writer);
//!
//! let cfg = AArch64Arch::default();
//! let dest = Reg(10); // x10
//!
//! // Large immediate that exceeds ADD limits
//! desugar.add(cfg, &dest, &Reg(5), &5000u64)?; // Desugars to mov + add
//!
//! // Memory operand in arithmetic
//! let mem = MemArgKind::Mem {
//!     base: ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 },
//!     offset: None,
//!     disp: 8,
//!     size: MemorySize::_64,
//!     reg_class: RegisterClass::Gpr,
//!     mode: AddressingMode::Offset,
//! };
//!
//! desugar.add(cfg, &dest, &Reg(5), &mem)?; // Loads mem into temp, then adds
//! ```

use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

use crate::{
    out::{
        arg::{ArgKind, MemArg, MemArgKind},
        WriterCore,
    },
    AArch64Arch, RegisterClass,
};

/// Configuration for the desugaring wrapper.
#[derive(Clone, Copy, Debug)]
pub struct DesugarConfig {
    /// Primary temporary register to use for address calculations and operand loading.
    /// Default: x16 (IP0) - intra-procedure-call scratch register.
    pub temp_reg: Reg,
    /// Secondary temporary register for when primary is in use.
    /// Default: x17 (IP1) - another intra-procedure-call scratch register.
    pub temp_reg2: Reg,
}

impl Default for DesugarConfig {
    fn default() -> Self {
        Self {
            temp_reg: Reg(16),  // x16 (IP0)
            temp_reg2: Reg(17), // x17 (IP1)
        }
    }
}

/// Wrapper around WriterCore that desugars complex operands.
///
/// This wrapper intercepts instructions and desugars operands that violate
/// AArch64 constraints into valid instruction sequences.
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

    /// Checks if an immediate fits in 12 bits (AArch64 ADD/SUB immediate range).
    fn fits_in_12_bits(value: i32) -> bool {
        value >= 0 && value < 4096
    }

    /// Checks if an immediate fits in 16 bits (AArch64 MOVK immediate range).
    fn fits_in_16_bits(value: u64) -> bool {
        value < (1 << 16)
    }

    /// Desugars a memory operand if needed.
    ///
    /// Currently, AArch64 supports most memory addressing modes directly,
    /// but this may need extension for future complex cases.
    fn desugar_mem_operand(
        &mut self,
        _arch: AArch64Arch,
        mem: &MemArgKind<ArgKind>,
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        // For now, AArch64 memory operands are mostly supported directly
        // This could be extended for complex cases in the future
        Ok(mem.clone())
    }

    /// Helper to create a simple memory operand from base and displacement.
    fn simple_mem(base: Reg, disp: i32, size: MemorySize, reg_class: RegisterClass) -> MemArgKind<ArgKind> {
        MemArgKind::Mem {
            base: ArgKind::Reg { reg: base, size },
            offset: None,
            disp,
            size,
            reg_class,
            mode: crate::out::arg::AddressingMode::Offset,
        }
    }

    /// Desugars a memory argument if needed.
    fn desugar_mem_arg(
        &mut self,
        arch: AArch64Arch,
        mem_arg: &(dyn MemArg + '_),
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = mem_arg.concrete_mem_kind();

        match &concrete {
            MemArgKind::NoMem(_) => Ok(concrete),
            MemArgKind::Mem { .. } => {
                // For now, pass through - AArch64 handles most memory modes directly
                self.desugar_mem_operand(arch, &concrete)
            }
        }
    }

    /// Desugars an operand that might be a memory reference or literal.
    /// Returns a MemArgKind that is guaranteed to be a register (not memory or literal).
    fn desugar_operand(
        &mut self,
        arch: AArch64Arch,
        operand: &(dyn MemArg + '_),
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = operand.concrete_mem_kind();

        match &concrete {
            MemArgKind::NoMem(ArgKind::Reg { .. }) => Ok(concrete), // Already a register
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // This is a literal operand - need to load it into a temp register
                let temp_reg = self.config.temp_reg;
                self.writer.mov_imm(arch, &temp_reg, *val)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: MemorySize::_64, // Literals are loaded as 64-bit values
                }))
            }
            MemArgKind::Mem { size, reg_class, .. } => {
                // This is a memory operand - need to load it into a temp register
                let temp_reg = self.config.temp_reg;
                let desugared_mem = self.desugar_mem_arg(arch, operand)?;

                // Load the memory operand into the temp register
                // Use the appropriate load instruction based on size
                match size {
                    MemorySize::_8 => self.writer.ldr(arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_16 => self.writer.ldr(arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_32 => self.writer.ldr(arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_64 => self.writer.ldr(arch, &temp_reg, &desugared_mem)?,
                }

                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: *size,
                }))
            }
        }
    }

    /// Helper for binary operations that may have memory or literal operands.
    /// Ensures that operands are loaded into registers as needed.
    fn binary_op<F>(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        op: F,
    ) -> Result<(), W::Error>
    where
        F: FnOnce(&mut W, AArch64Arch, &(dyn MemArg + '_), &(dyn MemArg + '_), &(dyn MemArg + '_)) -> Result<(), W::Error>,
    {
        let a_concrete = a.concrete_mem_kind();
        let b_concrete = b.concrete_mem_kind();

        // Check if operands need desugaring
        let a_needs_desugar = !matches!(a_concrete, MemArgKind::NoMem(ArgKind::Reg { .. }));
        let b_needs_desugar = !matches!(b_concrete, MemArgKind::NoMem(ArgKind::Reg { .. }));

        match (a_needs_desugar, b_needs_desugar) {
            (false, false) => {
                // Neither needs desugaring - use directly
                op(self.writer, cfg, dest, a, b)
            }
            (true, false) => {
                // Only a needs desugaring
                let desugared_a = self.desugar_operand(cfg, a)?;
                op(self.writer, cfg, dest, &desugared_a, b)
            }
            (false, true) => {
                // Only b needs desugaring
                let desugared_b = self.desugar_operand(cfg, b)?;
                op(self.writer, cfg, dest, a, &desugared_b)
            }
            (true, true) => {
                // Both need desugaring - handle memory operands specially to avoid conflicts
                let a_is_mem = matches!(a_concrete, MemArgKind::Mem { .. });
                let b_is_mem = matches!(b_concrete, MemArgKind::Mem { .. });

                if a_is_mem && b_is_mem {
                    // Both are memory - use different temp registers
                    let temp_reg_a = self.config.temp_reg;
                    let temp_reg_b = self.config.temp_reg2;

                    // Load a
                    let desugared_mem_a = self.desugar_mem_arg(cfg, a)?;
                    let a_size = if let MemArgKind::Mem { size, .. } = &a_concrete { *size } else { MemorySize::_64 };
                    match a_size {
                        MemorySize::_8 => self.writer.ldr(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_16 => self.writer.ldr(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_32 => self.writer.ldr(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_64 => self.writer.ldr(cfg, &temp_reg_a, &desugared_mem_a)?,
                    }

                    // Load b
                    let desugared_mem_b = self.desugar_mem_arg(cfg, b)?;
                    let b_size = if let MemArgKind::Mem { size, .. } = &b_concrete { *size } else { MemorySize::_64 };
                    match b_size {
                        MemorySize::_8 => self.writer.ldr(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_16 => self.writer.ldr(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_32 => self.writer.ldr(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_64 => self.writer.ldr(cfg, &temp_reg_b, &desugared_mem_b)?,
                    }

                    let desugared_a = MemArgKind::NoMem(ArgKind::Reg { reg: temp_reg_a, size: a_size });
                    let desugared_b = MemArgKind::NoMem(ArgKind::Reg { reg: temp_reg_b, size: b_size });

                    op(self.writer, cfg, dest, &desugared_a, &desugared_b)
                } else {
                    // At least one is literal, not memory - can use regular desugar_operand
                    let desugared_a = self.desugar_operand(cfg, a)?;
                    let desugared_b = self.desugar_operand(cfg, b)?;
                    op(self.writer, cfg, dest, &desugared_a, &desugared_b)
                }
            }
        }
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::String;
    use core::fmt::Write;

    #[test]
    fn test_desugar_literal_operand_in_add() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = AArch64Arch::default();
            let dest = Reg(10); // x10
            let a = Reg(5);     // x5

            // Literal operand in add instruction
            let b_literal = MemArgKind::NoMem(ArgKind::Lit(42));

            // This should desugar to mov + add
            let _ = desugar.add(cfg, &dest, &a, &b_literal);
        }

        // Check that output contains mov and add
        assert!(output.contains("mov"));
        assert!(output.contains("add"));
    }

    #[test]
    fn test_desugar_literal_operand_in_mov() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = AArch64Arch::default();
            let dest = Reg(10); // x10

            // Literal operand in mov instruction
            let src_literal = MemArgKind::NoMem(ArgKind::Lit(123));

            // This should desugar to mov_imm (not mov)
            let _ = desugar.mov(cfg, &dest, &src_literal);
        }

        // Check that output contains mov but not regular mov instruction
        // (it should use mov_imm which might output differently)
        assert!(output.contains("mov"));
    }

    #[test]
    fn test_desugar_memory_operand_in_add() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = AArch64Arch::default();
            let dest = Reg(10); // x10
            let a = Reg(5);     // x5

            // Memory operand in add instruction
            let b_mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::Offset,
            };

            // This should desugar to ldr + add
            let _ = desugar.add(cfg, &dest, &a, &b_mem);
        }

        // Check that output contains ldr and add
        assert!(output.contains("ldr"));
        assert!(output.contains("add"));
    }
}

impl<'a, W: WriterCore + ?Sized> WriterCore for DesugaringWriter<'a, W> {
    type Error = W::Error;

    // Memory load/store instructions that need desugaring

    fn ldr(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.ldr(cfg, dest, &desugared_mem)
    }

    fn str(
        &mut self,
        cfg: AArch64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.str(cfg, src, &desugared_mem)
    }

    fn stp(
        &mut self,
        cfg: AArch64Arch,
        src1: &(dyn MemArg + '_),
        src2: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.stp(cfg, src1, src2, &desugared_mem)
    }

    fn ldp(
        &mut self,
        cfg: AArch64Arch,
        dest1: &(dyn MemArg + '_),
        dest2: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(cfg, mem)?;
        self.writer.ldp(cfg, dest1, dest2, &desugared_mem)
    }

    // Forward all non-memory instructions directly to the underlying writer
    // (We only need to implement the trait - the default implementations will forward via todo!())

    fn brk(&mut self, cfg: AArch64Arch, imm: u16) -> Result<(), Self::Error> {
        self.writer.brk(cfg, imm)
    }

    fn mov(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let src_concrete = src.concrete_mem_kind();
        match &src_concrete {
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // Source is a literal - use mov_imm instead of mov
                self.writer.mov_imm(cfg, dest, *val)
            }
            _ => {
                // Source is register or memory - desugar and use mov
                let desugared_src = self.desugar_operand(cfg, src)?;
                self.writer.mov(cfg, dest, &desugared_src)
            }
        }
    }

    fn add(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.add(cfg, dest, a, b)
        })
    }

    fn sub(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.sub(cfg, dest, a, b)
        })
    }

    fn mov_imm(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        self.writer.mov_imm(cfg, dest, val)
    }

    fn mul(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.mul(cfg, dest, a, b)
        })
    }

    fn udiv(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.udiv(cfg, dest, a, b)
        })
    }

    fn sdiv(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.sdiv(cfg, dest, a, b)
        })
    }

    fn and(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.and(cfg, dest, a, b)
        })
    }

    fn orr(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.orr(cfg, dest, a, b)
        })
    }

    fn eor(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.eor(cfg, dest, a, b)
        })
    }

    fn lsl(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.lsl(cfg, dest, a, b)
        })
    }

    fn lsr(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.lsr(cfg, dest, a, b)
        })
    }

    fn sxt(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.sxt(cfg, dest, &desugared_src)
    }

    fn uxt(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.uxt(cfg, dest, &desugared_src)
    }

    fn mvn(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.mvn(cfg, dest, &desugared_src)
    }

    fn bl(&mut self, cfg: AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.bl(cfg, &desugared_target)
    }

    fn br(&mut self, cfg: AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.br(cfg, &desugared_target)
    }

    fn b(&mut self, cfg: AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.b(cfg, &desugared_target)
    }

    fn cmp(
        &mut self,
        cfg: AArch64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_a = self.desugar_operand(cfg, a)?;
        let desugared_b = self.desugar_operand(cfg, b)?;
        self.writer.cmp(cfg, &desugared_a, &desugared_b)
    }

    fn csel(
        &mut self,
        cfg: AArch64Arch,
        cond: crate::ConditionCode,
        dest: &(dyn MemArg + '_),
        true_val: &(dyn MemArg + '_),
        false_val: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_true = self.desugar_operand(cfg, true_val)?;
        let desugared_false = self.desugar_operand(cfg, false_val)?;
        self.writer.csel(cfg, cond, dest, &desugared_true, &desugared_false)
    }

    fn bcond(
        &mut self,
        cfg: AArch64Arch,
        cond: crate::ConditionCode,
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.bcond(cfg, cond, &desugared_target)
    }

    fn adr(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.adr(cfg, dest, &desugared_src)
    }

    fn ret(&mut self, cfg: AArch64Arch) -> Result<(), Self::Error> {
        self.writer.ret(cfg)
    }

    fn mrs_nzcv(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.mrs_nzcv(cfg, dest)
    }

    fn msr_nzcv(
        &mut self,
        cfg: AArch64Arch,
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.msr_nzcv(cfg, &desugared_src)
    }

    // Floating-point operations

    fn fadd(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fadd(cfg, dest, a, b)
        })
    }

    fn fsub(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fsub(cfg, dest, a, b)
        })
    }

    fn fmul(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fmul(cfg, dest, a, b)
        })
    }

    fn fdiv(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fdiv(cfg, dest, a, b)
        })
    }

    fn fmov(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.fmov(cfg, dest, &desugared_src)
    }


}

// Implement Writer trait for DesugaringWriter
// This enables label support - we simply forward to the underlying writer
impl<'a, W, L> crate::out::Writer<L> for DesugaringWriter<'a, W>
where
    W: crate::out::Writer<L> + ?Sized,
{
    fn set_label(&mut self, cfg: AArch64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.set_label(cfg, label)
    }

    fn adr_label(
        &mut self,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.adr_label(cfg, dest, label)
    }

    fn b_label(
        &mut self,
        cfg: AArch64Arch,
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.b_label(cfg, label)
    }

    fn bl_label(
        &mut self,
        cfg: AArch64Arch,
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.bl_label(cfg, label)
    }
}