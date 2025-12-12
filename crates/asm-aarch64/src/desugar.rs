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
//! desugar.add(ctx, cfg, &dest, &Reg(5), &5000u64)?; // Desugars to mov + add
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
//! desugarctx, .add(cfg, &dest, &Reg(5), &mem)?; // Loads mem into temp, then adds
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
    /// Tertiary temporary register for complex operations.
    /// Default: x15 - another scratch register.
    pub temp_reg3: Reg,
    /// If true, when temporary candidates conflict with operands, save/restore
    /// the chosen temporary on the stack (via pre/post-indexed str/ldr) to
    /// preserve its original value. Default: false.
    pub save_temps_on_stack: bool,
}

impl Default for DesugarConfig {
    fn default() -> Self {
        Self {
            temp_reg: Reg(16),  // x16 (IP0)
            temp_reg2: Reg(17), // x17 (IP1)
            temp_reg3: Reg(15), // x15
            save_temps_on_stack: false,
        }
    }
}

/// Manages temporary register allocation with push/pop caching.
///
/// This struct tracks the complete stack layout of pushed registers to ensure
/// correct stack discipline and enable more aggressive stack manipulation.
/// Registers are stored in push order, allowing proper LIFO popping and
/// preventing stack corruption.
pub struct TempRegManager {
    /// Stack of pushed registers, in push order (index 0 is bottom of stack)
    pushed_stack: [Reg; 16],
    /// Current number of pushed registers
    stack_depth: usize,
}

impl TempRegManager {
    pub fn new() -> Self {
        Self {
            pushed_stack: [Reg(0); 16], // Initialize with dummy values
            stack_depth: 0,
        }
    }

    /// Acquire a temporary register, pushing it to the stack if needed and not already pushed.
    pub fn acquire_temp<Context, W: WriterCore<Context> + ?Sized>(&mut self, writer: &mut W, ctx: &mut Context,
        config: &DesugarConfig,
        reg_class: RegisterClass,
        used_regs: &[Reg],
        used_count: usize
    ) -> Result<Reg, W::Error> {
        let candidates = match reg_class {
            RegisterClass::Gpr => [config.temp_reg, config.temp_reg2, config.temp_reg3],
            RegisterClass::Simd => [Reg(16), Reg(17), Reg(0)], // v16, v17 as SIMD temps, pad to 3 elements
        };

        // Find first candidate that doesn't conflict
        for &candidate in &candidates {
            if candidate.0 == 0 { continue; } // Skip padding
            let mut conflicts = false;
            for i in 0..used_count {
                if used_regs[i] == candidate {
                    conflicts = true;
                    break;
                }
            }
            if !conflicts {
                return Ok(candidate);
            }
        }

        // If all candidates conflict, check if we can use push/pop
        if !Self::sp_used(used_regs, used_count) {
            // Safe to use push/pop - pick the first candidate
            let temp_reg = candidates[0];

            // Check if already pushed (search the stack)
            let mut already_pushed = false;
            for i in 0..self.stack_depth {
                if self.pushed_stack[i] == temp_reg {
                    already_pushed = true;
                    break;
                }
            }

            if !already_pushed {
                // Not pushed yet, push it
                let sp = Reg(31); // SP register
                let mem = crate::out::arg::MemArgKind::Mem {
                    base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                    offset: None,
                    disp: -16, // Push 16 bytes
                    size: MemorySize::_64,
                    reg_class: RegisterClass::Gpr,
                    mode: crate::out::arg::AddressingMode::PreIndex,
                };
                let src_reg = crate::out::arg::MemArgKind::NoMem(crate::out::arg::ArgKind::Reg { reg: temp_reg, size: MemorySize::_64 });
                writer.str(ctx, AArch64Arch::default(), &src_reg, &mem)?;
                self.pushed_stack[self.stack_depth] = temp_reg;
                self.stack_depth += 1;
            }
            // Already pushed, can use it

            Ok(temp_reg)
        } else {
            // Cannot use push/pop, use the first candidate anyway (may cause incorrect code)
            Ok(candidates[0])
        }
    }

    /// Release a temporary register, popping it from the stack if it's at the top.
    /// If the register is buried deeper in the stack, it remains pushed for potential future use.
    pub fn release_temp<Context, W: WriterCore<Context> + ?Sized>(&mut self, writer: &mut W, ctx: &mut Context, reg: Reg) -> Result<(), W::Error> {
        // Only pop if this register is at the top of the stack
        if self.stack_depth > 0 && self.pushed_stack[self.stack_depth - 1] == reg {
            let sp = Reg(31); // SP register
            let mem = crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: 16, // Pop 16 bytes
                size: MemorySize::_64,
                reg_class: RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PostIndex,
            };
            let dest_reg = crate::out::arg::MemArgKind::NoMem(crate::out::arg::ArgKind::Reg { reg, size: MemorySize::_64 });
            writer.ldr(ctx, AArch64Arch::default(), &dest_reg, &mem)?;
            self.stack_depth -= 1;
        }
        // If not at the top, leave it pushed (might be used again)
        Ok(())
    }

    /// Release all pushed registers in reverse order (LIFO).
    /// This should be called at the end of a desugaring operation to clean up the stack.
    pub fn release_all<Context, W: WriterCore<Context> + ?Sized>(&mut self, writer: &mut W, ctx: &mut Context) -> Result<(), W::Error> {
        while self.stack_depth > 0 {
            let reg = self.pushed_stack[self.stack_depth - 1];
            let sp = Reg(31); // SP register
            let mem = crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: 16, // Pop 16 bytes
                size: MemorySize::_64,
                reg_class: RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PostIndex,
            };
            let dest_reg = crate::out::arg::MemArgKind::NoMem(crate::out::arg::ArgKind::Reg { reg, size: MemorySize::_64 });
            writer.ldr(ctx, AArch64Arch::default(), &dest_reg, &mem)?;
            self.stack_depth -= 1;
        }
        Ok(())
    }

    /// Check if SP (stack pointer) is used in the given registers.
    fn sp_used(used_regs: &[Reg], used_count: usize) -> bool {
        let sp = Reg(31); // SP is register 31 in AArch64
        for i in 0..used_count {
            if used_regs[i] == sp {
                return true;
            }
        }
        false
    }
}

/// Wrapper around WriterCore that desugars complex operands.
///
/// This wrapper intercepts instructions and desugars operands that violate
/// AArch64 constraints into valid instruction sequences.
pub struct DesugaringWriter<'a, W: WriterCore<Context> + ?Sized, Context> {
    /// The underlying writer.
    writer: &'a mut W,
    /// Configuration for desugaring.
    config: DesugarConfig,
    /// Manager for temporary register allocation with push/pop caching.
    temp_manager: TempRegManager,
    /// Marker to keep the Context generic parameter.
    _marker: core::marker::PhantomData<Context>,
}

impl<'a, W: WriterCore<Context> + ?Sized, Context> DesugaringWriter<'a, W, Context> {
    /// Creates a new desugaring wrapper with default configuration.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            config: DesugarConfig::default(),
            temp_manager: TempRegManager::new(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Creates a new desugaring wrapper with custom configuration.
    pub fn with_config(writer: &'a mut W, config: DesugarConfig) -> Self {
        Self {
            writer,
            config,
            temp_manager: TempRegManager::new(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Release all pushed temporary registers, restoring the stack to its original state.
    /// This should be called when desugaring operations are complete to ensure proper stack cleanup.
    pub fn release_all_temps(&mut self) -> Result<(), W::Error> {
        self.temp_manager.release_all(self.writer, ctx)
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
        &mut self, ctx: &mut Context,
        arch: AArch64Arch,
        mem_arg: &(dyn MemArg + '_),
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = mem_arg.concrete_mem_kind();

        match &concrete {
            MemArgKind::NoMem(_) => Ok(concrete),
            MemArgKind::Mem { base, offset, disp, size, reg_class, mode } => {
                // If this memory uses SP and we have pushed temps on the stack,
                // we must either adjust the displacement for the pushed temps
                // (for Offset addressing) or restore the pushed temps before
                // emitting an instruction that updates SP (PreIndex/PostIndex).
                if let ArgKind::Reg { reg, .. } = base {
                    let sp = Reg(31);
                    if *reg == sp && self.temp_manager.stack_depth > 0 {
                        match mode {
                            crate::out::arg::AddressingMode::Offset => {
                                // Adjust the displacement to account for pushed temps.
                                // Each pushed register used 16 bytes.
                                let adjust = (self.temp_manager.stack_depth as i32) * 16;
                                let new_disp = disp.wrapping_add(adjust);
                                return Ok(MemArgKind::Mem {
                                    base: base.clone(),
                                    offset: offset.clone(),
                                    disp: new_disp,
                                    size: *size,
                                    reg_class: *reg_class,
                                    mode: *mode,
                                });
                            }
                            _ => {
                                // For addressing modes that update SP (PreIndex/PostIndex),
                                // we must restore any pushed temps before proceeding so that
                                // the SP-relative addressing matches the original intent.
                                self.temp_manager.release_all(self.writer, ctx)?;
                                // After restoring, pass through the original memory (no adjustment)
                                return self.desugar_mem_operand(arch, &concrete);
                            }
                        }
                    }
                }

                // Default: pass through (no SP adjustments needed)
                self.desugar_mem_operand(arch, &concrete)
            }
        }
    }

    /// Loads a memory operand into a register if needed, returning a register operand.
    fn load_operand_to_reg(
        &mut self, ctx: &mut Context,
        arch: AArch64Arch,
        operand: &(dyn MemArg + '_),
        reg_class: RegisterClass,
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = operand.concrete_mem_kind();

        match &concrete {
            MemArgKind::NoMem(ArgKind::Reg { .. }) => Ok(concrete), // Already a register
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // This is a literal operand - need to load it into a temp register
                let temp_reg = self.config.temp_reg; // Use primary temp for literals
                self.writer.mov_imm(ctx,  arch, &temp_reg, *val)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: MemorySize::_64, // Literals are loaded as 64-bit values
                }))
            }
            MemArgKind::Mem { size, .. } => {
                // This is a memory operand - load it into a temp register
                let mut used = [Reg(0); 2];
                let count = Self::collect_used_regs(&concrete, &mut used);
                let temp_reg = self.temp_manager.acquire_temp(self.writer, ctx,  &self.config, reg_class, &used, count)?;

                let desugared_mem = self.desugar_mem_arg(ctx, arch, operand)?;
                match size {
                    MemorySize::_8 => self.writer.ldr(ctx,  arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_16 => self.writer.ldr(ctx,  arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_32 => self.writer.ldr(ctx,  arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_64 => self.writer.ldr(ctx,  arch, &temp_reg, &desugared_mem)?,
                }

                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: *size,
                }))
            }
        }
    }

    /// Desugars an operand that might be a memory reference or literal.
    /// For literals, loads them into a register. For memory, returns the desugared memory form.
    /// The caller is responsible for loading memory operands into registers if needed.
    fn desugar_operand(
        &mut self, ctx: &mut Context,
        arch: AArch64Arch,
        operand: &(dyn MemArg + '_),
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = operand.concrete_mem_kind();

        // If this operand is the SP register itself, restore pushed temps
        // before reading/writing SP directly.
        if let MemArgKind::NoMem(ArgKind::Reg { reg, .. }) = concrete {
            let sp = Reg(31);
            if reg == sp && self.temp_manager.stack_depth > 0 {
                self.temp_manager.release_all(self.writer, ctx)?;
            }
        }

        match &concrete {
            MemArgKind::NoMem(ArgKind::Reg { .. }) => Ok(concrete), // Already a register
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // This is a literal operand - need to load it into a temp register
                let temp_reg = self.config.temp_reg; // Use primary temp for literals
                self.writer.mov_imm(ctx,  arch, &temp_reg, *val)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: MemorySize::_64, // Literals are loaded as 64-bit values
                }))
            }
            MemArgKind::Mem { .. } => {
                // This is a memory operand - return desugared memory form
                // Caller will handle loading into register if needed
                self.desugar_mem_arg(ctx, arch, operand)
            }
        }
    }

    /// Restore pushed temps if the given operand reads/writes SP directly.
    fn restore_temps_if_sp_used(&mut self, ctx: &mut Context, operand: &(dyn MemArg + '_)) -> Result<(), W::Error> {
        let concrete = operand.concrete_mem_kind();
        match &concrete {
            MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => {
                if *reg == Reg(31) && self.temp_manager.stack_depth > 0 {
                    self.temp_manager.release_all(self.writer, ctx)?;
                }
            }
            MemArgKind::Mem { base, offset, .. } => {
                if let ArgKind::Reg { reg, .. } = base {
                    if *reg == Reg(31) && self.temp_manager.stack_depth > 0 {
                        self.temp_manager.release_all(self.writer, ctx)?;
                        return Ok(());
                    }
                }
                if let Some((ArgKind::Reg { reg, .. }, _)) = offset {
                    if *reg == Reg(31) && self.temp_manager.stack_depth > 0 {
                        self.temp_manager.release_all(self.writer, ctx)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper for binary operations that may have memory or literal operands.
    /// Ensures that operands are loaded into registers as needed.
    fn binary_op< F >( &mut self, ctx: &mut Context, cfg: AArch64Arch,
         dest: &(dyn MemArg + '_),
         a: &(dyn MemArg + '_),
         b: &(dyn MemArg + '_),
         op: F,
     ) -> Result<(), W::Error>
     where
         F: FnOnce(&mut W, &mut Context, AArch64Arch, &(dyn MemArg + '_), &(dyn MemArg + '_), &(dyn MemArg + '_)) -> Result<(), W::Error>,
     {
         let a_concrete = a.concrete_mem_kind();
         let b_concrete = b.concrete_mem_kind();
 
         // Check if operands need desugaring
         let a_needs_desugar = !matches!(a_concrete, MemArgKind::NoMem(ArgKind::Reg { .. }));
         let b_needs_desugar = !matches!(b_concrete, MemArgKind::NoMem(ArgKind::Reg { .. }));
 
        match (a_needs_desugar, b_needs_desugar) {
            (false, false) => {
                // Neither needs desugaring - use directly
                op(self.writer, ctx, cfg, dest, a, b)
            }
            (true, false) => {
                // Only a needs desugaring
                let desugared_a = self.desugar_operand(ctx, cfg, a)?;
                op(self.writer, ctx, cfg, dest, &desugared_a, b)
            }
            (false, true) => {
                // Only b needs desugaring
                let desugared_b = self.desugar_operand(ctx, cfg, b)?;
                op(self.writer, ctx, cfg, dest, a, &desugared_b)
            }
            (true, true) => {
                // Both need desugaring - handle memory operands specially
                let a_is_mem = matches!(a_concrete, MemArgKind::Mem { .. });
                let b_is_mem = matches!(b_concrete, MemArgKind::Mem { .. });

                if a_is_mem && b_is_mem {
                    // Both are memory - load one into temp
                    let mut all_used = [Reg(0); 6];
                    let a_count = Self::collect_used_regs(&a_concrete, &mut all_used[0..3]);
                    let b_count = Self::collect_used_regs(&b_concrete, &mut all_used[3..6]);
                    let total_count = a_count + b_count;
                    let temp_b = self.temp_manager.acquire_temp(self.writer, ctx,  &self.config, RegisterClass::Gpr, &all_used, total_count)?;

                    let desugared_mem_b = self.desugar_mem_arg(ctx, cfg, b)?;
                    let b_size = if let MemArgKind::Mem { size, .. } = &b_concrete { *size } else { MemorySize::_64 };
                    match b_size {
                        MemorySize::_8 => self.writer.ldr(ctx,  cfg, &temp_b, &desugared_mem_b)?,
                        MemorySize::_16 => self.writer.ldr(ctx,  cfg, &temp_b, &desugared_mem_b)?,
                        MemorySize::_32 => self.writer.ldr(ctx,  cfg, &temp_b, &desugared_mem_b)?,
                        MemorySize::_64 => self.writer.ldr(ctx,  cfg, &temp_b, &desugared_mem_b)?,
                    }

                    let desugared_a = self.desugar_mem_arg(cfg, a)?;
                    let desugared_b = MemArgKind::NoMem(ArgKind::Reg { reg: temp_b, size: b_size });

                    op(self.writer, ctx, cfg, dest, &desugared_a, &desugared_b)?;

                    // Release the temp register
                    self.temp_manager.release_temp(self.writer, ctx, temp_b)?;
                    Ok(())
                } else {
                    // At least one is literal - use desugar_operand
                    let desugared_a = self.desugar_operand(ctx, cfg, a)?;
                    let desugared_b = self.desugar_operand(ctx, cfg, b)?;
                    op(self.writer, ctx, cfg, dest, &desugared_a, &desugared_b)?;
                    Ok(())
                }
            }
        }
      }

 
     /// Collect registers used in a MemArgKind into the provided buffer.
    /// Returns the number of registers written (0..=buffer.len()).
    fn collect_used_regs(mem: &MemArgKind<ArgKind>, buffer: &mut [Reg]) -> usize {
        let mut count = 0usize;
        match mem {
            MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => {
                if buffer.len() > 0 {
                    buffer[0] = *reg;
                    count = 1;
                }
            }
            MemArgKind::NoMem(ArgKind::Lit(_)) => {
                // no registers
            }
            MemArgKind::Mem { base, offset, .. } => {
                if let ArgKind::Reg { reg, .. } = base {
                    if count < buffer.len() { buffer[count] = *reg; count += 1; }
                }
                if let Some((ArgKind::Reg { reg, .. }, _)) = offset {
                    if count < buffer.len() { buffer[count] = *reg; count += 1; }
                }
            }
        }
        count
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
            let _ = desugar.add(ctx, &dest, &a, &b_literal);
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
            let _ = desugar.mov(ctx, cfg, &dest, &src_literal);
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
            let _ = desugar.add(ctx, &dest, &a, &b_mem);
        }

        // Check that output contains ldr and add
        assert!(output.contains("ldr"));
        assert!(output.contains("add"));
    }
}

impl<'a, W: WriterCore<Context> + ?Sized, Context> WriterCore<Context> for DesugaringWriter<'a, W, Context> {
    type Error = W::Error;

    // Memory load/store instructions that need desugaring

    fn ldr(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(ctx, cfg, mem)?;
        self.writer.ldr(ctx,  cfg, dest, &desugared_mem)
    }

    fn str(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        src: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(ctx, cfg, mem)?;
        self.writer.str(ctx,  cfg, src, &desugared_mem)
    }

    fn stp(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        src1: &(dyn MemArg + '_),
        src2: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(ctx, cfg, mem)?;
        self.writer.stp(ctx,  cfg, src1, src2, &desugared_mem)
    }

    fn ldp(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest1: &(dyn MemArg + '_),
        dest2: &(dyn MemArg + '_),
        mem: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_mem = self.desugar_mem_arg(ctx, cfg, mem)?;
        self.writer.ldp(ctx,  cfg, dest1, dest2, &desugared_mem)
    }

    // Forward all non-memory instructions directly to the underlying writer
    // (We only need to implement the trait - the default implementations will forward via todo!())

    fn brk(&mut self, cfg: AArch64Arch, imm: u16) -> Result<(), Self::Error> {
        self.writer.brk(ctx, cfg, imm)
    }

    fn mov(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let dest_concrete = dest.concrete_mem_kind();
        let src_concrete = src.concrete_mem_kind();

        let dest_is_mem = matches!(dest_concrete, MemArgKind::Mem { .. });
        let src_is_mem = matches!(src_concrete, MemArgKind::Mem { .. });

        match (dest_is_mem, src_is_mem) {
            (false, false) => {
                // Both not memory
                match &src_concrete {
                    MemArgKind::NoMem(ArgKind::Lit(val)) => {
                        // Source is a literal - use mov_imm
                        self.writer.mov_imm(ctx,  cfg, dest, *val)
                    }
                    _ => {
                        // Both registers
                        self.writer.mov(ctx, cfg, dest, src)
                    }
                }
            }
            (false, true) => {
                // src is memory, dest is register - load from memory
                let desugared_src = self.desugar_mem_arg(cfg, src)?;
                self.writer.ldr(ctx,  cfg, dest, &desugared_src)
            }
            (true, false) => {
                // dest is memory, src is register/literal - store to memory
                let desugared_dest = self.desugar_mem_arg(cfg, dest)?;
                match &src_concrete {
                    MemArgKind::NoMem(ArgKind::Lit(val)) => {
                        // Source is literal - load to temp then store
                        let temp_reg = self.config.temp_reg;
                        self.writer.mov_imm(ctx,  cfg, &temp_reg, *val)?;
                        self.writer.str(ctx,  cfg, &temp_reg, &desugared_dest)
                    }
                    _ => {
                        // Source is register
                        self.writer.str(ctx,  cfg, src, &desugared_dest)
                    }
                }
            }
            (true, true) => {
                // Both memory - load src to temp then store to dest
                let mut used = [Reg(0); 4];
                let src_count = Self::collect_used_regs(&src_concrete, &mut used[0..2]);
                let dest_count = Self::collect_used_regs(&dest_concrete, &mut used[2..4]);
                let total_count = src_count + dest_count;
                let temp = self.temp_manager.acquire_temp(self.writer, ctx,  &self.config, RegisterClass::Gpr, &used, total_count)?;

                let desugared_src = self.desugar_mem_arg(cfg, src)?;
                let src_size = if let MemArgKind::Mem { size, .. } = &src_concrete { *size } else { MemorySize::_64 };
                match src_size {
                    MemorySize::_8 => self.writer.ldr(ctx,  cfg, &temp, &desugared_src)?,
                    MemorySize::_16 => self.writer.ldr(ctx,  cfg, &temp, &desugared_src)?,
                    MemorySize::_32 => self.writer.ldr(ctx,  cfg, &temp, &desugared_src)?,
                    MemorySize::_64 => self.writer.ldr(ctx,  cfg, &temp, &desugared_src)?,
                }

                let desugared_dest = self.desugar_mem_arg(cfg, dest)?;
                self.writer.str(ctx,  cfg, &temp, &desugared_dest)?;

                // Release temp
                self.temp_manager.release_temp(self.writer, ctx, temp)
            }
        }
    }

    fn add(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
writer.add(ctx, cfg, dest, a, b)
        })
    }

    fn sub(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.sub(ctx, cfg, dest, a, b)
        })
    }

    fn mov_imm(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        self.writer.mov_imm(ctx,  cfg, dest, val)
    }

    fn mul(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.mul(ctx, cfg, dest, a, b)
        })
    }

    fn udiv(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.udiv(ctx, cfg, dest, a, b)
        })
    }

    fn sdiv(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.sdiv(ctx, cfg, dest, a, b)
        })
    }

    fn and(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.and(ctx, cfg, dest, a, b)
        })
    }

    fn orr(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.orr(ctx, cfg, dest, a, b)
        })
    }

    fn eor(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.eor(ctx, cfg, dest, a, b)
        })
    }

    fn lsl(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.lsl(ctx, cfg, dest, a, b)
        })
    }

    fn lsr(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.lsr(ctx, cfg, dest, a, b)
        })
    }

    fn sxt(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.load_operand_to_reg(cfg, src, RegisterClass::Gpr)?;
        self.writer.sxt(ctx, cfg, dest, &desugared_src)
    }

    fn uxt(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.load_operand_to_reg(cfg, src, RegisterClass::Gpr)?;
        self.writer.uxt(ctx, cfg, dest, &desugared_src)
    }

    fn mvn(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.load_operand_to_reg(cfg, src, RegisterClass::Gpr)?;
        self.writer.mvn(ctx, cfg, dest, &desugared_src)
    }

    fn bl(&mut self, cfg: AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.load_operand_to_reg(cfg, target, RegisterClass::Gpr)?;
        self.writer.bl(ctx,  cfg, &desugared_target)
    }

    fn br(&mut self, cfg: AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.load_operand_to_reg(cfg, target, RegisterClass::Gpr)?;
        self.writer.br(ctx,  cfg, &desugared_target)
    }

    fn b(&mut self, cfg: AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.load_operand_to_reg(cfg, target, RegisterClass::Gpr)?;
        self.writer.b(ctx,  cfg, &desugared_target)
    }

    fn cmp(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_a = self.load_operand_to_reg(cfg, a, RegisterClass::Gpr)?;
        let desugared_b = self.load_operand_to_reg(cfg, b, RegisterClass::Gpr)?;
        self.writer.cmp(ctx,  cfg, &desugared_a, &desugared_b)
    }

    fn csel(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        cond: crate::ConditionCode,
        dest: &(dyn MemArg + '_),
        true_val: &(dyn MemArg + '_),
        false_val: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_true = self.load_operand_to_reg(cfg, true_val, RegisterClass::Gpr)?;
        let desugared_false = self.load_operand_to_reg(cfg, false_val, RegisterClass::Gpr)?;
        self.writer.csel(ctx,  cfg, cond, dest, &desugared_true, &desugared_false)
    }

    fn bcond(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        cond: crate::ConditionCode,
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_target = self.load_operand_to_reg(cfg, target, RegisterClass::Gpr)?;
        self.writer.bcond(ctx,  cfg, cond, &desugared_target)
    }

    fn adr(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.load_operand_to_reg(cfg, src, RegisterClass::Gpr)?;
        self.writer.adr(ctx,  cfg, dest, &desugared_src)
    }

    fn msr_nzcv(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.load_operand_to_reg(cfg, src, RegisterClass::Gpr)?;
        self.writer.msr_nzcv(ctx,  cfg, &desugared_src)
    }



    fn ret(&mut self, cfg: AArch64Arch) -> Result<(), Self::Error> {
        self.writer.ret(ctx,  cfg)
    }

    fn mrs_nzcv(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.writer.mrs_nzcv(ctx,  cfg, dest)
    }

    // Floating-point operations

    fn fadd(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.fadd(ctx, cfg, dest, a, b)
        })
    }

    fn fsub(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.fsub(ctx, cfg, dest, a, b)
        })
    }

    fn fmul(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.fmul(ctx, cfg, dest, a, b)
        })
    }

    fn fdiv(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(ctx, cfg, dest, a, b, |writer, ctx, cfg, dest, a, b| {
            writer.fdiv(ctx, cfg, dest, a, b)
        })
    }

    fn fmov(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.fmov(ctx, cfg, dest, &desugared_src)
    }


}

// Implement Writer trait for DesugaringWriter
// This enables label support - we simply forward to the underlying writer
impl<'a, W, L, Context> crate::out::Writer<L, Context> for DesugaringWriter<'a, W, Context>
where
    W: crate::out::Writer<L, Context> + ?Sized,
{
    fn set_label(&mut self, ctx: &mut Context, cfg: AArch64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.set_label(ctx, cfg, label)
    }

    fn adr_label(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        dest: &(dyn MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.adr_label(ctx, cfg, dest, label)
    }

    fn b_label(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.b_label(ctx, cfg, label)
    }

    fn bl_label(
        &mut self,
        ctx: &mut Context,
        cfg: AArch64Arch,
        label: L,
    ) -> Result<(), Self::Error> {
        self.writer.bl_label(ctx, cfg, label)
    }
}
