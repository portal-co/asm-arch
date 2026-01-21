//! x86-64 to AArch64 translation shim (moved out of `portal-solutions-asm-aarch64`).
//!
//! Adapted to live in a separate crate; references types from `portal-solutions-asm-aarch64`.

use portal_solutions_asm_aarch64::out::arg::MemArg;
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


/// Adapter that converts x86-64 MemArg to AArch64 MemArg.
///
/// This type wraps a reference to an x86-64 MemArg and implements the AArch64 MemArg trait,
/// allowing x86-64 arguments to be passed to AArch64 instruction generation functions.
pub struct MemArgAdapter<'a> {
    inner: &'a (dyn X64MemArg + 'a),
    arch: X64Arch,
}

/// Describes how to access APX-extended registers when APX is enabled.
///
/// Variants:
/// - None: no special handling required
/// - RegOffset: register operand must be accessed by loading/storing from `[base + offset_bytes]`
/// - MemBaseOffset: memory operand uses an APX register as its base; replace base with `base` and add `added_disp` to displacement
/// - MemIndex: memory operand uses an APX register as its index; indicates index number and scale to compute an extra byte offset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum APXAccess {
    None,
    RegOffset { base: Reg, offset_bytes: i32 },
    MemBaseOffset { base: Reg, added_disp: i32 },
    MemIndex { base: Reg, index: u32, scale: u32 },
}

impl<'a> MemArgAdapter<'a> {
    /// Creates a new adapter wrapping an x86-64 MemArg and records the x86_64 arch.
    pub fn new(inner: &'a (dyn X64MemArg + 'a), arch: X64Arch) -> Self {
        Self { inner, arch }
    }

    /// Determine whether this argument refers to an APX "late" register or memory
    /// operand that must be accessed via a reserved AArch64 base pointer. Returns
    /// an APXAccess describing how the caller should generate accesses.
    ///
    /// Rules:
    /// - If the operand is a directly-mapped register (r16..=r23), return None so
    ///   caller can use the mapped AArch64 register directly.
    /// - If the operand is a register r>=24 and APX is enabled, return RegOffset
    ///   with base=X28 and offset = (r - 24) * 8 (bytes) so callers load/store from [X28 + offset].
    /// - If the operand is a memory argument and its base is an APX register r>=24,
    ///   return MemBaseOffset with added_disp = (r - 24) * 8 so the base register is replaced
    ///   with X28 and the displacement increased.
    /// - If the operand is a memory argument and its index is an APX register r>=24,
    ///   return MemIndex with index=(r-24) and scale (the scale provided) — callers can
    ///   turn that into additional byte displacement = index * 8 * scale if desired.
    pub fn apx_access(&self) -> APXAccess {
        use portal_solutions_asm_x86_64::out::arg::ArgKind as X64ArgKind;
        use portal_solutions_asm_x86_64::out::arg::MemArgKind as X64MemArgKind;

        // Only applicable when APX is enabled
        if !self.arch.apx {
            return APXAccess::None;
        }

        match self.inner.concrete_mem_kind() {
            X64MemArgKind::NoMem(arg) => {
                match arg {
                    X64ArgKind::Reg { reg, .. } => {
                        let r = reg.0;
                        // Directly mapped registers r16..=r23 should be used directly
                        if (16..=23).contains(&r) {
                            return APXAccess::None;
                        }
                        if r >= 24 {
                            // r24+ accessed via base pointer X28 at offset (r-24)*8
                            let offset = ((r - 24) as i32) * 8;
                            return APXAccess::RegOffset {
                                base: Reg(28),
                                offset_bytes: offset,
                            };
                        }
                        APXAccess::None
                    }
                    _ => APXAccess::None,
                }
            }
            X64MemArgKind::Mem {
                base, offset, disp, ..
            } => {
                // If base is a register that is an APX late register, indicate to replace base with X28
                match base {
                    X64ArgKind::Reg { reg, .. } => {
                        let r = reg.0;
                        if r >= 24 {
                            let added = ((r - 24) as i32) * 8;
                            return APXAccess::MemBaseOffset {
                                base: Reg(28),
                                added_disp: added,
                            };
                        }
                    }
                    _ => {}
                }
                // If index/offset uses an APX register (offset is (reg, scale))
                if let Some((off_arg, scale)) = offset {
                    if let X64ArgKind::Reg { reg, .. } = off_arg {
                        let r = reg.0;
                        if r >= 24 {
                            return APXAccess::MemIndex {
                                base: Reg(28),
                                index: (r - 24) as u32,
                                scale,
                            };
                        }
                    }
                }

                APXAccess::None
            }
            _ => APXAccess::None,
        }
    }
}

impl<'a> portal_solutions_asm_aarch64::out::arg::MemArg for MemArgAdapter<'a> {
    fn mem_kind(
        &self,
        go: &mut (dyn FnMut(portal_solutions_asm_aarch64::out::arg::MemArgKind<&'_ (dyn portal_solutions_asm_aarch64::out::arg::Arg + '_)>) + '_),
    ) {
        use portal_solutions_asm_aarch64::out::arg::MemArgKind as AArch64MemArgKind;
        use portal_solutions_asm_x86_64::out::arg::MemArgKind as X64MemArgKind;

        // Get the x86-64 memory argument kind
        let x64_kind = self.inner.concrete_mem_kind();

        // Convert to AArch64 memory argument kind
        match x64_kind {
            X64MemArgKind::NoMem(arg) => {
                // Direct operand - check for APX register stored in memory
                match self.apx_access() {
                    APXAccess::None => {
                        let aarch64_arg = convert_arg_kind(arg, self.arch);
                        go(AArch64MemArgKind::NoMem(&aarch64_arg));
                    }
                    APXAccess::RegOffset { base, offset_bytes } => {
                        // Represent the APX register as a memory operand at [base + offset_bytes]
                        let aarch64_base = portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                            reg: base,
                            size: MemorySize::_64,
                        };
                        go(AArch64MemArgKind::Mem {
                            base: &aarch64_base,
                            offset: None,
                            disp: offset_bytes,
                            size: MemorySize::_64,
                            reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                            mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::Offset,
                        });
                    }
                    _ => {
                        // Other APX variants won't occur for NoMem here
                        let aarch64_arg = convert_arg_kind(arg, self.arch);
                        go(AArch64MemArgKind::NoMem(&aarch64_arg));
                    }
                }
            }
            X64MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => {
                // Memory reference - convert components, with APX handling
                // Default conversions
                let mut aarch64_base = convert_arg_kind(base, self.arch);
                let mut aarch64_offset =
                    offset.map(|(off, scale)| (convert_arg_kind(off, self.arch), scale));
                let mut aarch64_disp = disp as i32; // Convert u32 to i32
                let aarch64_reg_class = convert_register_class(reg_class);

                // If base is an APX late register, replace base with APX base pointer and adjust displacement
                match base {
                    portal_solutions_asm_x86_64::out::arg::ArgKind::Reg { reg, .. } => {
                        let r = reg.0;
                        if self.arch.apx && r >= 24 {
                            let added = ((r - 24) as i32) * 8;
                            // Replace base with Reg(28)
                            aarch64_base = portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                                reg: Reg(28),
                                size: MemorySize::_64,
                            };
                            aarch64_disp = aarch64_disp.wrapping_add(added);
                        }
                    }
                    _ => {}
                }

                // If offset/index uses an APX register, try to fold it into disp by treating
                // the APX register as stored at base pointer + index_offset and adding that
                // value as an immediate (approximation). Compute extra_disp = (r-24)*8*scale
                if let Some((off_arg, scale)) = offset {
                    if let portal_solutions_asm_x86_64::out::arg::ArgKind::Reg { reg, .. } = off_arg
                    {
                        let r = reg.0;
                        if self.arch.apx && r >= 24 {
                            let extra = ((r - 24) as i32) * 8 * (scale as i32);
                            aarch64_disp = aarch64_disp.wrapping_add(extra);
                            aarch64_offset = None;
                        }
                    }
                }

                // Create the memory argument and pass references to its components
                // x86-64 doesn't have pre/post-index, so always use Offset mode
                match &aarch64_offset {
                    None => {
                        go(AArch64MemArgKind::Mem {
                            base: &aarch64_base,
                            offset: None,
                            disp: aarch64_disp,
                            size,
                            reg_class: aarch64_reg_class,
                            mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::Offset,
                        });
                    }
                    Some((off, scale)) => {
                        go(AArch64MemArgKind::Mem {
                            base: &aarch64_base,
                            offset: Some((off, *scale)),
                            disp: aarch64_disp,
                            size,
                            reg_class: aarch64_reg_class,
                            mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::Offset,
                        });
                    }
                }
            }
            _ => {
                // Handle any future variants with a default behavior
                let aarch64_arg = portal_solutions_asm_aarch64::out::arg::ArgKind::Lit(0);
                go(AArch64MemArgKind::NoMem(&aarch64_arg));
            }
        }
    }
}

/// Converts x86-64 ArgKind to AArch64 ArgKind with register mapping.
fn convert_arg_kind(
    arg: portal_solutions_asm_x86_64::out::arg::ArgKind,
    arch: X64Arch,
) -> portal_solutions_asm_aarch64::out::arg::ArgKind {
    use portal_solutions_asm_aarch64::out::arg::ArgKind as AArch64ArgKind;
    use portal_solutions_asm_x86_64::out::arg::ArgKind as X64ArgKind;

    match arg {
        X64ArgKind::Reg { reg, size } => {
            // Map x86-64 register to AArch64 System V ABI register, taking APX into account
            let aarch64_reg = map_x64_register_to_aarch64(reg, arch);
            AArch64ArgKind::Reg {
                reg: aarch64_reg,
                size,
            }
        }
        X64ArgKind::Lit(val) => AArch64ArgKind::Lit(val),
        _ => AArch64ArgKind::Lit(0), // Handle any future variants
    }
}

/// Converts x86-64 RegisterClass to AArch64 RegisterClass.
fn convert_register_class(
    reg_class: portal_solutions_asm_x86_64::RegisterClass,
) -> portal_solutions_asm_aarch64::RegisterClass {
    use portal_solutions_asm_aarch64::RegisterClass as AArch64RegClass;
    use portal_solutions_asm_x86_64::RegisterClass as X64RegClass;

    match reg_class {
        X64RegClass::Gpr => AArch64RegClass::Gpr,
        X64RegClass::Xmm => AArch64RegClass::Simd,
        _ => AArch64RegClass::Gpr, // Handle any future variants
    }
}

/// Maps x86-64 registers to AArch64 System V ABI registers.
///
/// This function implements the register mapping between x86-64 and AArch64:
/// - RAX (0) → X0 (0) - argument/return register
/// - RCX (1) → X1 (1) - argument register  
/// - RDX (2) → X2 (2) - argument register
/// - RBX (3) → X19 (19) - callee-saved register
/// - RSP (4) → SP (31) - stack pointer
/// - RBP (5) → X29 (29) - frame pointer
/// - RSI (6) → X3 (3) - argument register
/// - RDI (7) → X4 (4) - argument register
/// - R8-R15 → X5-X15, X20-X28
///
/// For SIMD registers, XMM0-XMM15 map directly to V0-V15.
pub fn map_x64_register_to_aarch64(reg: Reg, arch: X64Arch) -> Reg {
    // When APX is enabled we provide mappings for r16..r23 into x20..x27 where possible.
    // Remaining higher APX registers (r24+) are accessed via a reserved base pointer (X28).
    if arch.apx {
        let r = reg.0;
        // Map r16..=23 -> x20..=27
        if (16..=23).contains(&r) {
            return Reg(20 + (r - 16));
        }
        // For r24+ reserve X28 as base pointer for later APX registers
        if r >= 24 {
            return Reg(28);
        }
    }

    match reg.0 {
        // Map x86-64 GPRs to AArch64 System V ABI registers
        0 => Reg(0),   // RAX → X0
        1 => Reg(1),   // RCX → X1
        2 => Reg(2),   // RDX → X2
        3 => Reg(19),  // RBX → X19 (callee-saved)
        4 => Reg(31),  // RSP → SP
        5 => Reg(29),  // RBP → X29 (frame pointer)
        6 => Reg(3),   // RSI → X3
        7 => Reg(4),   // RDI → X4
        8 => Reg(5),   // R8 → X5
        9 => Reg(6),   // R9 → X6
        10 => Reg(7),  // R10 → X7
        11 => Reg(8),  // R11 → X8
        12 => Reg(9),  // R12 → X9
        13 => Reg(10), // R13 → X10
        14 => Reg(11), // R14 → X11
        15 => Reg(12), // R15 → X12
        // For higher registers or SIMD, pass through (XMM maps to V directly)
        n => Reg(n),
    }
}

/// Extension trait for memory access helpers on AArch64 Writer.
///
/// This trait provides helper methods for handling memory operations,
/// automatically detecting whether operands are registers or memory
/// and emitting appropriate LDR/STR instructions.
pub trait WriterShimExt<Context>: portal_solutions_asm_aarch64::out::Writer<ShimLabel, Context> {
    /// Loads a value to a register, using LDR if source is memory.
    ///
    /// If `src` is already a register, this performs a MOV.
    /// If `src` is a memory location, this performs an LDR.
    fn load_to_reg(
        &mut self,
        ctx: &mut Context,
        cfg: portal_solutions_asm_aarch64::AArch64Arch,
        dest: &Reg,
        src: &(dyn portal_solutions_asm_aarch64::out::arg::MemArg + '_),
    ) -> Result<(), <Self as portal_solutions_asm_aarch64::out::WriterCore<Context>>::Error> {
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let src_kind = src.concrete_mem_kind();
        match src_kind {
            MemArgKind::NoMem(_) => {
                // Source is a register or immediate, use MOV
                self.mov(ctx, cfg, dest, src)
            }
            MemArgKind::Mem { .. } => {
                // Source is memory, use LDR
                self.ldr(ctx, cfg, dest, src)
            }
              _ => todo!()
        }
    }

    /// Stores a value from a register, using STR if destination is memory.
    ///
    /// If `dest` is already a register, this performs a MOV.
    /// If `dest` is a memory location, this performs a STR.
    fn store_from_reg(
        &mut self,
        ctx: &mut Context,
        cfg: portal_solutions_asm_aarch64::AArch64Arch,
        dest: &(dyn portal_solutions_asm_aarch64::out::arg::MemArg + '_),
        src: &Reg,
    ) -> Result<(), <Self as portal_solutions_asm_aarch64::out::WriterCore<Context>>::Error> {
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let dest_kind = dest.concrete_mem_kind();
        match dest_kind {
            MemArgKind::NoMem(_) => {
                // Destination is a register, use MOV
                self.mov(ctx, cfg, dest, src)
            }
            MemArgKind::Mem { .. } => {
                // Destination is memory, use STR
                self.str(ctx, cfg, src, dest)
            },
            _ => todo!()
        }
    }
}

// Blanket implementation for all types that implement Writer<ShimLabel>
impl<Context, W: portal_solutions_asm_aarch64::out::Writer<ShimLabel, Context>> WriterShimExt<Context> for W {}

/// Helper macro to handle two-operand instructions with memory operands.
///
/// Pattern: INSTR a, b where a = INSTR(a, b)
/// Handles all combinations of register/memory operands using LDR/STR as needed.
macro_rules! handle_two_operand_instr {
    ($self:expr, $ctx:expr, $a:expr, $b:expr, $instr:ident, $cfg:expr) => {{
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let a_adapter = MemArgAdapter::new($a, $cfg);
        let b_adapter = MemArgAdapter::new($b, $cfg);

        let a_kind = a_adapter.concrete_mem_kind();
        let b_kind = b_adapter.concrete_mem_kind();

        match (a_kind, b_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both are registers/immediates - direct operation
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &a_adapter, &a_adapter, &b_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // a is memory, b is register - LDR a, INSTR, STR a
                let temp = Reg(16); // x16
                $self.load_memarg_into_temp($ctx, &a_adapter, &temp)?;
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &temp, &temp, &b_adapter)?;
                $self.inner.str($ctx, $self.aarch64_cfg, &temp, &a_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // a is register, b is memory - LDR b into temp, then INSTR
                let temp = Reg(17); // x17
                $self.load_memarg_into_temp($ctx, &b_adapter, &temp)?;
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &a_adapter, &a_adapter, &temp)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Both are memory - LDR both, INSTR, STR result
                let temp_a = Reg(16); // x16
                let temp_b = Reg(17); // x17
                $self.load_memarg_into_temp($ctx, &a_adapter, &temp_a)?;
                $self.load_memarg_into_temp($ctx, &b_adapter, &temp_b)?;
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &temp_a, &temp_a, &temp_b)?;
                $self
                    .inner
                    .str($ctx, $self.aarch64_cfg, &temp_a, &a_adapter)
            }
              _ => todo!()
        }
    }};
}

/// Helper for two-operand instructions where result overwrites first operand (like SUB in x86).
/// For SUB, the AArch64 instruction is: SUB a, a, b (but only takes 2 args in trait)
macro_rules! handle_two_operand_instr_2arg {
    ($self:expr, $ctx:expr, $a:expr, $b:expr, $instr:ident, $cfg:expr) => {{
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let a_adapter = MemArgAdapter::new($a, $cfg);
        let b_adapter = MemArgAdapter::new($b, $cfg);

        let a_kind = a_adapter.concrete_mem_kind();
        let b_kind = b_adapter.concrete_mem_kind();

        match (a_kind, b_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both are registers/immediates - direct operation with dest=a
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &a_adapter, &a_adapter, &b_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // a is memory, b is register - LDR a, INSTR, STR a
                let temp = Reg(16); // x16
                $self.load_memarg_into_temp($ctx, &a_adapter, &temp)?;
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &temp, &temp, &b_adapter)?;
                $self.inner.str($ctx, $self.aarch64_cfg, &temp, &a_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // a is register, b is memory - LDR b into temp, then INSTR with dest=a
                let temp = Reg(17); // x17
                $self.load_memarg_into_temp($ctx, &b_adapter, &temp)?;
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &a_adapter, &a_adapter, &temp)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Both are memory - LDR both, INSTR, STR result
                let temp_a = Reg(16); // x16
                let temp_b = Reg(17); // x17
                $self.load_memarg_into_temp($ctx, &a_adapter, &temp_a)?;
                $self.load_memarg_into_temp($ctx, &b_adapter, &temp_b)?;
                $self
                    .inner
                    .$instr($ctx, $self.aarch64_cfg, &temp_a, &temp_a, &temp_b)?;
                $self
                    .inner
                    .str($ctx, $self.aarch64_cfg, &temp_a, &a_adapter)
            }
            _ => todo!()
        }
    }};
}

/// Wrapper that translates x86-64 instructions to AArch64.
///
/// This type wraps an AArch64 writer and implements the x86-64 WriterCore trait,
/// translating each x86-64 instruction to equivalent AArch64 instructions.
///
/// # Shim System
///
/// The shim maintains a stack-based calling convention compatible with x86-64:
/// - CALL instructions use label-based shims that push LR and branch to the target
/// - RET instructions directly emit code that pops the return address from the stack, then returns
///
/// This ensures x86-64 semantics where the return address is stored on the stack rather than in LR.
pub struct X64ToAArch64Shim<W> {
    /// The underlying AArch64 writer.
    pub inner: W,
    /// AArch64 architecture configuration.
    pub aarch64_cfg: portal_solutions_asm_aarch64::AArch64Arch,
    /// Counter for generating unique shim labels.
    shim_counter: usize,
}

impl<W> X64ToAArch64Shim<W> {
    /// Creates a new shim wrapping the given AArch64 writer.
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            aarch64_cfg: Default::default(),
            shim_counter: 0,
        }
    }

    /// Creates a new shim with a specific AArch64 configuration.
    pub fn with_config(inner: W, aarch64_cfg: portal_solutions_asm_aarch64::AArch64Arch) -> Self {
        Self {
            inner,
            aarch64_cfg,
            shim_counter: 0,
        }
    }

    /// Generates a unique shim label.
    fn next_shim_label(&mut self) -> ShimLabel {
        let label = ShimLabel(self.shim_counter);
        self.shim_counter += 1;
        label
    }

    /// Load a value from a possibly-APX memory argument into `dest`.
    ///
    /// If the given adapter references a memory operand whose index is an APX register,
    /// this will first load the APX index value from the APX backing store ([X28 + slot])
    /// into a temporary register and then perform the actual load using that temp as the
    /// scaled index. Otherwise, delegates to the underlying writer's LDR.
    fn load_memarg_into_temp<Context>(
        &mut self,
        ctx: &mut Context,
        adapter: &MemArgAdapter<'_>,
        dest: &Reg,
    ) -> Result<(), W::Error>
    where
        W: portal_solutions_asm_aarch64::out::Writer<ShimLabel, Context>,
    {
        use portal_solutions_asm_x86_64::out::arg::MemArgKind as X64MemArgKind;

        match adapter.apx_access() {
            APXAccess::MemIndex {
                base: _base,
                index,
                scale,
            } => {
                // Load the APX index value from APX backing store into a temp register
                let slot_offset = (index as i32) * 8; // each slot is 8 bytes
                let mem_slot = portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                    base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                        reg: Reg(28),
                        size: MemorySize::_64,
                    },
                    offset: None,
                    disp: slot_offset,
                    size: MemorySize::_64,
                    reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                    mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::Offset,
                };
                let temp_idx = Reg(18);
                // Load index value from APX backing store
                self.inner
                    .ldr(ctx, self.aarch64_cfg, &temp_idx, &mem_slot)?;

                // Rebuild the memory argument using temp_idx as the index register
                // Extract original components to get base, disp, size and reg_class
                if let X64MemArgKind::Mem {
                    base: orig_base,
                    offset: _orig_offset,
                    disp,
                    size,
                    reg_class,
                } = adapter.inner.concrete_mem_kind()
                {
                    let aarch64_base = convert_arg_kind(orig_base, adapter.arch);
                    let aarch64_disp = disp as i32;
                    let off_arg = portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                        reg: temp_idx,
                        size: MemorySize::_64,
                    };
                    let mem_arg = portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                        base: aarch64_base,
                        offset: Some((off_arg, scale)),
                        disp: aarch64_disp,
                        size,
                        reg_class: convert_register_class(reg_class),
                        mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::Offset,
                    };
                    // Perform final load into dest using reconstructed memory operand
                    self.inner.ldr(ctx, self.aarch64_cfg, dest, &mem_arg)
                } else {
                    // should not happen: adapter indicated MemIndex but not a Mem
                    self.inner.ldr(ctx, self.aarch64_cfg, dest, adapter)
                }
            }
            _ => {
                // Default: delegate to underlying writer
                self.inner.ldr(ctx, self.aarch64_cfg, dest, adapter)
            }
        }
    }
}

/// Translates x86-64 condition codes to AArch64 condition codes.
///
/// # Translation Table
///
/// | x86-64 | AArch64 | Notes |
/// |--------|---------|-------|
/// | E/Z    | EQ      | Equal / Zero |
/// | NE/NZ  | NE      | Not equal / Not zero |
/// | B/C    | LO      | Unsigned less / Carry |
/// | NB/NC  | HS      | Unsigned >= / No carry |
/// | A      | HI      | Unsigned greater |
/// | NA     | LS      | Unsigned <= |
/// | L      | LT      | Signed less |
/// | NL     | GE      | Signed >= |
/// | G      | GT      | Signed greater |
/// | NG     | LE      | Signed <= |
/// | O      | VS      | Overflow |
/// | NO     | VC      | No overflow |
/// | S      | MI      | Sign / Negative |
/// | NS     | PL      | No sign / Positive |
/// | P/PE   | AL      | Parity even (no equivalent) |
/// | NP/PO  | AL      | Parity odd (no equivalent) |
pub fn translate_condition(cc: X64ConditionCode) -> portal_solutions_asm_aarch64::ConditionCode {
    match cc {
        X64ConditionCode::E => portal_solutions_asm_aarch64::ConditionCode::EQ, // Equal
        X64ConditionCode::NE => portal_solutions_asm_aarch64::ConditionCode::NE, // Not equal
        X64ConditionCode::B => portal_solutions_asm_aarch64::ConditionCode::LO, // Unsigned less (below)
        X64ConditionCode::NB => portal_solutions_asm_aarch64::ConditionCode::HS, // Unsigned greater or equal (not below)
        X64ConditionCode::A => portal_solutions_asm_aarch64::ConditionCode::HI, // Unsigned greater (above)
        X64ConditionCode::NA => portal_solutions_asm_aarch64::ConditionCode::LS, // Unsigned less or equal (not above)
        X64ConditionCode::L => portal_solutions_asm_aarch64::ConditionCode::LT, // Signed less
        X64ConditionCode::NL => portal_solutions_asm_aarch64::ConditionCode::GE, // Signed greater or equal
        X64ConditionCode::G => portal_solutions_asm_aarch64::ConditionCode::GT, // Signed greater
        X64ConditionCode::NG => portal_solutions_asm_aarch64::ConditionCode::LE, // Signed less or equal
        X64ConditionCode::O => portal_solutions_asm_aarch64::ConditionCode::VS, // Overflow
        X64ConditionCode::NO => portal_solutions_asm_aarch64::ConditionCode::VC, // No overflow
        X64ConditionCode::S => portal_solutions_asm_aarch64::ConditionCode::MI, // Sign (negative)
        X64ConditionCode::NS => portal_solutions_asm_aarch64::ConditionCode::PL, // No sign (positive)
        X64ConditionCode::P => portal_solutions_asm_aarch64::ConditionCode::AL, // Parity - no direct equivalent, use always
        X64ConditionCode::NP => portal_solutions_asm_aarch64::ConditionCode::AL, // No parity - no direct equivalent
        _ => portal_solutions_asm_aarch64::ConditionCode::AL,                   // Catch-all for any future variants
    }
}

impl<Context, W: portal_solutions_asm_aarch64::out::Writer<ShimLabel, Context>> X64WriterCore<Context>
    for X64ToAArch64Shim<W>
{
    type Error = W::Error;

    fn hlt(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 HLT -> AArch64 BRK
        self.inner.brk(ctx, self.aarch64_cfg, 0)
    }

    fn xchg(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 XCHG doesn't have a direct AArch64 equivalent
        // We need a temporary register. Use x16 (IP0) which is caller-saved
        // PERFORMANCE: Uses 3 MOV instructions instead of 1 XCHG
        let temp = Reg(16);
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner
            .mov(ctx, self.aarch64_cfg, &temp, &dest_adapter)?;
        self.inner
            .mov(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)?;
        self.inner.mov(ctx, self.aarch64_cfg, &src_adapter, &temp)?;
        Ok(())
    }

    fn mov(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,

        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOV -> AArch64 MOV/LDR/STR depending on operands
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);

        let dest_kind = dest_adapter.concrete_mem_kind();
        let src_kind = src_adapter.concrete_mem_kind();

        match (dest_kind, src_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Register to register or immediate to register - use MOV
                self.inner
                    .mov(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // Memory to register - use LDR
                self.inner
                    .ldr(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // Register to memory - use STR
                self.inner
                    .str(ctx, self.aarch64_cfg, &src_adapter, &dest_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Memory to memory - need temporary register
                // Use x16 (IP0) as temporary
                let temp = Reg(16);
                self.load_memarg_into_temp(ctx, &src_adapter, &temp)?;
                self.inner.str(ctx, self.aarch64_cfg, &temp, &dest_adapter)
            },
            _ => todo!()
        }
    }

    fn sub(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SUB a, b (a = a - b) -> AArch64 SUB a, a, b
        // Handle memory operands with LDR/STR
        handle_two_operand_instr_2arg!(self, ctx, a, b, sub, _cfg)
    }

    fn add(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 ADD a, b (a = a + b) -> AArch64 ADD a, a, b
        // Handle memory operands with LDR/STR
        handle_two_operand_instr_2arg!(self, ctx, a, b, add, _cfg)
    }

    fn movsx(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOVSX -> AArch64 SXTB/SXTH/SXTW (handle memory operands)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        let src_kind = src_adapter.concrete_mem_kind();

        match src_kind {
            MemArgKind::NoMem(_) => {
                // Source is register - direct SXT, then store if needed
                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner
                            .sxt(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)
                    }
                    MemArgKind::Mem { .. } => {
                        let temp = Reg(16); // x16
                        self.inner.sxt(ctx, self.aarch64_cfg, &temp, &src_adapter)?;
                        self.inner.str(ctx, self.aarch64_cfg, &temp, &dest_adapter)
                    }
                    _ => todo!()
                }
            }
            MemArgKind::Mem { .. } => {
                // Source is memory - LDR, SXT, store if needed
                let temp = Reg(16); // x16
                self.load_memarg_into_temp(ctx, &src_adapter, &temp)?;
                let temp2 = Reg(17); // x17 for result
                self.inner.sxt(ctx, self.aarch64_cfg, &temp2, &temp)?;

                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner.mov(ctx, self.aarch64_cfg, &dest_adapter, &temp2)
                    }
                    MemArgKind::Mem { .. } => {
                        self.inner.str(ctx, self.aarch64_cfg, &temp2, &dest_adapter)
                    }
                    _ => todo!()
                }
            }
            _ => todo!()
        }
    }

    fn movzx(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOVZX -> AArch64 UXTB/UXTH (handle memory operands)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        let src_kind = src_adapter.concrete_mem_kind();

        match src_kind {
            MemArgKind::NoMem(_) => {
                // Source is register - direct UXT, then store if needed
                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner
                            .uxt(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)
                    }
                    MemArgKind::Mem { .. } => {
                        let temp = Reg(16); // x16
                        self.inner.uxt(ctx, self.aarch64_cfg, &temp, &src_adapter)?;
                        self.inner.str(ctx, self.aarch64_cfg, &temp, &dest_adapter)
                    }
                    _ => todo!()
                }
            }
            MemArgKind::Mem { .. } => {
                // Source is memory - LDR, UXT, store if needed
                let temp = Reg(16); // x16
                self.load_memarg_into_temp(ctx, &src_adapter, &temp)?;
                let temp2 = Reg(17); // x17 for result
                self.inner.uxt(ctx, self.aarch64_cfg, &temp2, &temp)?;

                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner.mov(ctx, self.aarch64_cfg, &dest_adapter, &temp2)
                    }
                    MemArgKind::Mem { .. } => {
                        self.inner.str(ctx, self.aarch64_cfg, &temp2, &dest_adapter)
                    }
                    _ => todo!()
                }
            }
            _ => todo!()
        }
    }

    fn push(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 PUSH -> AArch64 STR with pre-indexed addressing
        // [sp, #-8]! means: sp = sp - 8, then str to [sp]
        let sp = Reg(31); // SP
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.str(
            ctx,
            self.aarch64_cfg,
            &op_adapter,
            &portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                    reg: sp,
                    size: MemorySize::_64,
                },
                offset: None,
                disp: -8,
                size: MemorySize::_64,
                reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::PreIndex,
            },
        )
    }

    fn pop(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 POP -> AArch64 LDR with post-indexed addressing
        // [sp], #8 means: ldr from [sp], then sp = sp + 8
        let sp = Reg(31); // SP
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.ldr(
            ctx,
            self.aarch64_cfg,
            &op_adapter,
            &portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                    reg: sp,
                    size: MemorySize::_64,
                },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::PostIndex,
            },
        )
    }

    fn pushf(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 PUSHF -> AArch64 MRS NZCV + STR with pre-indexed addressing
        // Store NZCV flags using MRS
        let temp = Reg(16); // x16
        let sp = Reg(31);
        // Read NZCV flags into temp register
        self.inner.mrs_nzcv(ctx, self.aarch64_cfg, &temp)?;
        // Store flags to stack with pre-decrement: [sp, #-8]!
        self.inner.str(
            ctx,
            self.aarch64_cfg,
            &temp,
            &portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                    reg: sp,
                    size: MemorySize::_64,
                },
                offset: None,
                disp: -8,
                size: MemorySize::_64,
                reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::PreIndex,
            },
        )
    }

    fn popf(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 POPF -> AArch64 LDR with post-indexed addressing + MSR NZCV
        let temp = Reg(16); // x16
        let sp = Reg(31);
        // Load flags from stack with post-increment: [sp], #8
        self.inner.ldr(
            ctx,
            self.aarch64_cfg,
            &temp,
            &portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                    reg: sp,
                    size: MemorySize::_64,
                },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::PostIndex,
            },
        )?;
        // Write flags back to NZCV
        self.inner.msr_nzcv(ctx, self.aarch64_cfg, &temp)
    }

    fn call(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 CALL -> AArch64 call shim using labels
        // Strategy: Branch to a shim that pushes LR and branches to the target
        // The shim is emitted inline with a jump over it to ensure correctness

        let sp = Reg(31); // SP
        let lr = Reg(30); // LR (x30)

        // Generate unique labels
        let shim_label = self.next_shim_label();
        let skip_label = self.next_shim_label();

        // Jump over the shim to skip_label
        self.inner.b_label(ctx, self.aarch64_cfg, skip_label)?;

        // Emit the call shim inline
        self.inner.set_label(ctx, self.aarch64_cfg, shim_label)?;

        // Push LR onto stack with pre-indexed addressing: [sp, #-8]!
        self.inner.str(
            ctx,
            self.aarch64_cfg,
            &lr,
            &portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                    reg: sp,
                    size: MemorySize::_64,
                },
                offset: None,
                disp: -8,
                size: MemorySize::_64,
                reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::PreIndex,
            },
        )?;

        // Branch to the target
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.b(ctx, self.aarch64_cfg, &op_adapter)?;

        // Set skip label (execution continues here)
        self.inner.set_label(ctx, self.aarch64_cfg, skip_label)?;

        // Branch and link to the shim
        self.inner.bl_label(ctx, self.aarch64_cfg, shim_label)?;

        Ok(())
    }

    fn jmp(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 JMP -> AArch64 B or BR
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.b(ctx, self.aarch64_cfg, &op_adapter)
    }

    fn cmp(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 CMP -> AArch64 CMP (handle memory operands)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);

        let a_kind = a_adapter.concrete_mem_kind();
        let b_kind = b_adapter.concrete_mem_kind();

        match (a_kind, b_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both are registers/immediates - direct CMP
                self.inner
                    .cmp(ctx, self.aarch64_cfg, &a_adapter, &b_adapter)
            }
            (MemArgKind::Mem { .. }, _) => {
                // a is memory - LDR into temp, then CMP
                let temp = Reg(16); // x16
                self.load_memarg_into_temp(ctx, &a_adapter, &temp)?;
                if matches!(b_kind, MemArgKind::Mem { .. }) {
                    let temp_b = Reg(17); // x17
                    self.inner.ldr(ctx, self.aarch64_cfg, &temp_b, &b_adapter)?;
                    self.inner.cmp(ctx, self.aarch64_cfg, &temp, &temp_b)
                } else {
                    self.inner.cmp(ctx, self.aarch64_cfg, &temp, &b_adapter)
                }
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // b is memory - LDR into temp, then CMP
                let temp = Reg(17); // x17
                self.load_memarg_into_temp(ctx, &b_adapter, &temp)?;
                self.inner.cmp(ctx, self.aarch64_cfg, &a_adapter, &temp)
            }
            _ => todo!()
        }
    }

    fn cmp0(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 CMP op, 0 -> AArch64 CMP op, #0 (handle memory operands)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let op_adapter = MemArgAdapter::new(op, _cfg);
        let op_kind = op_adapter.concrete_mem_kind();

        match op_kind {
            MemArgKind::NoMem(_) => {
                // Register/immediate - direct CMP
                self.inner.cmp(ctx, self.aarch64_cfg, &op_adapter, &0u64)
            }
            MemArgKind::Mem { .. } => {
                // Memory - LDR into temp, then CMP
                let temp = Reg(16); // x16
                self.load_memarg_into_temp(ctx, &op_adapter, &temp)?;
                self.inner.cmp(ctx, self.aarch64_cfg, &temp, &0u64)
            }
            _ => todo!()
        }
    }

    fn cmovcc64(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        cond: X64ConditionCode,
        op: &(dyn X64MemArg + '_),
        val: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 CMOVcc -> AArch64 CSEL (handle memory operands)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let aarch64_cond = translate_condition(cond);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let val_adapter = MemArgAdapter::new(val, _cfg);

        let op_kind = op_adapter.concrete_mem_kind();
        let val_kind = val_adapter.concrete_mem_kind();

        match (op_kind, val_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both registers - direct CSEL
                self.inner.csel(
                    ctx,
                    self.aarch64_cfg,
                    aarch64_cond,
                    &op_adapter,
                    &val_adapter,
                    &op_adapter,
                )
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // op is memory - LDR, CSEL, STR
                let temp = Reg(16); // x16
                self.load_memarg_into_temp(ctx, &op_adapter, &temp)?;
                self.inner.csel(
                    ctx,
                    self.aarch64_cfg,
                    aarch64_cond,
                    &temp,
                    &val_adapter,
                    &temp,
                )?;
                self.inner.str(ctx, self.aarch64_cfg, &temp, &op_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // val is memory - LDR val, then CSEL
                let temp = Reg(17); // x17
                self.load_memarg_into_temp(ctx, &val_adapter, &temp)?;
                self.inner.csel(
                    ctx,
                    self.aarch64_cfg,
                    aarch64_cond,
                    &op_adapter,
                    &temp,
                    &op_adapter,
                )
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Both memory - LDR both, CSEL, STR
                let temp_op = Reg(16); // x16
                let temp_val = Reg(17); // x17
                self.load_memarg_into_temp(ctx, &op_adapter, &temp_op)?;
                self.load_memarg_into_temp(ctx, &val_adapter, &temp_val)?;
                self.inner.csel(
                    ctx,
                    self.aarch64_cfg,
                    aarch64_cond,
                    &temp_op,
                    &temp_val,
                    &temp_op,
                )?;
                self.inner.str(ctx, self.aarch64_cfg, &temp_op, &op_adapter)
            }
            _ => todo!()
        }
    }

    fn jcc(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        cond: X64ConditionCode,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 Jcc -> AArch64 B.cond
        let aarch64_cond = translate_condition(cond);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner
            .bcond(ctx, self.aarch64_cfg, aarch64_cond, &op_adapter)
    }

    fn u32(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 AND op, 0xffffffff -> AArch64 AND op, op, #0xffffffff (handle memory)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let op_adapter = MemArgAdapter::new(op, _cfg);
        let op_kind = op_adapter.concrete_mem_kind();
        let temp = Reg(16); // Use temp register for immediate
        let temp2 = Reg(17); // For result if memory

        self.inner
            .mov_imm(ctx, self.aarch64_cfg, &temp, 0xffffffff)?;

        match op_kind {
            MemArgKind::NoMem(_) => {
                // Register - direct AND
                self.inner
                    .and(ctx, self.aarch64_cfg, &op_adapter, &op_adapter, &temp)
            }
            MemArgKind::Mem { .. } => {
                // Memory - LDR, AND, STR
                self.load_memarg_into_temp(ctx, &op_adapter, &temp2)?;
                self.inner
                    .and(ctx, self.aarch64_cfg, &temp2, &temp2, &temp)?;
                self.inner.str(ctx, self.aarch64_cfg, &temp2, &op_adapter)
            }
            _ => todo!()
        }
    }

    fn not(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 NOT -> AArch64 MVN (handle memory operands)
        use portal_solutions_asm_aarch64::out::arg::MemArgKind;

        let op_adapter = MemArgAdapter::new(op, _cfg);
        let op_kind = op_adapter.concrete_mem_kind();

        match op_kind {
            MemArgKind::NoMem(_) => {
                // Register - direct MVN
                self.inner
                    .mvn(ctx, self.aarch64_cfg, &op_adapter, &op_adapter)
            }
            MemArgKind::Mem { .. } => {
                // Memory - LDR, MVN, STR
                let temp = Reg(16); // x16
                self.load_memarg_into_temp(ctx, &op_adapter, &temp)?;
                self.inner.mvn(ctx, self.aarch64_cfg, &temp, &temp)?;
                self.inner.str(ctx, self.aarch64_cfg, &temp, &op_adapter)
            }
            _ => todo!()
        }
    }

    fn lea(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 LEA -> AArch64 ADD/ADR (depending on context)
        // For simplicity, use ADR for now
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner
            .adr(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)
    }

    fn get_ip(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 get IP (typically via CALL trick) -> AArch64 ADR
        // PERFORMANCE: Different approach than x86-64
        let pc_reg = Reg(30); // LR (link register)
        self.inner.adr(ctx, self.aarch64_cfg, &pc_reg, &0u64)
    }

    fn ret(&mut self, ctx: &mut Context, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 RET -> AArch64 ret shim (inline, no jump)
        // Directly emit: pop return address from stack, then return

        let sp = Reg(31); // SP
        let lr = Reg(30); // LR (x30)

        // Pop return address from stack with post-indexed addressing: [sp], #8
        self.inner.ldr(
            ctx,
            self.aarch64_cfg,
            &lr,
            &portal_solutions_asm_aarch64::out::arg::MemArgKind::Mem {
                base: portal_solutions_asm_aarch64::out::arg::ArgKind::Reg {
                    reg: sp,
                    size: MemorySize::_64,
                },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: portal_solutions_asm_aarch64::RegisterClass::Gpr,
                mode: portal_solutions_asm_aarch64::out::arg::AddressingMode::PostIndex,
            },
        )?;

        // Return
        self.inner.ret(ctx, self.aarch64_cfg)
    }

    fn mov64(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        r: &(dyn X64MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        // x86-64 MOV r, imm64 -> AArch64 MOVZ/MOVK sequence
        let r_adapter = MemArgAdapter::new(r, _cfg);
        self.inner.mov_imm(ctx, self.aarch64_cfg, &r_adapter, val)
    }

    fn mul(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MUL a, b -> AArch64 MUL a, a, b
        handle_two_operand_instr!(self, ctx, a, b, mul, _cfg)
    }

    fn div(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 DIV a, b -> AArch64 UDIV a, a, b
        handle_two_operand_instr!(self, ctx, a, b, udiv, _cfg)
    }

    fn idiv(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 IDIV a, b -> AArch64 SDIV a, a, b
        handle_two_operand_instr!(self, ctx, a, b, sdiv, _cfg)
    }

    fn and(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 AND a, b -> AArch64 AND a, a, b
        handle_two_operand_instr!(self, ctx, a, b, and, _cfg)
    }

    fn or(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 OR a, b -> AArch64 ORR a, a, b
        handle_two_operand_instr!(self, ctx, a, b, orr, _cfg)
    }

    fn eor(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 XOR a, b -> AArch64 EOR a, a, b
        handle_two_operand_instr!(self, ctx, a, b, eor, _cfg)
    }

    fn shl(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SHL a, b -> AArch64 LSL a, a, b
        handle_two_operand_instr!(self, ctx, a, b, lsl, _cfg)
    }

    fn shr(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SHR a, b -> AArch64 LSR a, a, b
        handle_two_operand_instr!(self, ctx, a, b, lsr, _cfg)
    }

    fn fadd(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 ADDSD -> AArch64 FADD
        handle_two_operand_instr!(self, ctx, dest, src, fadd, _cfg)
    }

    fn fsub(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SUBSD -> AArch64 FSUB
        handle_two_operand_instr!(self, ctx, dest, src, fsub, _cfg)
    }

    fn fmul(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MULSD -> AArch64 FMUL
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fmul(
            ctx,
            self.aarch64_cfg,
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
        // x86-64 DIVSD -> AArch64 FDIV
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fdiv(
            ctx,
            self.aarch64_cfg,
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
        // x86-64 MOVSD -> AArch64 FMOV
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner
            .fmov(ctx, self.aarch64_cfg, &dest_adapter, &src_adapter)
    }
}

impl<W: portal_solutions_asm_aarch64::out::Writer<ShimLabel, Context>, L, Context> X64Writer<L, Context>
    for X64ToAArch64Shim<W>
where
    W: portal_solutions_asm_aarch64::out::Writer<L, Context>,
{
    fn set_label(&mut self, ctx: &mut Context, _cfg: X64Arch, s: L) -> Result<(), Self::Error> {
        self.inner.set_label(ctx, self.aarch64_cfg, s)
    }

    fn lea_label(
        &mut self,
        ctx: &mut Context,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        self.inner
            .adr_label(ctx, self.aarch64_cfg, &dest_adapter, label)
    }
}

/// Instruction translation guide.
///
/// Documents how x86-64 instructions map to AArch64, including performance notes.
pub mod translation_guide {
    //! # Instruction Translation Reference
    //!
    //! ## Direct Translations (1:1 mapping)
    //! - `MOV` → `MOV` (register-to-register)
    //! - `MOV` → `LDR/STR` (memory operations)
    //! - `ADD` → `ADD`
    //! - `SUB` → `SUB`
    //! - `CMP` → `CMP`
    //! - `RET` → `RET`
    //! - `CALL` → `BL` (direct) / `BLR` (indirect)
    //! - `JMP` → `B` (direct) / `BR` (indirect)
    //! - `Jcc` → `B.cond`
    //! - `MUL` → `MUL`
    //! - `DIV` → `UDIV`
    //! - `IDIV` → `SDIV`
    //! - `AND` → `AND`
    //! - `OR` → `ORR`
    //! - `XOR` → `EOR`
    //! - `SHL` → `LSL`
    //! - `SHR` → `LSR`
    //! - `NOT` → `MVN`
    //! - `MOVSX` → `SXTB/SXTH/SXTW`
    //! - `MOVZX` → `UXTB/UXTH`
    //! - `CMOVcc` → `CSEL`
    //! - `ADDSD` → `FADD`
    //! - `SUBSD` → `FSUB`
    //! - `MULSD` → `FMUL`
    //! - `DIVSD` → `FDIV`
    //! - `MOVSD` → `FMOV`
    //!
    //! ## Complex Translations (requires multiple instructions)
    //! - `XCHG a, b` → `MOV temp, a; MOV a, b; MOV b, temp` (3 instructions)
    //! - `PUSH op` → `SUB sp, sp, #8; STR op, [sp]` (2 instructions)
    //! - `POP op` → `LDR op, [sp]; ADD sp, sp, #8` (2 instructions)
    //! - `PUSHF` → `MRS temp, NZCV; SUB sp, sp, #8; STR temp, [sp]` (3 instructions)
    //! - `POPF` → `LDR temp, [sp]; ADD sp, sp, #8; MSR NZCV, temp` (3 instructions)
    //! - `LEA` → `ADR` or `ADD` (depending on addressing mode)
    //! - `MOV r, imm64` → `MOVZ/MOVK` sequence (1-4 instructions)
    //!
    //! ## Approximations (behavior differs)
    //! - Parity flag conditions (`P`/`NP`) → Always true (AArch64 has no parity flag)
    //! - `XCHG` → Not atomic without explicit barriers
    //!
    //! ## Register Mapping
    //! - x86-64: RAX-RDI (0-7), R8-R15 (8-15)
    //! - AArch64: X0-X30 (0-30), SP (31)
    //! - Mapping: Direct for 0-30, special handling for SP
    //! - Temporary: X16-X17 (IP0-IP1) used for instruction sequences
}
