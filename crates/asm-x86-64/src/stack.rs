//! Stack management and optimization for x86-64.
//!
//! This module provides advanced stack management capabilities including:
//! - Offset-based stack data access
//! - Inter-instruction stack optimization
//! - Stack layout tracking and manipulation
//! - Push/pop caching with conflict avoidance

use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

use crate::{
    RegisterClass, X64Arch,
    out::{
        WriterCore,
        arg::{ArgKind, MemArgKind},
    },
};

/// Represents a stack slot with its offset and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StackSlot {
    /// Offset from the stack pointer (positive for downward growth).
    pub offset: i32,
    /// Size of the stack slot in bytes.
    pub size: u32,
    /// Register class for this slot (affects how it's accessed).
    pub reg_class: RegisterClass,
}

/// Stack access pattern for optimization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackAccess {
    /// Push operation (decreases stack pointer).
    Push(Reg),
    /// Pop operation (increases stack pointer).
    Pop(Reg),
    /// Direct memory access at offset.
    Access { offset: i32, size: MemorySize },
}

/// Advanced stack manager with inter-instruction optimization.
///
/// This manager tracks the complete stack layout and enables optimizations
/// across multiple instructions, including offset-based data access and
/// push/pop caching.
pub struct StackManager {
    /// Current stack pointer offset (0 = initial SP, positive = downward growth).
    stack_offset: i32,
    /// Stack of pushed registers/values in push order (index 0 is bottom of stack).
    stack_slots: [StackSlot; 32],
    /// Current number of active stack slots.
    stack_depth: usize,
    /// Pending push/pop operations that can be optimized.
    pending_ops: [Option<StackAccess>; 16],
    /// Number of pending operations.
    pending_count: usize,
}

impl StackManager {
    /// Creates a new stack manager with empty stack.
    pub fn new() -> Self {
        Self {
            stack_offset: 0,
            stack_slots: [StackSlot {
                offset: 0,
                size: 0,
                reg_class: RegisterClass::Gpr,
            }; 32],
            stack_depth: 0,
            pending_ops: [None; 16],
            pending_count: 0,
        }
    }

    /// Allocates a stack slot of the given size and register class.
    /// Returns the offset from the current stack pointer.
    pub fn allocate_slot(&mut self, size: u32, reg_class: RegisterClass) -> i32 {
        let offset = self.stack_offset;
        self.stack_offset += size as i32;

        // Add to stack slots if we have space
        if self.stack_depth < self.stack_slots.len() {
            self.stack_slots[self.stack_depth] = StackSlot {
                offset,
                size,
                reg_class,
            };
            self.stack_depth += 1;
        }

        offset
    }

    /// Deallocates the top stack slot.
    /// Returns the deallocated slot if it existed.
    pub fn deallocate_slot(&mut self) -> Option<StackSlot> {
        if self.stack_depth > 0 {
            self.stack_depth -= 1;
            let slot = self.stack_slots[self.stack_depth];
            self.stack_offset -= slot.size as i32;
            Some(slot)
        } else {
            None
        }
    }

    /// Gets the current stack offset.
    pub fn current_offset(&self) -> i32 {
        self.stack_offset
    }

    /// Creates a memory argument for stack access at the given offset.
    pub fn stack_mem_arg(
        &self,
        offset: i32,
        size: MemorySize,
        reg_class: RegisterClass,
    ) -> MemArgKind<ArgKind> {
        MemArgKind::Mem {
            base: ArgKind::Reg {
                reg: Reg(4),
                size: MemorySize::_64,
            }, // rsp
            offset: None,
            disp: offset as u32,
            size,
            reg_class,
        }
    }

    /// Creates a memory argument for local variable access using frame pointer.
    /// This performs stack offset fixups similar to RISC-V implementation.
    pub fn local_mem_arg(
        &self,
        local: u32,
        size: MemorySize,
        reg_class: RegisterClass,
    ) -> MemArgKind<ArgKind> {
        // Calculate offset from frame pointer: locals are at negative offsets
        // Each local takes 8 bytes, so local N is at -(N+1)*8 from fp
        let offset = -((local as i32 + 1) * 8);
        MemArgKind::Mem {
            base: ArgKind::Reg {
                reg: Reg(5),
                size: MemorySize::_64,
            }, // rbp (frame pointer)
            offset: None,
            disp: offset as u32,
            size,
            reg_class,
        }
    }

    /// Records a pending stack operation for potential optimization.
    pub fn record_operation(&mut self, op: StackAccess) {
        if self.pending_count < self.pending_ops.len() {
            self.pending_ops[self.pending_count] = Some(op);
            self.pending_count += 1;
        }
    }

    /// Optimizes pending stack operations and executes them.
    /// Returns true if any optimizations were applied.
    pub fn optimize_and_execute<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
    ) -> Result<bool, W::Error> {
        if self.pending_count == 0 {
            return Ok(false);
        }

        // Simple optimization: cancel out push/pop pairs for the same register
        let mut optimized = false;
        let mut i = 0;
        while i < self.pending_count.saturating_sub(1) {
            if let (Some(StackAccess::Push(push_reg)), Some(StackAccess::Pop(pop_reg))) =
                (self.pending_ops[i], self.pending_ops[i + 1])
            {
                if push_reg == pop_reg {
                    // Remove both operations - they cancel out
                    self.pending_ops[i] = None;
                    self.pending_ops[i + 1] = None;
                    optimized = true;
                    i += 2; // Skip the next operation
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Execute remaining operations
        for j in 0..self.pending_count {
            if let Some(op) = self.pending_ops[j] {
                match op {
                    StackAccess::Push(reg) => {
                        writer.push(ctx, arch, &reg)?;
                    }
                    StackAccess::Pop(reg) => {
                        writer.pop(ctx, arch, &reg)?;
                    }
                    StackAccess::Access { offset: _, size: _ } => {
                        // Direct access - no stack pointer change needed
                        // This is just recorded for optimization purposes
                    }
                }
            }
        }

        // Clear pending operations
        self.pending_count = 0;
        Ok(optimized)
    }

    /// Performs an optimized push operation.
    pub fn push<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
        reg: &Reg,
    ) -> Result<(), W::Error> {
        self.record_operation(StackAccess::Push(*reg));
        // For now, delegate to writer - in future this could be optimized
        writer.push(ctx, arch, reg)
    }

    /// Performs an optimized pop operation.
    pub fn pop<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
        reg: &Reg,
    ) -> Result<(), W::Error> {
        self.record_operation(StackAccess::Pop(*reg));
        // For now, delegate to writer - in future this could be optimized
        writer.pop(ctx, arch, reg)
    }

    /// Accesses stack data at the given offset with optimization.
    pub fn access_stack<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
        offset: i32,
        size: MemorySize,
        reg_class: RegisterClass,
        dest: &Reg,
    ) -> Result<(), W::Error> {
        self.record_operation(StackAccess::Access { offset, size });

        // Create memory argument for stack access
        let mem_arg = self.stack_mem_arg(offset, size, reg_class);

        // Perform the access
        writer.mov(ctx, arch, dest, &mem_arg)
    }

    /// Accesses a local variable using frame pointer with proper offset fixups.
    /// This mirrors the RISC-V GetLocal functionality.
    pub fn get_local<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
        local: u32,
        size: MemorySize,
        reg_class: RegisterClass,
        dest: &Reg,
    ) -> Result<(), W::Error> {
        // Create memory argument for local access with fixup
        let mem_arg = self.local_mem_arg(local, size, reg_class);

        // Perform the load
        writer.mov(ctx, arch, dest, &mem_arg)
    }

    /// Stores a value to a local variable using frame pointer with proper offset fixups.
    /// This mirrors the RISC-V SetLocal functionality.
    pub fn set_local<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
        local: u32,
        size: MemorySize,
        reg_class: RegisterClass,
        src: &Reg,
    ) -> Result<(), W::Error> {
        // Create memory argument for local access with fixup
        let mem_arg = self.local_mem_arg(local, size, reg_class);

        // Perform the store
        writer.mov(ctx, arch, &mem_arg, src)
    }

    /// Gets the number of pending operations.
    pub fn pending_count(&self) -> usize {
        self.pending_count
    }

    /// Gets the current stack depth (number of allocated slots).
    pub fn stack_depth(&self) -> usize {
        self.stack_depth
    }

    /// Gets a reference to the stack slots.
    pub fn stack_slots(&self) -> &[StackSlot] {
        &self.stack_slots[..self.stack_depth]
    }

    /// Checks if RSP (stack pointer) is used in any pending operations.
    pub fn uses_rsp(&self) -> bool {
        let rsp = Reg(4); // RSP register
        for i in 0..self.pending_count {
            if let Some(op) = self.pending_ops[i] {
                match op {
                    StackAccess::Push(reg) | StackAccess::Pop(reg) => {
                        if reg == rsp {
                            return true;
                        }
                    }
                    StackAccess::Access { .. } => {
                        // Access operations use RSP implicitly for addressing
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Flushes all pending stack operations before an RSP-using instruction.
    /// This ensures the stack is in a consistent state for RSP operations.
    pub fn flush_before_rsp_use<Context, W: WriterCore<Context> + ?Sized>(
        &mut self,
        writer: &mut W,
        ctx: &mut Context,
        arch: X64Arch,
    ) -> Result<(), W::Error> {
        if self.pending_count > 0 {
            self.optimize_and_execute(writer, ctx, arch)?;
        }
        Ok(())
    }

    /// Calculates the adjusted offset for a stack access considering pending operations.
    /// This allows memory accesses to work correctly even with pending stack operations.
    pub fn adjusted_offset(&self, original_offset: i32) -> i32 {
        let mut adjustment = 0;
        for i in 0..self.pending_count {
            if let Some(op) = self.pending_ops[i] {
                match op {
                    StackAccess::Push(_) => {
                        // Each pending push will decrease the stack pointer
                        adjustment -= 8; // Assuming 8-byte pushes for simplicity
                    }
                    StackAccess::Pop(_) => {
                        // Each pending pop will increase the stack pointer
                        adjustment += 8; // Assuming 8-byte pops for simplicity
                    }
                    StackAccess::Access { .. } => {
                        // Access operations don't change the stack pointer
                    }
                }
            }
        }
        original_offset + adjustment
    }

    /// Resets the stack manager to initial state.
    pub fn reset(&mut self) {
        self.stack_offset = 0;
        self.stack_depth = 0;
        self.pending_count = 0;
        for op in &mut self.pending_ops {
            *op = None;
        }
    }
}

impl Default for StackManager {
    fn default() -> Self {
        Self::new()
    }
}
