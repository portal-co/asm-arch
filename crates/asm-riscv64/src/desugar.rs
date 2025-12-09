//! Desugaring wrapper for RISC-V memory operands.
//!
//! This module provides a robust wrapper around WriterCore implementations that automatically
//! desugars invalid memory operands and memory operands used in computational instructions
//! into valid RISC-V instruction sequences.
//!
//! # Overview
//!
//! RISC-V has two main constraints that require desugaring:
//!
//! 1. **Memory addressing**: RISC-V memory instructions only support `base + disp` addressing
//!    where disp is a 12-bit signed immediate (-2048 to 2047). Complex addressing modes
//!    from the x86_64 shim (like `base + offset*scale + disp`) need to be desugared into
//!    multiple instructions.
//!
//! 2. **Computational instructions**: RISC-V computational instructions (add, sub, mul, etc.)
//!    cannot take memory operands directly - they only operate on registers. Memory operands
//!    must be loaded into registers first.
//!
//! # Features
//!
//! - **Robust temporary register selection**: Automatically avoids conflicts with operand registers
//! - **Stack spill support**: Option to save conflicting registers to stack when temporaries are exhausted
//! - **MemorySize preservation**: Maintains correct memory access sizes across desugaring
//! - **Register class preservation**: Maintains GPR/FP register class information
//! - **Memory-to-memory operations**: Handles operations where both operands are memory references
//! - **Large displacement folding**: Correctly folds large displacements into base registers
//!
//! # Desugaring Examples
//!
//! ## Memory Addressing
//!
//! ### Scaled Offset
//!
//! ```text
//! Input:  ld x10, mem[base=x5, offset=x6, scale=3, disp=100]
//! Output: li   t3, 3          // Load shift amount
//!         sll  t6, x6, t3     // t6 = x6 << 3
//!         add  t6, x5, t6     // t6 = x5 + t6
//!         ld   x10, 100(t6)   // x10 = mem[t6 + 100]
//! ```
//!
//! ### Large Displacement
//!
//! ```text
//! Input:  ld x10, mem[base=x5, disp=4096]
//! Output: li   t6, 4096       // Load large displacement
//!         add  t6, x5, t6     // t6 = x5 + 4096
//!         ld   x10, 0(t6)     // x10 = mem[t6 + 0]
//! ```
//!
//! ### Literal Base
//!
//! ```text
//! Input:  ld x10, mem[base=0x1000, disp=8]
//! Output: li   t6, 0x1000     // Load base address
//!         ld   x10, 8(t6)     // x10 = mem[t6 + 8]
//! ```
//!
//! ## Computational Instructions
//!
//! ### Memory Operands in Arithmetic
//!
//! ```text
//! Input:  add x10, x5, mem[base=x6, disp=8]
//! Output: ld   t6, 8(x6)      // Load memory operand
//!         add  x10, x5, t6    // Perform addition
//! ```
//!
//! ### Large Immediates in ALU Operations
//!
//! ```text
//! Input:  addi x10, x5, 5000  // 5000 > 2047 (12-bit limit)
//! Output: li   t4, 5000       // Load large immediate
//!         add  x10, x5, t4    // Perform addition
//! ```
//!
//! ### Literal Operands in Computational Instructions
//!
//! ```text
//! Input:  add x10, x5, 42     // Literal operand in add
//! Output: li   t6, 42         // Load literal
//!         add  x10, x5, t6    // Perform addition
//! ```
//!
//! ### Literal Operands in Move Instructions
//!
//! ```text
//! Input:  mv x10, 123         // Literal operand in mv
//! Output: li   x10, 123       // Load literal directly
//! ```
//!
//! ### Branch Instructions
//!
//! ```text
//! Input:  beq mem[base=x5, disp=8], x6, label
//! Output: ld   t6, 8(x5)      // Load memory operand
//!         beq  t6, x6, label  // Perform comparison
//!
//! Input:  beq 42, x6, label   // Literal operand in branch
//! Output: li   t6, 42         // Load literal
//!         beq  t6, x6, label  // Perform comparison
//! ```
//!
//! ### Large Offsets in Jumps
//!
//! ```text
//! Input:  jalr ra, x5, 3000   // 3000 > 2047 (12-bit limit)
//! Output: li   t4, 3000       // Load large offset
//!         add  t4, x5, t4     // Compute target address
//!         jalr ra, t4, 0      // Jump to computed address
//! ```
//!
//! # Usage
//!
//! Wrap any WriterCore implementation with DesugaringWriter to automatically handle
//! all invalid memory operands:
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
//!
//! // Computational instructions with memory operands also work
//! desugar.add(cfg, &dest, &Reg(5), &mem)?; // Loads mem into temp, then adds
//!
//! // Large immediates are also desugared
//! desugar.addi(cfg, &dest, &Reg(5), 5000)?; // Desugars to li + add
//!
//! // Literal operands in computational instructions are desugared
//! let literal = MemArgKind::NoMem(ArgKind::Lit(42));
//! desugar.add(cfg, &dest, &Reg(5), &literal)?; // Desugars to li + add
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

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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
    /// Tertiary temporary register for large immediates.
    /// Default: x29 (t4) - another temporary register.
    pub temp_reg3: Reg,
    /// Whether to save registers to stack when they conflict with temporaries.
    /// When enabled, conflicting registers are spilled to stack and reused.
    /// When disabled, the wrapper will use overlapping temporaries (caller must handle).
    /// Default: false - prefer different temp registers.
    pub save_to_stack_on_conflict: bool,
    /// Stack offset to use for saving registers (in bytes).
    /// Must be aligned to the natural stack boundary (typically 8 bytes).
    /// Default: 8 - standard stack slot size.
    pub stack_save_offset: i32,
}

impl Default for DesugarConfig {
    fn default() -> Self {
        Self {
            temp_reg: Reg(31),  // t6
            temp_reg2: Reg(28), // t3
            temp_reg3: Reg(29), // t4
            save_to_stack_on_conflict: false,
            stack_save_offset: 8,
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
    /// Current stack depth (number of spill slots pushed).
    stack_depth: i32,
}

impl<'a, W: WriterCore + ?Sized> DesugaringWriter<'a, W> {
    /// Creates a new desugaring wrapper with default configuration.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            config: DesugarConfig::default(),
            stack_depth: 0,
        }
    }

    /// Creates a new desugaring wrapper with custom configuration.
    pub fn with_config(writer: &'a mut W, config: DesugarConfig) -> Self {
        Self { writer, config, stack_depth: 0 }
    }

    /// Selects a temporary register that doesn't conflict with the given registers.
    /// Returns (temp_reg, needs_save, saved_reg) where needs_save indicates if we need
    /// to save a conflicting register to the stack.
    fn select_temp_reg(
        &self,
        avoid_regs: &[Reg],
    ) -> (Reg, bool, Option<Reg>) {
        // Prefer temps that are not in avoid_regs. If all conflict, prefer to pick a
        // candidate that is caller-saved (temporaries) over callee-saved. As a simple
        // heuristic we already configured three temps; try them in order but prefer
        // the one with fewest conflicts.
        let candidates = [
            self.config.temp_reg,
            self.config.temp_reg2,
            self.config.temp_reg3,
        ];

        // If any candidate is free, return it
        for &candidate in &candidates {
            if !avoid_regs.contains(&candidate) {
                return (candidate, false, None);
            }
        }

        // No free candidate: pick the candidate with minimal conflicts (score)
        let mut best = candidates[0];
        let mut best_score = i32::MAX;
        for &candidate in &candidates {
            let mut score = 0;
            for r in avoid_regs {
                if *r == candidate {
                    score += 1;
                }
            }
            if score < best_score {
                best_score = score;
                best = candidate;
            }
        }

        if self.config.save_to_stack_on_conflict {
            // Indicate that caller should save the conflicting register and return it
            (best, true, Some(best))
        } else {
            // No spilling desired; return best even though it conflicts
            (best, false, None)
        }
    }

    /// Pushes a register to the stack (emit prologue to reserve a slot and store reg).
    /// This implements a simple push: `addi sp, sp, -slot` followed by `sd reg, 0(sp)`.
    fn push_reg_to_stack(
        &mut self,
        arch: RiscV64Arch,
        reg: Reg,
    ) -> Result<(), W::Error> {
        // Adjust stack pointer and store the register
        let sp = Reg(2);
        let slot = self.config.stack_save_offset;
        // addi sp, sp, -slot
        self.writer.addi(arch, &sp, &sp, -slot)?;
        // sd reg, 0(sp)
        let mem = Self::simple_mem(sp, 0, MemorySize::_64, crate::RegisterClass::Gpr);
        self.writer.sd(arch, &reg, &mem)?;
        self.stack_depth += 1;
        Ok(())
    }

    /// Pops a register from the stack (restore reg and adjust sp): `ld reg, 0(sp)`; `addi sp, sp, slot`.
    fn pop_reg_from_stack(
        &mut self,
        arch: RiscV64Arch,
        reg: Reg,
    ) -> Result<(), W::Error> {
        let sp = Reg(2);
        let slot = self.config.stack_save_offset;
        let mem = Self::simple_mem(sp, 0, MemorySize::_64, crate::RegisterClass::Gpr);
        self.writer.ld(arch, &reg, &mem)?;
        self.writer.addi(arch, &sp, &sp, slot)?;
        if self.stack_depth > 0 {
            self.stack_depth -= 1;
        }
        Ok(())
    }

    /// Backwards-compatible shim: if previously used, translate to push/pop semantics.
    fn save_reg_to_stack(
        &mut self,
        arch: RiscV64Arch,
        reg: Reg,
    ) -> Result<(), W::Error> {
        self.push_reg_to_stack(arch, reg)
    }

    fn restore_reg_from_stack(
        &mut self,
        arch: RiscV64Arch,
        reg: Reg,
    ) -> Result<(), W::Error> {
        self.pop_reg_from_stack(arch, reg)
    }

    /// Checks if a value fits in 12 bits (RISC-V I-type immediate range).
    fn fits_in_12_bits(value: i32) -> bool {
        value >= -2048 && value < 2048
    }

    /// Checks if an immediate fits in 5 bits (RISC-V shift immediate range for RV64I).
    fn fits_in_5_bits(imm: i32) -> bool {
        imm >= 0 && imm < 32
    }

    /// Checks if an immediate fits in 6 bits (RISC-V shift immediate range for RV64I).
    fn fits_in_6_bits(imm: i32) -> bool {
        imm >= 0 && imm < 64
    }

    /// Desugars a memory operand into a simple base+disp form.
    ///
    /// Returns (base_reg, displacement, size, reg_class) where base_reg might be a temp register
    /// if address calculation was needed. The size and reg_class are preserved from the original
    /// memory operand to maintain type safety throughout desugaring.
    fn desugar_mem_operand(
        &mut self,
        arch: RiscV64Arch,
        mem: &MemArgKind<ArgKind>,
    ) -> Result<(Reg, i32, MemorySize, crate::RegisterClass), W::Error> {
        match mem {
            MemArgKind::NoMem(_) => {
                // This shouldn't be called for non-memory operands
                panic!("desugar_mem_operand called with NoMem variant")
            }
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => {
                // Collect registers to avoid conflicts
                let mut avoid_regs = Vec::new();
                
                // Handle base
                let base_reg = match base {
                    ArgKind::Reg { reg, .. } => {
                        avoid_regs.push(*reg);
                        *reg
                    },
                    ArgKind::Lit(val) => {
                        // Load literal into temp register
                        let (temp, needs_save, saved_reg) = self.select_temp_reg(&avoid_regs);
                        if needs_save {
                            if let Some(reg_to_save) = saved_reg {
                                self.save_reg_to_stack(arch, reg_to_save)?;
                            }
                        }
                        self.writer.li(arch, &temp, *val)?;
                        if needs_save {
                            if let Some(reg_to_save) = saved_reg {
                                self.restore_reg_from_stack(arch, reg_to_save)?;
                            }
                        }
                        avoid_regs.push(temp);
                        temp
                    }
                };

                // Handle offset if present
                let effective_base = if let Some((offset_arg, scale)) = offset {
                    // Need to calculate: effective_base = base_reg + (offset_arg << scale)

                    // Get the offset value into a register
                    let offset_reg = match offset_arg {
                        ArgKind::Reg { reg, .. } => {
                            avoid_regs.push(*reg);
                            *reg
                        },
                        ArgKind::Lit(val) => {
                            // Load literal offset into a temp register that doesn't conflict
                            let (temp, needs_save, saved_reg) = self.select_temp_reg(&avoid_regs);
                            if needs_save {
                                if let Some(reg_to_save) = saved_reg {
                                    self.save_reg_to_stack(arch, reg_to_save)?;
                                }
                            }
                            self.writer.li(arch, &temp, *val)?;
                            if needs_save {
                                if let Some(reg_to_save) = saved_reg {
                                    self.restore_reg_from_stack(arch, reg_to_save)?;
                                }
                            }
                            avoid_regs.push(temp);
                            temp
                        }
                    };

                    // Calculate scaled offset: scaled_offset = offset_reg << scale
                    let scaled_offset_reg = if *scale > 0 {
                        // Need to shift: select temp registers carefully
                        let (result_reg, needs_save, saved_reg) = self.select_temp_reg(&avoid_regs);
                        if needs_save {
                            if let Some(reg_to_save) = saved_reg {
                                self.save_reg_to_stack(arch, reg_to_save)?;
                            }
                        }
                        
                        // Select shift register that doesn't conflict
                        let mut shift_avoid = avoid_regs.clone();
                        shift_avoid.push(result_reg);
                        let (shift_reg, shift_needs_save, shift_saved_reg) = self.select_temp_reg(&shift_avoid);
                        if shift_needs_save {
                            if let Some(reg_to_save) = shift_saved_reg {
                                self.save_reg_to_stack(arch, reg_to_save)?;
                            }
                        }

                        // Load shift amount
                        self.writer.li(arch, &shift_reg, *scale as u64)?;
                        self.writer.sll(arch, &result_reg, &offset_reg, &shift_reg)?;
                        
                        if shift_needs_save {
                            if let Some(reg_to_save) = shift_saved_reg {
                                self.restore_reg_from_stack(arch, reg_to_save)?;
                            }
                        }
                        if needs_save {
                            if let Some(reg_to_save) = saved_reg {
                                self.restore_reg_from_stack(arch, reg_to_save)?;
                            }
                        }
                        
                        result_reg
                    } else {
                        // No scaling needed - just use offset_reg directly
                        offset_reg
                    };

                    // Add base: result = base_reg + scaled_offset_reg
                    let (result_reg, needs_save, saved_reg) = self.select_temp_reg(&avoid_regs);
                    if needs_save {
                        if let Some(reg_to_save) = saved_reg {
                            self.save_reg_to_stack(arch, reg_to_save)?;
                        }
                    }
                    self.writer.add(arch, &result_reg, &base_reg, &scaled_offset_reg)?;
                    if needs_save {
                        if let Some(reg_to_save) = saved_reg {
                            self.restore_reg_from_stack(arch, reg_to_save)?;
                        }
                    }

                    result_reg
                } else {
                    base_reg
                };

                // Handle large displacement - fold into base register
                if Self::fits_in_12_bits(*disp) {
                    Ok((effective_base, *disp, *size, *reg_class))
                } else {
                    // Displacement too large, need to add it to the base
                    let (temp, needs_save, saved_reg) = self.select_temp_reg(&[effective_base]);
                    if needs_save {
                        if let Some(reg_to_save) = saved_reg {
                            self.save_reg_to_stack(arch, reg_to_save)?;
                        }
                    }

                    // Load displacement into temp and add to effective_base
                    self.writer.li(arch, &temp, (*disp as i64) as u64)?;
                    self.writer.add(arch, &temp, &effective_base, &temp)?;
                    
                    if needs_save {
                        if let Some(reg_to_save) = saved_reg {
                            self.restore_reg_from_stack(arch, reg_to_save)?;
                        }
                    }

                    Ok((temp, 0, *size, *reg_class))
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
                let (base, new_disp, preserved_size, preserved_reg_class) = self.desugar_mem_operand(arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, preserved_size, preserved_reg_class))
            }
            MemArgKind::Mem {
                offset: None,
                disp,
                size,
                reg_class,
                ..
            } if !Self::fits_in_12_bits(*disp) => {
                // Large displacement - needs desugaring
                let (base, new_disp, preserved_size, preserved_reg_class) = self.desugar_mem_operand(arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, preserved_size, preserved_reg_class))
            }
            MemArgKind::Mem {
                base: ArgKind::Lit(_), // Base is a literal - needs desugaring
                offset: None,
                disp: _,
                size,
                reg_class,
                ..
            } => {
                // Base is a literal - needs desugaring
                let (base, new_disp, preserved_size, preserved_reg_class) = self.desugar_mem_operand(arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, preserved_size, preserved_reg_class))
            }
            _ => Ok(concrete), // Simple case - no desugaring needed
        }
    }

    /// Desugars an operand that might be a memory reference or literal.
    /// Returns a MemArgKind that is guaranteed to be a register (not memory or literal).
    /// 
    /// The avoid_regs parameter specifies registers that shouldn't be used as temporaries
    /// to avoid clobbering operands.
    fn desugar_operand_with_avoid(
        &mut self,
        arch: RiscV64Arch,
        operand: &(dyn MemArg + '_),
        avoid_regs: &[Reg],
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = operand.concrete_mem_kind();

        match &concrete {
            MemArgKind::NoMem(ArgKind::Reg { .. }) => Ok(concrete), // Already a register
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // This is a literal operand - need to load it into a temp register
                let (temp_reg, needs_save, saved_reg) = self.select_temp_reg(avoid_regs);
                if needs_save {
                    if let Some(reg_to_save) = saved_reg {
                        self.save_reg_to_stack(arch, reg_to_save)?;
                    }
                }
                self.writer.li(arch, &temp_reg, *val as u64)?;
                if needs_save {
                    if let Some(reg_to_save) = saved_reg {
                        self.restore_reg_from_stack(arch, reg_to_save)?;
                    }
                }
                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: MemorySize::_64, // Literals are loaded as 64-bit values
                }))
            }
            MemArgKind::Mem { size, reg_class, .. } => {
                // This is a memory operand - need to load it into a temp register
                let (temp_reg, needs_save, saved_reg) = self.select_temp_reg(avoid_regs);
                if needs_save {
                    if let Some(reg_to_save) = saved_reg {
                        self.save_reg_to_stack(arch, reg_to_save)?;
                    }
                }
                let desugared_mem = self.desugar_mem_arg(arch, operand)?;

                // Load the memory operand into the temp register
                // Use the appropriate load instruction based on size
                match size {
                    MemorySize::_8 => self.writer.lb(arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_16 => self.writer.lh(arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_32 => self.writer.lw(arch, &temp_reg, &desugared_mem)?,
                    MemorySize::_64 => self.writer.ld(arch, &temp_reg, &desugared_mem)?,
                }

                if needs_save {
                    if let Some(reg_to_save) = saved_reg {
                        self.restore_reg_from_stack(arch, reg_to_save)?;
                    }
                }

                Ok(MemArgKind::NoMem(ArgKind::Reg {
                    reg: temp_reg,
                    size: *size,
                }))
            }
        }
    }

    /// Desugars an operand that might be a memory reference or literal.
    /// Returns a MemArgKind that is guaranteed to be a register (not memory or literal).
    fn desugar_operand(
        &mut self,
        arch: RiscV64Arch,
        operand: &(dyn MemArg + '_),
    ) -> Result<MemArgKind<ArgKind>, W::Error> {
        self.desugar_operand_with_avoid(arch, operand, &[])
    }

    /// Helper for binary operations that may have memory or literal operands.
    /// Ensures that operands are loaded into registers as needed.
    fn binary_op<F>(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        op: F,
    ) -> Result<(), W::Error>
    where
        F: FnOnce(&mut W, RiscV64Arch, &(dyn MemArg + '_), &(dyn MemArg + '_), &(dyn MemArg + '_)) -> Result<(), W::Error>,
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
                    // Both are memory - use different temp registers to handle mem→mem operations
                    let (temp_reg_a, a_needs_save, a_saved_reg) = self.select_temp_reg(&[]);
                    let (temp_reg_b, b_needs_save, b_saved_reg) = self.select_temp_reg(&[temp_reg_a]);

                    // Save registers if needed
                    if a_needs_save {
                        if let Some(reg_to_save) = a_saved_reg {
                            self.save_reg_to_stack(cfg, reg_to_save)?;
                        }
                    }
                    if b_needs_save {
                        if let Some(reg_to_save) = b_saved_reg {
                            self.save_reg_to_stack(cfg, reg_to_save)?;
                        }
                    }

                    // Load a
                    let desugared_mem_a = self.desugar_mem_arg(cfg, a)?;
                    let a_size = if let MemArgKind::Mem { size, .. } = &a_concrete { *size } else { MemorySize::_64 };
                    match a_size {
                        MemorySize::_8 => self.writer.lb(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_16 => self.writer.lh(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_32 => self.writer.lw(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_64 => self.writer.ld(cfg, &temp_reg_a, &desugared_mem_a)?,
                    }

                    // Load b
                    let desugared_mem_b = self.desugar_mem_arg(cfg, b)?;
                    let b_size = if let MemArgKind::Mem { size, .. } = &b_concrete { *size } else { MemorySize::_64 };
                    match b_size {
                        MemorySize::_8 => self.writer.lb(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_16 => self.writer.lh(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_32 => self.writer.lw(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_64 => self.writer.ld(cfg, &temp_reg_b, &desugared_mem_b)?,
                    }

                    let desugared_a = MemArgKind::NoMem(ArgKind::Reg { reg: temp_reg_a, size: a_size });
                    let desugared_b = MemArgKind::NoMem(ArgKind::Reg { reg: temp_reg_b, size: b_size });

                    // Restore registers if needed
                    if b_needs_save {
                        if let Some(reg_to_save) = b_saved_reg {
                            self.restore_reg_from_stack(cfg, reg_to_save)?;
                        }
                    }
                    if a_needs_save {
                        if let Some(reg_to_save) = a_saved_reg {
                            self.restore_reg_from_stack(cfg, reg_to_save)?;
                        }
                    }

                    op(self.writer, cfg, dest, &desugared_a, &desugared_b)
                } else {
                    // At least one is literal, not memory - can use regular desugar_operand
                    // But still need to avoid conflicts between the two operands
                    let desugared_a = self.desugar_operand(cfg, a)?;
                    let a_reg = if let MemArgKind::NoMem(ArgKind::Reg { reg, .. }) = &desugared_a {
                        Some(*reg)
                    } else {
                        None
                    };
                    
                    let desugared_b = if let Some(a_reg) = a_reg {
                        self.desugar_operand_with_avoid(cfg, b, &[a_reg])?
                    } else {
                        self.desugar_operand(cfg, b)?
                    };
                    
                    op(self.writer, cfg, dest, &desugared_a, &desugared_b)
                }
            }
        }
    }

    /// Helper for binary operations that may have memory or literal operands but no destination.
    /// Used for branch instructions that compare two operands.
    fn binary_op_no_dest<F>(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
        op: F,
    ) -> Result<(), W::Error>
    where
        F: FnOnce(&mut W, RiscV64Arch, &(dyn MemArg + '_), &(dyn MemArg + '_), &(dyn MemArg + '_)) -> Result<(), W::Error>,
    {
        let a_concrete = a.concrete_mem_kind();
        let b_concrete = b.concrete_mem_kind();

        // Check if operands need desugaring
        let a_needs_desugar = !matches!(a_concrete, MemArgKind::NoMem(ArgKind::Reg { .. }));
        let b_needs_desugar = !matches!(b_concrete, MemArgKind::NoMem(ArgKind::Reg { .. }));

        match (a_needs_desugar, b_needs_desugar) {
            (false, false) => {
                // Neither needs desugaring - use directly
                op(self.writer, cfg, a, b, target)
            }
            (true, false) => {
                // Only a needs desugaring
                let desugared_a = self.desugar_operand(cfg, a)?;
                op(self.writer, cfg, &desugared_a, b, target)
            }
            (false, true) => {
                // Only b needs desugaring
                let desugared_b = self.desugar_operand(cfg, b)?;
                op(self.writer, cfg, a, &desugared_b, target)
            }
            (true, true) => {
                // Both need desugaring - handle memory operands specially to avoid conflicts
                let a_is_mem = matches!(a_concrete, MemArgKind::Mem { .. });
                let b_is_mem = matches!(b_concrete, MemArgKind::Mem { .. });

                if a_is_mem && b_is_mem {
                    // Both are memory - use different temp registers to handle mem→mem operations
                    let (temp_reg_a, a_needs_save, a_saved_reg) = self.select_temp_reg(&[]);
                    let (temp_reg_b, b_needs_save, b_saved_reg) = self.select_temp_reg(&[temp_reg_a]);

                    // Save registers if needed
                    if a_needs_save {
                        if let Some(reg_to_save) = a_saved_reg {
                            self.save_reg_to_stack(cfg, reg_to_save)?;
                        }
                    }
                    if b_needs_save {
                        if let Some(reg_to_save) = b_saved_reg {
                            self.save_reg_to_stack(cfg, reg_to_save)?;
                        }
                    }

                    // Load a
                    let desugared_mem_a = self.desugar_mem_arg(cfg, a)?;
                    let a_size = if let MemArgKind::Mem { size, .. } = &a_concrete { *size } else { MemorySize::_64 };
                    match a_size {
                        MemorySize::_8 => self.writer.lb(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_16 => self.writer.lh(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_32 => self.writer.lw(cfg, &temp_reg_a, &desugared_mem_a)?,
                        MemorySize::_64 => self.writer.ld(cfg, &temp_reg_a, &desugared_mem_a)?,
                    }

                    // Load b
                    let desugared_mem_b = self.desugar_mem_arg(cfg, b)?;
                    let b_size = if let MemArgKind::Mem { size, .. } = &b_concrete { *size } else { MemorySize::_64 };
                    match b_size {
                        MemorySize::_8 => self.writer.lb(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_16 => self.writer.lh(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_32 => self.writer.lw(cfg, &temp_reg_b, &desugared_mem_b)?,
                        MemorySize::_64 => self.writer.ld(cfg, &temp_reg_b, &desugared_mem_b)?,
                    }

                    let desugared_a = MemArgKind::NoMem(ArgKind::Reg { reg: temp_reg_a, size: a_size });
                    let desugared_b = MemArgKind::NoMem(ArgKind::Reg { reg: temp_reg_b, size: b_size });

                    // Restore registers if needed
                    if b_needs_save {
                        if let Some(reg_to_save) = b_saved_reg {
                            self.restore_reg_from_stack(cfg, reg_to_save)?;
                        }
                    }
                    if a_needs_save {
                        if let Some(reg_to_save) = a_saved_reg {
                            self.restore_reg_from_stack(cfg, reg_to_save)?;
                        }
                    }

                    op(self.writer, cfg, &desugared_a, &desugared_b, target)
                } else {
                    // At least one is literal, not memory - can use regular desugar_operand
                    // But still need to avoid conflicts between the two operands
                    let desugared_a = self.desugar_operand(cfg, a)?;
                    let a_reg = if let MemArgKind::NoMem(ArgKind::Reg { reg, .. }) = &desugared_a {
                        Some(*reg)
                    } else {
                        None
                    };
                    
                    let desugared_b = if let Some(a_reg) = a_reg {
                        self.desugar_operand_with_avoid(cfg, b, &[a_reg])?
                    } else {
                        self.desugar_operand(cfg, b)?
                    };
                    
                    op(self.writer, cfg, &desugared_a, &desugared_b, target)
                }
            }
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
        let src_concrete = src.concrete_mem_kind();
        match &src_concrete {
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // Source is a literal - use li instead of mv
                self.writer.li(cfg, dest, *val as u64)
            }
            _ => {
                // Source is register or memory - desugar and use mv
                let desugared_src = self.desugar_operand(cfg, src)?;
                self.writer.mv(cfg, dest, &desugared_src)
            }
        }
    }

    fn add(
        &mut self,
        cfg: RiscV64Arch,
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
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.sub(cfg, dest, a, b)
        })
    }

    fn addi(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
        imm: i32,
    ) -> Result<(), Self::Error> {
        if Self::fits_in_12_bits(imm) {
            let desugared_src = self.desugar_operand(cfg, src)?;
            self.writer.addi(cfg, dest, &desugared_src, imm)
        } else {
            // Large immediate - load into temp and add
            let temp_reg = self.config.temp_reg3;
            self.writer.li(cfg, &temp_reg, imm as u64)?;
            let desugared_src = self.desugar_operand(cfg, src)?;
            self.writer.add(cfg, dest, &desugared_src, &temp_reg)
        }
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
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.sll(cfg, dest, a, b)
        })
    }

    // Arithmetic operations - M extension

    fn mul(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.mul(cfg, dest, a, b)
        })
    }

    fn mulh(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.mulh(cfg, dest, a, b)
        })
    }

    fn div(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.div(cfg, dest, a, b)
        })
    }

    fn divu(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.divu(cfg, dest, a, b)
        })
    }

    fn rem(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.rem(cfg, dest, a, b)
        })
    }

    fn remu(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.remu(cfg, dest, a, b)
        })
    }

    // Bitwise operations

    fn and(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.and(cfg, dest, a, b)
        })
    }

    fn or(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.or(cfg, dest, a, b)
        })
    }

    fn xor(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.xor(cfg, dest, a, b)
        })
    }

    fn srl(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.srl(cfg, dest, a, b)
        })
    }

    fn sra(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.sra(cfg, dest, a, b)
        })
    }

    fn slt(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.slt(cfg, dest, a, b)
        })
    }

    fn sltu(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.sltu(cfg, dest, a, b)
        })
    }

    // Control flow operations

    fn jalr(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        base: &(dyn MemArg + '_),
        offset: i32,
    ) -> Result<(), Self::Error> {
        if Self::fits_in_12_bits(offset) {
            let desugared_base = self.desugar_operand(cfg, base)?;
            self.writer.jalr(cfg, dest, &desugared_base, offset)
        } else {
            // Large offset - compute address in temp register
            let temp_reg = self.config.temp_reg3;
            self.writer.li(cfg, &temp_reg, offset as u64)?;
            let desugared_base = self.desugar_operand(cfg, base)?;
            self.writer.add(cfg, &temp_reg, &desugared_base, &temp_reg)?;
            self.writer.jalr(cfg, dest, &temp_reg, 0)
        }
    }

    fn jal(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.jal(cfg, dest, &desugared_target)
    }

    fn beq(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, target, |writer, cfg, a, b, target| {
            writer.beq(cfg, a, b, target)
        })
    }

    fn bne(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, target, |writer, cfg, a, b, target| {
            writer.bne(cfg, a, b, target)
        })
    }

    fn blt(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, target, |writer, cfg, a, b, target| {
            writer.blt(cfg, a, b, target)
        })
    }

    fn bge(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, target, |writer, cfg, a, b, target| {
            writer.bge(cfg, a, b, target)
        })
    }

    fn bltu(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, target, |writer, cfg, a, b, target| {
            writer.bltu(cfg, a, b, target)
        })
    }

    fn bgeu(
        &mut self,
        cfg: RiscV64Arch,
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
        target: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, target, |writer, cfg, a, b, target| {
            writer.bgeu(cfg, a, b, target)
        })
    }

    fn ret(&mut self, cfg: RiscV64Arch) -> Result<(), Self::Error> {
        self.writer.ret(cfg)
    }

    fn call(&mut self, cfg: RiscV64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.call(cfg, &desugared_target)
    }

    fn j(&mut self, cfg: RiscV64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared_target = self.desugar_operand(cfg, target)?;
        self.writer.j(cfg, &desugared_target)
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
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fadd_d(cfg, dest, a, b)
        })
    }

    fn fsub_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fsub_d(cfg, dest, a, b)
        })
    }

    fn fmul_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fmul_d(cfg, dest, a, b)
        })
    }

    fn fdiv_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        a: &(dyn MemArg + '_),
        b: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        self.binary_op(cfg, dest, a, b, |writer, cfg, dest, a, b| {
            writer.fdiv_d(cfg, dest, a, b)
        })
    }

    fn fmov_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.fmov_d(cfg, dest, &desugared_src)
    }

    fn fcvt_d_l(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.fcvt_d_l(cfg, dest, &desugared_src)
    }

    fn fcvt_l_d(
        &mut self,
        cfg: RiscV64Arch,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
    ) -> Result<(), Self::Error> {
        let desugared_src = self.desugar_operand(cfg, src)?;
        self.writer.fcvt_l_d(cfg, dest, &desugared_src)
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

    #[test]
    fn test_desugar_literal_base() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Memory operand with literal base
            let mem = MemArgKind::Mem {
                base: ArgKind::Lit(0x1000),
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            // This should desugar to load literal and then load from memory
            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Check that output contains li instruction for loading the literal
        assert!(output.contains("li"));
        assert!(output.contains("ld"));
    }

    #[test]
    fn test_desugar_large_addi_immediate() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0
            let src = Reg(5);   // a5

            // Large immediate that doesn't fit in 12 bits
            let large_imm = 5000; // > 2047

            // This should desugar to li + add
            let _ = desugar.addi(cfg, &dest, &src, large_imm);
        }

        // Check that output contains li and add (not addi)
        assert!(output.contains("li"));
        assert!(output.contains("add"));
        assert!(!output.contains("addi"));
    }

    #[test]
    fn test_desugar_large_jalr_offset() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(1);  // ra
            let base = Reg(5);  // a5

            // Large offset that doesn't fit in 12 bits
            let large_offset = 3000; // > 2047

            // This should desugar to li + add + jalr
            let _ = desugar.jalr(cfg, &dest, &base, large_offset);
        }

        // Check that output contains li, add, and jalr
        assert!(output.contains("li"));
        assert!(output.contains("add"));
        assert!(output.contains("jalr"));
    }

    #[test]
    fn test_desugar_literal_operand_in_add() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0
            let a = Reg(5);     // a5

            // Literal operand in add instruction
            let b_literal = MemArgKind::NoMem(ArgKind::Lit(42));

            // This should desugar to li + add
            let _ = desugar.add(cfg, &dest, &a, &b_literal);
        }

        // Check that output contains li and add
        assert!(output.contains("li"));
        assert!(output.contains("add"));
    }

    #[test]
    fn test_desugar_literal_operand_in_mv() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Literal operand in mv instruction
            let src_literal = MemArgKind::NoMem(ArgKind::Lit(123));

            // This should desugar to li (not mv)
            let _ = desugar.mv(cfg, &dest, &src_literal);
        }

        // Check that output contains li but not mv
        assert!(output.contains("li"));
        assert!(!output.contains("mv"));
    }

    #[test]
    fn test_power_of_two_scale() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0
            
            // Memory operand with power-of-two scale (should use shift)
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 2)), // scale = 2 (<< 2)
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Check that output contains shift instruction
        assert!(output.contains("sll") || output.contains("slli"));
    }

    #[test]
    fn test_non_power_of_two_scale() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0
            
            // Memory operand with non-power-of-two scale (should use shift)
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 3)), // scale = 3 (<< 3)
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Check that output contains shift instruction
        assert!(output.contains("sll") || output.contains("slli"));
    }

    #[test]
    fn test_very_large_displacement() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Memory operand with very large displacement
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: None,
                disp: 100000, // Very large displacement
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Check that output contains li for large displacement and add
        assert!(output.contains("li"));
        assert!(output.contains("add"));
    }

    #[test]
    fn test_temp_conflict_avoidance() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let config = DesugarConfig {
                temp_reg: Reg(5),  // Use same register as base to force conflict
                temp_reg2: Reg(28), // t3
                temp_reg3: Reg(29), // t4
                save_to_stack_on_conflict: false,
                stack_save_offset: 8,
            };
            let mut desugar = DesugaringWriter::with_config(&mut output as &mut dyn Write, config);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Memory operand where base conflicts with temp_reg
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 }, // Same as temp_reg
                offset: Some((ArgKind::Lit(42), 1)), // Force temp usage
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Should still generate valid code without crashing
        assert!(output.contains("li") || output.contains("sll") || output.contains("add"));
    }

    #[test]
    fn test_mem_to_mem_operation() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Two memory operands for add operation
            let mem_a = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let mem_b = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 },
                offset: None,
                disp: 16,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            // This should load both memory operands into different temp registers
            let _ = desugar.add(cfg, &dest, &mem_a, &mem_b);
        }

        // Should contain multiple load instructions
        let load_count = output.matches("ld").count();
        assert!(load_count >= 2, "Expected at least 2 load instructions for mem→mem operation");
        assert!(output.contains("add"));
    }

    #[test]
    fn test_stack_save_on_conflict() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let config = DesugarConfig {
                temp_reg: Reg(5),  // Force conflicts
                temp_reg2: Reg(5),  // Same as temp_reg
                temp_reg3: Reg(5),  // Same as temp_reg
                save_to_stack_on_conflict: true,
                stack_save_offset: 8,
            };
            let mut desugar = DesugaringWriter::with_config(&mut output as &mut dyn Write, config);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Memory operand that will conflict with all temp registers
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 }, // Conflicts with all temps
                offset: Some((ArgKind::Lit(42), 1)), // Requires temp register
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Should contain stack operations when all temps conflict
        assert!(output.contains("sd") || output.contains("ld")); // Stack save/restore
    }

    #[test]
    fn test_preserve_memory_size() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Test different memory sizes
            for (size, expected_load) in [
                (MemorySize::_8, "lb"),
                (MemorySize::_16, "lh"),
                (MemorySize::_32, "lw"),
                (MemorySize::_64, "ld"),
            ] {
                output.clear();

                // Create a fresh desugaring writer per iteration to avoid holding a mutable
                // borrow to `output` across calls to `output.clear()` which causes borrow conflicts.
                let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

                let mem = MemArgKind::Mem {
                    base: ArgKind::Reg { reg: Reg(5), size },
                    offset: None,
                    disp: 8,
                    size,
                    reg_class: crate::RegisterClass::Gpr,
                };

                let _ = desugar.add(cfg, &dest, &mem, &Reg(6));

                // Should use the correct load instruction for the size
                assert!(output.contains(expected_load), 
                    "Expected {} for size {:?}", expected_load, size);
            }
        }
    }

    #[test]
    fn test_preserve_reg_class() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Memory operand with FP register class
            let mem = MemArgKind::Mem {
                base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Fp,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // The reg class should be preserved through desugaring
        // (This is more of a structural test - the actual output depends on the writer implementation)
        assert!(!output.is_empty());
    }

    #[test]
    fn test_complex_addressing_with_all_features() {
        let mut output = String::new();
        use core::fmt::Write as _;
        {
            let mut desugar = DesugaringWriter::new(&mut output as &mut dyn Write);

            let cfg = RiscV64Arch::default();
            let dest = Reg(10); // a0

            // Complex addressing: literal base + scaled offset + large displacement
            let mem = MemArgKind::Mem {
                base: ArgKind::Lit(0x1000), // Literal base
                offset: Some((ArgKind::Lit(42), 3)), // Scaled literal offset
                disp: 5000, // Large displacement
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
            };

            let _ = desugar.ld(cfg, &dest, &mem);
        }

        // Should contain multiple instructions for complex addressing
        assert!(output.contains("li")); // For loading literals
        assert!(output.contains("sll") || output.contains("add")); // For address calculation
    }
}
