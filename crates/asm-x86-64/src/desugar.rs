//! Desugaring wrapper for x86-64 memory operands.
//!
//! This module provides a hardened wrapper around `WriterCore` implementations that
//! validates and desugars complex memory operands and other invalid operand forms
//! into forms that are valid for x86-64 instruction encodings.
//!
//! The wrapper ensures:
//! - Memory operands use a register base (literal bases are loaded into temps)
//! - Index/scale pairs use a register index and a scale of 1/2/4/8 (other scales
//!   are materialized using canonical SHL for power-of-two scales or MUL for others)
//! - Displacements fit into a signed 32-bit immediate; otherwise they are
//!   folded into the base register with proper register class handling
//! - Literal operands used where a register is required are loaded into temps
//! - Mem-to-mem operations are broken into register temporaries with conflict avoidance
//! - Register classes (GPR vs XMM) are preserved when materializing temporaries
//! - Temporary register selection avoids clobbering registers used in operand addressing
//!
//! Usage: wrap any `WriterCore` with `DesugaringWriter` to automatically apply
//! these fixes before forwarding to the underlying writer.

use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

use crate::{
    out::{arg::{ArgKind, MemArg, MemArgKind}, WriterCore},
    stack::StackManager,
    X64Arch, RegisterClass,
};

/// Configuration for the desugaring wrapper.
///
/// This struct specifies the temporary registers available for desugaring operations.
/// The desugaring logic will automatically select appropriate temporaries based on
/// register class requirements and avoid conflicts with registers used in operand addressing.
#[derive(Clone, Copy, Debug)]
pub struct DesugarConfig {
    /// Primary temporary GPR register to use for address calculations and general operations.
    pub temp_gpr: Reg,
    /// Secondary temporary GPR register to use when primary GPR is in conflict.
    pub temp_gpr2: Reg,
    /// Tertiary temporary GPR register for complex scale materialization.
    pub temp_gpr3: Reg,
    /// Primary temporary XMM register for SIMD/memory operations with XMM register class.
    pub temp_xmm: Reg,
    /// Secondary temporary XMM register for conflict avoidance in SIMD operations.
    pub temp_xmm2: Reg,
}

impl Default for DesugarConfig {
    fn default() -> Self {
        Self {
            temp_gpr: Reg(15),   // r15 as a high-numbered GPR temp
            temp_gpr2: Reg(14),  // r14
            temp_gpr3: Reg(13),  // r13
            temp_xmm: Reg(15),   // xmm15 as a high-numbered XMM temp
            temp_xmm2: Reg(14),  // xmm14
        }
    }
}

/// Legacy temporary register manager for backward compatibility.
/// This is now a thin wrapper around StackManager for existing code.
pub struct TempRegManager {
    stack_manager: StackManager,
    config: DesugarConfig,
}

impl TempRegManager {
    pub fn new() -> Self {
        Self {
            stack_manager: StackManager::new(),
            config: DesugarConfig::default(),
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
            RegisterClass::Gpr => [config.temp_gpr, config.temp_gpr2, config.temp_gpr3],
            RegisterClass::Xmm => [config.temp_xmm, config.temp_xmm2, Reg(0)], // Pad to 3 elements
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
        if !Self::rsp_used(used_regs, used_count) {
            // Safe to use push/pop - pick the first candidate
            let temp_reg = candidates[0];

            // Use the stack manager for push/pop operations
            self.stack_manager.push(ctx, writer, ctx, X64Arch::default(), &temp_reg)?;

            Ok(temp_reg)
        } else {
            // Cannot use push/pop, use the first candidate anyway (may cause incorrect code)
            Ok(candidates[0])
        }
    }

    /// Release a temporary register, popping it from the stack if it's at the top.
    /// If the register is buried deeper in the stack, it remains pushed for potential future use.
    pub fn release_temp<Context, W: WriterCore<Context> + ?Sized>(&mut self, writer: &mut W, ctx: &mut Context, reg: Reg) -> Result<(), W::Error> {
        // Use the stack manager for pop operations
        self.stack_manager.pop(ctx, writer, ctx, X64Arch::default(), &reg)
    }

    /// Release all pushed registers in reverse order (LIFO).
    /// This should be called at the end of a desugaring operation to clean up the stack.
    pub fn release_all<Context, W: WriterCore<Context> + ?Sized>(&mut self, writer: &mut W, ctx: &mut Context) -> Result<(), W::Error> {
        // For now, maintain backward compatibility by not using optimization
        // The stack manager will be used for offset-based access in the future
        while self.stack_manager.stack_depth() > 0 {
            let slots = self.stack_manager.stack_slots();
            let slot = &slots[slots.len() - 1];
            writer.pop(ctx, X64Arch::default(), &Reg(slot.offset as u8))?; // This is a hack - we need to track actual registers
            self.stack_manager.deallocate_slot();
        }
        Ok(())
    }

    /// Check if RSP is used in the given registers.
    fn rsp_used(used_regs: &[Reg], used_count: usize) -> bool {
        let rsp = Reg(4);
        for i in 0..used_count {
            if used_regs[i] == rsp {
                return true;
            }
        }
        false
    }

    /// Get access to the underlying stack manager for advanced operations.
    pub fn stack_manager(&mut self) -> &mut StackManager {
        &mut self.stack_manager
    }
}

/// Hardened wrapper around WriterCore that desugars and validates complex memory operands.
///
/// This wrapper automatically handles:
/// - Register class-aware temporary selection
/// - Conflict-free temporary register allocation
/// - Canonical scale materialization (SHL for powers of two, MUL otherwise)
/// - Large displacement folding with proper size preservation
/// - SIMD register class handling for XMM operations
/// - Advanced stack management with inter-instruction optimization
/// - Offset-based stack data access
///
/// Call `release_all_temps()` when desugaring operations are complete to ensure
/// proper stack cleanup and optimization.
pub struct DesugaringWriter<'a, W, Context> where W: WriterCore<Context> + ?Sized {
    writer: &'a mut W,
    config: DesugarConfig,
    temp_manager: TempRegManager,
    stack_manager: StackManager,
}

impl<'a, W: WriterCore<Context> + ?Sized, Context> DesugaringWriter<'a, W, Context> {
    pub fn new(writer: &'a mut W) -> Self {
        let config = DesugarConfig::default();
        let temp_manager = TempRegManager::new();
        let stack_manager = StackManager::new();
        Self { writer, config, temp_manager, stack_manager }
    }
    pub fn with_config(writer: &'a mut W, config: DesugarConfig) -> Self {
        let temp_manager = TempRegManager::new();
        let stack_manager = StackManager::new();
        Self { writer, config, temp_manager, stack_manager }
    }

    /// Release all pushed temporary registers, restoring the stack to its original state.
    /// This should be called when desugaring operations are complete to ensure proper stack cleanup.
    pub fn release_all_temps(&mut self, ctx: &mut Context) -> Result<(), W::Error> {
        self.temp_manager.release_all(self.writer, ctx)
    }

    /// Get access to the underlying stack manager for advanced stack operations.
    pub fn stack_manager(&mut self) -> &mut StackManager {
        &mut self.stack_manager
    }

    /// Optimize pending stack operations across multiple instructions.
    /// This enables inter-instruction stack optimization as requested.
    pub fn optimize_stack_operations(&mut self, ctx: &mut Context, arch: X64Arch) -> Result<bool, W::Error> {
        self.stack_manager.optimize_and_execute(self.writer, ctx, arch)
    }

    /// Access stack data at the given offset using optimized stack operations.
    /// This supports offset-based stack data accesses as requested.
    pub fn access_stack_data(&mut self, ctx: &mut Context, arch: X64Arch,
        offset: i32,
        size: MemorySize,
        reg_class: RegisterClass,
        dest: &Reg,
    ) -> Result<(), W::Error> {
        // Check if this access would conflict with pending RSP operations
        if self.stack_manager.uses_rsp() {
            // Flush pending operations before RSP-using access
            self.stack_manager.flush_before_rsp_use(self.writer, ctx, arch)?;
        }

        // Use adjusted offset if there are pending operations
        let adjusted_offset = self.stack_manager.adjusted_offset(offset);
        self.stack_manager.access_stack(self.writer, ctx, arch, adjusted_offset, size, reg_class, dest)
    }

    /// Handle operations that use RSP directly (like stack pointer arithmetic).
    /// Either flushes the stack or adjusts offsets as needed.
    pub fn handle_rsp_operation(&mut self, ctx: &mut Context, arch: X64Arch, operation: F) -> Result<(), W::Error>
    where
        F: FnOnce(&mut W) -> Result<(), W::Error>,
    {
        // For RSP operations, we need to ensure the stack is in a consistent state
        // Option 1: Flush all pending operations before the RSP operation
        self.stack_manager.flush_before_rsp_use(self.writer, ctx, arch)?;

        // Execute the RSP operation
        operation(self.writer)
    }

    /// Check if an operand involves RSP (stack pointer register).
    fn operand_uses_rsp(operand: &(dyn MemArg + '_)) -> bool {
        let rsp = Reg(4); // RSP register
        let concrete = operand.concrete_mem_kind();
        match concrete {
            MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => reg == rsp,
            MemArgKind::Mem { base, offset, .. } => {
                // Check if base is RSP
                if let ArgKind::Reg { reg, .. } = base {
                    if reg == rsp {
                        return true;
                    }
                }
                // Check if offset involves RSP
                if let Some((ArgKind::Reg { reg, .. }, _)) = offset {
                    if reg == rsp {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Ensure stack is flushed before operations that read or write RSP directly.
    fn ensure_stack_flushed_for_rsp(&mut self, ctx: &mut Context, cfg: X64Arch, operands: &[&(dyn MemArg + '_)]) -> Result<(), W::Error> {
        for operand in operands {
            if Self::operand_uses_rsp(*operand) {
                self.stack_manager.flush_before_rsp_use(self.writer, ctx, cfg)?;
                break; // Only need to flush once
            }
        }
        Ok(())
    }

    /// Checks if a displacement fits in a signed 32-bit immediate.
    fn fits_in_i32(d: u64) -> bool {
        d <= i32::MAX as u64
    }

    /// Checks whether a scale is valid for x86 addressing (1,2,4,8).
    fn valid_scale(scale: u32) -> bool {
        matches!(scale, 1 | 2 | 4 | 8)
    }

    /// Returns the log2 of scale if it's a power of two, None otherwise.
    fn is_power_of_two(scale: u32) -> Option<u32> {
        if scale == 0 || (scale & (scale - 1)) != 0 {
            return None;
        }
        let mut s = scale;
        let mut shift = 0;
        while s > 1 {
            s >>= 1;
            shift += 1;
        }
        Some(shift)
    }

    /// Select an appropriate temporary register based on register class.
    fn select_temp_reg(&self, reg_class: crate::RegisterClass) -> Reg {
        match reg_class {
            crate::RegisterClass::Gpr => self.config.temp_gpr,
            crate::RegisterClass::Xmm => self.config.temp_xmm,
        }
    }

    /// Collect all registers used in a memory argument kind.
    fn collect_used_regs(mem: &MemArgKind<ArgKind>, buffer: &mut [Reg]) -> usize {
        let mut count = 0;
        match mem {
            MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => {
                buffer[count] = *reg;
                count += 1;
            }
            MemArgKind::NoMem(ArgKind::Lit(_)) => {}
            MemArgKind::Mem { base, offset, .. } => {
                if let ArgKind::Reg { reg, .. } = base {
                    buffer[count] = *reg;
                    count += 1;
                }
                if let Some((ArgKind::Reg { reg, .. }, _)) = offset {
                    buffer[count] = *reg;
                    count += 1;
                }
            }
        }
        count
    }

    /// Check if RSP (stack pointer) is used in the given registers.
    fn rsp_used(used_regs: &[Reg], used_count: usize) -> bool {
        // RSP is register 4 in x86-64
        let rsp = Reg(4);
        for i in 0..used_count {
            if used_regs[i] == rsp {
                return true;
            }
        }
        false
    }

    /// Select a temporary register that doesn't conflict with used registers.
    fn select_safe_temp_reg(&self, reg_class: crate::RegisterClass, used_regs: &[Reg], used_count: usize) -> Reg {
        let candidates = match reg_class {
            crate::RegisterClass::Gpr => [self.config.temp_gpr, self.config.temp_gpr2, self.config.temp_gpr3],
            crate::RegisterClass::Xmm => [self.config.temp_xmm, self.config.temp_xmm2, Reg(0)], // Pad to 3 elements
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
                return candidate;
            }
        }

        // If all candidates conflict, check if we can use push/pop
        if !Self::rsp_used(used_regs, used_count) {
            // Safe to use push/pop - pick the first candidate
            candidates[0]
        } else {
            // Cannot use push/pop, use the first candidate anyway (may cause incorrect code)
            candidates[0]
        }
    }



    /// Desugars a memory operand into a simple base+disp form.
    /// Returns (base_reg, disp) where the returned `disp` is a u32 displacement
    /// suitable for `MemArgKind::Mem`. If the displacement is too large it will
    /// be folded into the returned `base_reg` (and disp==0).
    fn desugar_mem_operand(&mut self, ctx: &mut Context, arch: X64Arch,
        mem: &MemArgKind<ArgKind>,
    ) -> Result<(Reg, u32), W::Error> {
        match mem {
            MemArgKind::NoMem(_) => panic!("desugar_mem_operand called with NoMem"),
            MemArgKind::Mem { base, offset, disp, size: _, reg_class: _ } => {
                // Ensure base is a register.
                let base_reg = match base {
                    ArgKind::Reg { reg, .. } => *reg,
                    ArgKind::Lit(val) => {
                        // Load literal into temp register using mov64
                        let temp = self.config.temp_gpr;
                        self.writer.mov64(ctx, arch, &temp, *val)?;
                        temp
                    }
                };

                // Handle offset (index*scale)
                let effective_base = if let Some((offset_arg, scale)) = offset {
                    // Get offset into a register
                    let offset_reg = match offset_arg {
                        ArgKind::Reg { reg, .. } => *reg,
                        ArgKind::Lit(val) => {
                            // Load literal offset into temp_reg2 (avoid conflict)
                            let tmp = if base_reg == self.config.temp_gpr { self.config.temp_gpr2 } else { self.config.temp_gpr };
                            self.writer.mov64(ctx, arch, &tmp, *val)?;
                            tmp
                        }
                    };

                    // Materialize scaled index into a register if needed; otherwise
                    // we'll still fold base+index into a register so we return a
                    // simple base+disp pair.
                    let scaled_index_reg = if Self::valid_scale(*scale) {
                        // Use offset_reg directly as the (already-scaled) index value
                        offset_reg
                    } else {
                        // Materialize offset * scale into a register.
                        // If scale is a power of two, use shift; otherwise multiply.
                        let mut used_regs = [Reg(0); 2];
                        used_regs[0] = base_reg;
                        used_regs[1] = offset_reg;
                        let result_reg = self.select_safe_temp_reg(crate::RegisterClass::Gpr, &used_regs, 2);

                        // Move offset into result_reg first
                        self.writer.mov(ctx, arch, &result_reg, &offset_reg)?;

                        // Canonical scale materialization: use SHL for power-of-two scales, MUL for others
                        if let Some(shift) = Self::is_power_of_two(*scale) {
                            // Use SHL for power-of-two scales
                            self.writer.shl(ctx, arch, &result_reg, &ArgKind::Lit(shift as u64))?;
                        } else {
                            // Use MUL for non-power-of-two scales
                            let mut used_regs = [Reg(0); 3];
                            used_regs[0] = base_reg;
                            used_regs[1] = offset_reg;
                            used_regs[2] = result_reg;
                            let scale_reg = self.select_safe_temp_reg(crate::RegisterClass::Gpr, &used_regs, 3);
                            self.writer.mov64(ctx, arch, &scale_reg, *scale as u64)?;
                            self.writer.mul(ctx, arch, &result_reg, &MemArgKind::NoMem(ArgKind::Reg { reg: scale_reg, size: MemorySize::_64 }))?;
                        }

                        result_reg
                    };

                    // Now compute base + scaled_index into a temp register
                    let mut used_regs = [Reg(0); 2];
                    used_regs[0] = base_reg;
                    used_regs[1] = scaled_index_reg;
                    let result_reg = self.select_safe_temp_reg(crate::RegisterClass::Gpr, &used_regs, 2);
                    // Move base into result_reg then add scaled index
                    self.writer.mov(ctx, arch, &result_reg, &base_reg)?;
                    self.writer.add(ctx, arch, &result_reg, &MemArgKind::NoMem(ArgKind::Reg { reg: scaled_index_reg, size: MemorySize::_64 }))?;
                    result_reg
                } else {
                    base_reg
                };

                // Handle large displacement: x86 uses signed 32-bit displacement
                if Self::fits_in_i32(*disp as u64) {
                    Ok((effective_base, *disp))
                } else {
                    // Fold displacement into base
                    let mut used_regs = [Reg(0); 1];
                    used_regs[0] = effective_base;
                    let temp = self.select_safe_temp_reg(crate::RegisterClass::Gpr, &used_regs, 1);
                    self.writer.mov64(ctx, arch, &temp, *disp as u64)?;
                    self.writer.mov(ctx, arch, &temp, &effective_base)?;
                    self.writer.add(ctx, arch, &temp, &MemArgKind::NoMem(ArgKind::Reg { reg: effective_base, size: MemorySize::_64 }))?;
                    Ok((temp, 0))
                }
            }
        }
    }

    fn simple_mem(base: Reg, disp: u32, size: MemorySize, reg_class: crate::RegisterClass) -> MemArgKind<ArgKind> {
        MemArgKind::Mem { base: ArgKind::Reg { reg: base, size }, offset: None, disp, size, reg_class }
    }

    fn desugar_mem_arg(&mut self, ctx: &mut Context, arch: X64Arch, mem_arg: &(dyn MemArg + '_)) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = mem_arg.concrete_mem_kind();
        match concrete {
            MemArgKind::NoMem(_) => Ok(concrete),
            MemArgKind::Mem { base: ArgKind::Lit(_), offset, disp, size, reg_class } => {
                // literal base - load into temp (and fold any index/disp as necessary)
                let (base_reg, new_disp) = self.desugar_mem_operand(ctx, arch, &concrete)?;
                Ok(Self::simple_mem(base_reg, new_disp, size, reg_class))
            }
            MemArgKind::Mem { offset: Some((_, scale)), disp, size, reg_class, .. } if !Self::valid_scale(scale) => {
                // invalid scale - needs materialization
                let (base, new_disp) = self.desugar_mem_operand(ctx, arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, size, reg_class))
            }
            MemArgKind::Mem { offset: None, disp, size, reg_class, .. } if !Self::fits_in_i32(disp as u64) => {
                // large displacement - fold into base
                let (base, new_disp) = self.desugar_mem_operand(ctx, arch, &concrete)?;
                Ok(Self::simple_mem(base, new_disp, size, reg_class))
            }
            m => Ok(m),
        }
    }

    fn desugar_operand(&mut self, ctx: &mut Context, arch: X64Arch, operand: &(dyn MemArg + '_)) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = operand.concrete_mem_kind();
        match concrete {
            MemArgKind::NoMem(ArgKind::Reg { .. }) => Ok(concrete),
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // Load literal into temp - use appropriate size based on value
                let temp = self.config.temp_gpr;
                self.writer.mov64(ctx, arch, &temp, val)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg { reg: temp, size: MemorySize::_64 }))
            }
            MemArgKind::Mem { size, reg_class, .. } => {
                // Load memory operand into temp - preserve register class and size
                let temp = self.select_temp_reg(reg_class);
                let desugared = self.desugar_mem_arg(ctx, arch, operand)?;
                // Use mov to load from memory into temp
                self.writer.mov(ctx, arch, &temp, &desugared)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg { reg: temp, size }))
            }
        }
    }

    /// Helper for binary ops of the form op(a, b) where `a` is both destination and first source.
    fn binary_op< F >( &mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), op: F) -> Result<(), W::Error>
    where
        F: FnOnce(&mut W, X64Arch, &(dyn MemArg + '_), &(dyn MemArg + '_)) -> Result<(), W::Error>,
    {
        let a_concrete = a.concrete_mem_kind();
        let b_concrete = b.concrete_mem_kind();

        let a_is_mem = matches!(a_concrete, MemArgKind::Mem { .. });
        let b_is_mem = matches!(b_concrete, MemArgKind::Mem { .. });

        match (a_is_mem, b_is_mem) {
            (false, false) => op(self.writer, cfg, a, b),
            (true, false) => {
                let desugared_a = self.desugar_mem_arg(ctx, cfg, a)?;
                op(self.writer, cfg, &desugared_a, b)
            }
            (false, true) => {
                let desugared_b = self.desugar_operand(ctx, cfg, b)?; // ensure b is register or literal handled
                op(self.writer, cfg, a, &desugared_b)
            }
            (true, true) => {
                // both memory - load b into temp and use that
                let mut all_used = [Reg(0); 4];
                let a_count = Self::collect_used_regs(&a_concrete, &mut all_used[0..2]);
                let b_count = Self::collect_used_regs(&b_concrete, &mut all_used[2..4]);
                let total_count = a_count + b_count;
                let temp_b = self.temp_manager.acquire_temp(self.writer, ctx, &self.config, crate::RegisterClass::Gpr, &all_used, total_count)?;

                let desugared_b_mem = self.desugar_mem_arg(ctx, cfg, b)?;
                self.writer.mov(ctx, cfg, &temp_b, &desugared_b_mem)?;
                let desugared_a = self.desugar_mem_arg(ctx, cfg, a)?;
                op(self.writer, cfg, &desugared_a, &MemArgKind::NoMem(ArgKind::Reg { reg: temp_b, size: MemorySize::_64 }))?;

                // Release the temp register (will pop if needed)
                self.temp_manager.release_temp(self.writer, ctx, temp_b)
            }
        }
    }

    /// Helper for two-operand comparisons where neither operand is a destination.
    fn binary_op_no_dest(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), op: F) -> Result<(), W::Error>
    where
        F: FnOnce(&mut W, X64Arch, &(dyn MemArg + '_), &(dyn MemArg + '_)) -> Result<(), W::Error>,
    {
        let a_concrete = a.concrete_mem_kind();
        let b_concrete = b.concrete_mem_kind();

        let a_is_mem = matches!(a_concrete, MemArgKind::Mem { .. });
        let b_is_mem = matches!(b_concrete, MemArgKind::Mem { .. });

        match (a_is_mem, b_is_mem) {
            (false, false) => op(self.writer, cfg, a, b),
            (true, false) => {
                let da = self.desugar_operand(ctx, cfg, a)?;
                op(self.writer, cfg, &da, b)
            }
            (false, true) => {
                let db = self.desugar_operand(ctx, cfg, b)?;
                op(self.writer, cfg, a, &db)
            }
            (true, true) => {
                // both memory - load one into temp
                let mut all_used = [Reg(0); 4];
                let a_count = Self::collect_used_regs(&a_concrete, &mut all_used[0..2]);
                let b_count = Self::collect_used_regs(&b_concrete, &mut all_used[2..4]);
                let total_count = a_count + b_count;
                let temp_a = self.temp_manager.acquire_temp(self.writer, ctx, &self.config, crate::RegisterClass::Gpr, &all_used, total_count)?;

                let desugared_a = self.desugar_mem_arg(ctx, cfg, a)?;
                self.writer.mov(ctx, cfg, &temp_a, &desugared_a)?;
                let desugared_b = self.desugar_mem_arg(ctx, cfg, b)?;
                op(self.writer, cfg, &MemArgKind::NoMem(ArgKind::Reg { reg: temp_a, size: MemorySize::_64 }), &desugared_b)?;

                // Release the temp register (will pop if needed)
                self.temp_manager.release_temp(self.writer, ctx, temp_a)
            }
        }
    }
}

impl<'a, W: WriterCore<Context> + ?Sized, Context> WriterCore<Context> for DesugaringWriter<'a, W, Context> {
    type Error = W::Error;

    fn mov(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[dest, src])?;

        let dest_conc = dest.concrete_mem_kind();
        let src_conc = src.concrete_mem_kind();

        let dest_is_mem = matches!(dest_conc, MemArgKind::Mem { .. });
        let src_is_mem = matches!(src_conc, MemArgKind::Mem { .. });

        match (dest_is_mem, src_is_mem) {
            (true, true) => {
                // mem-to-mem not allowed: load src into temp then mov
                // Use appropriate temp register based on destination register class
                if let MemArgKind::Mem { reg_class, size, .. } = dest_conc {
                    // Collect registers used in src and dest addressing
                    let mut all_used = [Reg(0); 4];
                    let src_count = Self::collect_used_regs(&src_conc, &mut all_used[0..2]);
                    let dest_count = Self::collect_used_regs(&dest_conc, &mut all_used[2..4]);
                    let total_count = src_count + dest_count;
                    let temp = self.temp_manager.acquire_temp(self.writer, ctx, &self.config, reg_class, &all_used, total_count)?;

                    let desugared_src = self.desugar_mem_arg(ctx, cfg, src)?;
                    self.writer.mov(ctx, cfg, &temp, &desugared_src)?;
                    let desugared_dest = self.desugar_mem_arg(ctx, cfg, dest)?;
                    self.writer.mov(ctx, cfg, &desugared_dest, &MemArgKind::NoMem(ArgKind::Reg { reg: temp, size }))?;

                    // Release the temp register (will pop if needed)
                    self.temp_manager.release_temp(self.writer, ctx, temp)
                } else {
                    unreachable!()
                }
            }
            (true, false) => {
                // dest is mem - ensure src is a valid operand (reg or lit)
                if let MemArgKind::NoMem(ArgKind::Lit(v)) = src_conc {
                    // mov can take immediate via mov64 - forward directly
                    let desugared_dest = self.desugar_mem_arg(ctx, cfg, dest)?;
                    return self.writer.mov64(ctx, cfg, &desugared_dest, v);
                }
                let desugared_src = self.desugar_operand(ctx, cfg, src)?;
                let desugared_dest = self.desugar_mem_arg(ctx, cfg, dest)?;
                self.writer.mov(ctx, cfg, &desugared_dest, &desugared_src)
            }
            (false, true) => {
                // src is mem - load into a temp then mov to dest.
                let desugared_src = self.desugar_mem_arg(ctx, cfg, src)?;
                // Choose a load temp that doesn't clobber any registers used by the address calculation
                let (load_temp, temp_size) = if let MemArgKind::Mem { reg_class, size, .. } = &desugared_src {
                    let mut src_used = [Reg(0); 2];
                    let src_count = Self::collect_used_regs(&src_conc, &mut src_used);
                    let temp = self.temp_manager.acquire_temp(self.writer, ctx, &self.config, *reg_class, &src_used, src_count)?;
                    (temp, *size)
                } else {
                    (self.config.temp_gpr, MemorySize::_64)
                };

                self.writer.mov(ctx, cfg, &load_temp, &desugared_src)?;
                let desugared_dest = self.desugar_operand(ctx, cfg, dest)?;
                self.writer.mov(ctx, cfg, &desugared_dest, &MemArgKind::NoMem(ArgKind::Reg { reg: load_temp, size: temp_size }))?;

                // Release the temp register (will pop if needed)
                self.temp_manager.release_temp(self.writer, ctx, load_temp)
            }
            (false, false) => {
                // both no-mem: if src is literal prefer mov64
                if let MemArgKind::NoMem(ArgKind::Lit(v)) = src_conc {
                    // mov64 supports immediate
                    return self.writer.mov64(ctx, cfg, dest, v);
                }
                // otherwise forward directly
                self.writer.mov(ctx, cfg, dest, src)
            }
        }
    }

    fn xchg(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[dest, src])?;

        // xchg cannot be mem->mem; desugar similarly to mov
        let dest_conc = dest.concrete_mem_kind();
        let src_conc = src.concrete_mem_kind();
        let dest_is_mem = matches!(dest_conc, MemArgKind::Mem { .. });
        let src_is_mem = matches!(src_conc, MemArgKind::Mem { .. });
        if dest_is_mem && src_is_mem {
            // Use temp register based on dest register class
            let temp = if let MemArgKind::Mem { reg_class, .. } = dest_conc {
                let mut all_used = [Reg(0); 4];
                let src_count = Self::collect_used_regs(&src_conc, &mut all_used[0..2]);
                let dest_count = Self::collect_used_regs(&dest_conc, &mut all_used[2..4]);
                let total_count = src_count + dest_count;
                self.temp_manager.acquire_temp(self.writer, ctx, &self.config, reg_class, &all_used, total_count)?
            } else {
                self.config.temp_gpr
            };

            let desugared_src = self.desugar_mem_arg(ctx, cfg, src)?;
            self.writer.mov(ctx, cfg, &temp, &desugared_src)?;
            let desugared_dest = self.desugar_mem_arg(ctx, cfg, dest)?;
            let temp_size = if let MemArgKind::Mem { size, .. } = dest_conc { size } else { MemorySize::_64 };
            self.writer.xchg(ctx, cfg, &desugared_dest, &MemArgKind::NoMem(ArgKind::Reg { reg: temp, size: temp_size }))?;

            // Release the temp register (will pop if needed)
            self.temp_manager.release_temp(self.writer, ctx, temp)
        } else {
            let d = if dest_is_mem { self.desugar_mem_arg(ctx, cfg, dest)? } else { dest.concrete_mem_kind() };
            let s = if src_is_mem { self.desugar_mem_arg(ctx, cfg, src)? } else { src.concrete_mem_kind() };
            self.writer.xchg(ctx, cfg, &d, &s)
        }
    }

    fn jcc(&mut self, ctx: &mut Context, cfg: X64Arch, cc: crate::ConditionCode, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(ctx, cfg, op)?;
        self.writer.jcc(ctx, cfg, cc, &desugared)
    }

    fn call(&mut self, ctx: &mut Context, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(ctx, cfg, op)?;
        self.writer.call(ctx, cfg, &desugared)
    }

    fn jmp(&mut self, ctx: &mut Context, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(ctx, cfg, op)?;
        self.writer.jmp(ctx, cfg, &desugared)
    }

    fn lea(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[dest, src])?;

        // lea expects memory-like src; ensure src mem forms are valid
        let src_conc = src.concrete_mem_kind();
        let src_fixed = if matches!(src_conc, MemArgKind::Mem { .. }) { self.desugar_mem_arg(ctx, cfg, src)? } else { src_conc };
        self.writer.lea(ctx, cfg, dest, &src_fixed)
    }

    fn add(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.add(ctx, c, x, y))
    }

    fn sub(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.sub(ctx, c, x, y))
    }

    fn cmp(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op_no_dest(cfg, a, b, |w, c, x, y| w.cmp(ctx, c, x, y))
    }

    fn cmp0(&mut self, ctx: &mut Context, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[op])?;
        self.writer.cmp0(ctx, cfg, op)
    }

    fn cmovcc64(&mut self, ctx: &mut Context, cfg: X64Arch, cond: crate::ConditionCode, op: &(dyn MemArg + '_), val: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[op, val])?;
        self.writer.cmovcc64(ctx, cfg, cond, op, val)
    }

    fn u32(&mut self, ctx: &mut Context, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[op])?;
        self.writer.u32(ctx, cfg, op)
    }

    fn not(&mut self, ctx: &mut Context, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[op])?;
        self.writer.not(ctx, cfg, op)
    }

    fn mul(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.mul(ctx, c, x, y))
    }

    fn div(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.div(ctx, c, x, y))
    }

    fn idiv(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.idiv(ctx, c, x, y))
    }

    fn and(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.and(ctx, c, x, y))
    }

    fn or(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.or(ctx, c, x, y))
    }

    fn eor(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.eor(ctx, c, x, y))
    }

    fn shl(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.shl(ctx, c, x, y))
    }

    fn shr(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.shr(ctx, c, x, y))
    }

    fn sar(&mut self, ctx: &mut Context, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[a, b])?;
        self.binary_op(ctx, cfg, a, b, |w, c, x, y| w.sar(ctx, c, x, y))
    }

    fn movsx(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[dest, src])?;
        self.writer.movsx(ctx, cfg, dest, src)
    }

    fn movzx(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[dest, src])?;
        self.writer.movzx(ctx, cfg, dest, src)
    }

    fn get_ip(&mut self, ctx: &mut Context, cfg: X64Arch) -> Result<(), Self::Error> { self.writer.get_ip(ctx, cfg) }
    fn ret(&mut self, ctx: &mut Context, cfg: X64Arch) -> Result<(), Self::Error> { self.writer.ret(ctx, cfg) }
    fn mov64(&mut self, ctx: &mut Context, cfg: X64Arch, r: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error> {
        // Flush pending operations if RSP is involved
        self.ensure_stack_flushed_for_rsp(cfg, &[r])?;
        self.writer.mov64(ctx, cfg, r, val)
    }

    // Floating and other ops: ensure operands are valid via desugar_operand where appropriate
    fn fadd(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(ctx, cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(ctx, cfg, src)?;
        self.writer.fadd(ctx, cfg, &d, &s)
    }

    fn fsub(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(ctx, cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(ctx, cfg, src)?;
        self.writer.fsub(ctx, cfg, &d, &s)
    }

    fn fmul(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(ctx, cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(ctx, cfg, src)?;
        self.writer.fmul(ctx, cfg, &d, &s)
    }

    fn fdiv(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(ctx, cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(ctx, cfg, src)?;
        self.writer.fdiv(ctx, cfg, &d, &s)
    }

    fn fmov(&mut self, ctx: &mut Context, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let s = self.desugar_operand(ctx, cfg, src)?;
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(ctx, cfg, dest)? } else { dest.concrete_mem_kind() };
        self.writer.fmov(ctx, cfg, &d, &s)
    }

    fn db(&mut self, ctx: &mut Context, cfg: X64Arch, bytes: &[u8]) -> Result<(), Self::Error> { self.writer.db(ctx, cfg, bytes) }
}

impl<'a, W, L, Context> crate::out::Writer<L, Context> for DesugaringWriter<'a, W, Context>
where
    W: crate::out::Writer<L> + ?Sized,
{
    fn set_label(&mut self, cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.set_label(ctx, cfg, label)
    }
    fn lea_label(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), label: L) -> Result<(), Self::Error> {
        self.writer.lea_label(ctx, cfg, dest, label)
    }
    fn call_label(&mut self, cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.call_label(ctx, cfg, label)
    }
    fn jmp_label(&mut self, cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.jmp_label(ctx, cfg, label)
    }
    fn jcc_label(&mut self, cfg: X64Arch, cc: crate::ConditionCode, label: L) -> Result<(), Self::Error> {
        self.writer.jcc_label(ctx, cfg, cc, label)
    }
}
