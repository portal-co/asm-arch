//! x86-64 to AArch64 translation shim.
//!
//! This module provides a complete translation layer for x86-64 instructions to AArch64,
//! including a MemArg adapter and stack-based calling convention shim.
//!
//! # Architecture Notes
//!
//! The shim handles conversion between x86-64 and AArch64:
//! - **Memory addressing modes**: Different displacement types (u32 vs i32)
//! - **Register mapping**: x86-64 has 16 GPRs, AArch64 has 31 + SP
//! - **Register classes**: Xmm → Simd, Gpr → Gpr
//! - **Calling convention**: x86-64 stores return addresses on stack, AArch64 uses LR
//!
//! # Stack-Based Calling Convention
//!
//! The shim implements x86-64's calling convention where return addresses are on the stack:
//!
//! - **CALL instructions**: Use label-based shims that push LR and branch to target
//!   - Emits an inline shim with a jump over it to ensure correctness
//!   - The shim pushes LR onto the stack before branching to the target
//!   - Branch-and-link (BL) instruction is used to call the shim
//!   - This maintains x86-64 semantics where return addresses are on the stack
//!
//! - **RET instructions**: Pop return address from stack, then return
//!   - Restores return address from the stack into LR
//!   - Maintains consistency with CALL's stack manipulation
//!
//! The shim system uses labels (via the `Writer<ShimLabel>` trait) to generate
//! inline shims that are more efficient on AArch64 CPUs.
//!
//! # Performance Notes
//!
//! Some x86-64 instructions require multiple AArch64 instructions:
//! - **XCHG**: 3 MOV instructions (no atomic exchange in base AArch64)
//! - **PUSH/POP**: 2 instructions each (SUB+STR / LDR+ADD)
//! - **PUSHF/POPF**: 3 instructions each (MRS+SUB+STR / LDR+ADD+MSR)
//! - **CALL**: 7 instructions (B skip, label, SUB, STR, B target, skip:, BL shim)
//! - **RET**: 3 instructions (LDR+ADD+RET)
//! - **Parity flags**: No direct equivalent, always evaluates to "true"

use portal_pc_asm_common::types::{reg::Reg, mem::MemorySize};
use portal_solutions_asm_x86_64::{
    ConditionCode as X64ConditionCode,
    X64Arch,
    out::{WriterCore as X64WriterCore, Writer as X64Writer, arg::MemArg as X64MemArg},
};
use crate::out::arg::MemArg;

/// Label type for shim system.
///
/// This newtype wraps a `usize` to uniquely identify shim labels in the generated assembly.
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
                            return APXAccess::RegOffset { base: Reg(28), offset_bytes: offset };
                        }
                        APXAccess::None
                    }
                    _ => APXAccess::None,
                }
            }
            X64MemArgKind::Mem { base, offset, disp, .. } => {
                // If base is a register that is an APX late register, indicate to replace base with X28
                match base {
                    X64ArgKind::Reg { reg, .. } => {
                        let r = reg.0;
                        if r >= 24 {
                            let added = ((r - 24) as i32) * 8;
                            return APXAccess::MemBaseOffset { base: Reg(28), added_disp: added };
                        }
                    }
                    _ => {}
                }
                // If index/offset uses an APX register (offset is (reg, scale))
                if let Some((off_arg, scale)) = offset {
                    if let X64ArgKind::Reg { reg, .. } = off_arg {
                        let r = reg.0;
                        if r >= 24 {
                            return APXAccess::MemIndex { base: Reg(28), index: (r - 24) as u32, scale };
                        }
                    }
                }

                APXAccess::None
            }
            _ => APXAccess::None,
        }
    }
}

impl<'a> crate::out::arg::MemArg for MemArgAdapter<'a> {
    fn mem_kind(&self, go: &mut (dyn FnMut(crate::out::arg::MemArgKind<&'_ (dyn crate::out::arg::Arg + '_)>) + '_)) {
        use portal_solutions_asm_x86_64::out::arg::MemArgKind as X64MemArgKind;
        use crate::out::arg::MemArgKind as AArch64MemArgKind;
        
        // Get the x86-64 memory argument kind
        let x64_kind = self.inner.concrete_mem_kind();
        
        // Convert to AArch64 memory argument kind
        match x64_kind {
            X64MemArgKind::NoMem(arg) => {
                // Direct operand - convert register or literal
                let aarch64_arg = convert_arg_kind(arg, self.arch);
                go(AArch64MemArgKind::NoMem(&aarch64_arg));
            }
            X64MemArgKind::Mem { base, offset, disp, size, reg_class } => {
                // Memory reference - convert components
                let aarch64_base = convert_arg_kind(base, self.arch);
                let aarch64_offset = offset.map(|(off, scale)| (convert_arg_kind(off, self.arch), scale));
                let aarch64_disp = disp as i32; // Convert u32 to i32
                let aarch64_reg_class = convert_register_class(reg_class);
                
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
                            mode: crate::out::arg::AddressingMode::Offset,
                        });
                    }
                    Some((off, scale)) => {
                        go(AArch64MemArgKind::Mem {
                            base: &aarch64_base,
                            offset: Some((off, *scale)),
                            disp: aarch64_disp,
                            size,
                            reg_class: aarch64_reg_class,
                            mode: crate::out::arg::AddressingMode::Offset,
                        });
                    }
                }
            }
            _ => {
                // Handle any future variants with a default behavior
                let aarch64_arg = crate::out::arg::ArgKind::Lit(0);
                go(AArch64MemArgKind::NoMem(&aarch64_arg));
            }
        }
    }
}

/// Converts x86-64 ArgKind to AArch64 ArgKind with register mapping.
fn convert_arg_kind(arg: portal_solutions_asm_x86_64::out::arg::ArgKind, arch: X64Arch) -> crate::out::arg::ArgKind {
    use portal_solutions_asm_x86_64::out::arg::ArgKind as X64ArgKind;
    use crate::out::arg::ArgKind as AArch64ArgKind;
    
    match arg {
        X64ArgKind::Reg { reg, size } => {
            // Map x86-64 register to AArch64 System V ABI register, taking APX into account
            let aarch64_reg = map_x64_register_to_aarch64(reg, arch);
            AArch64ArgKind::Reg { reg: aarch64_reg, size }
        }
        X64ArgKind::Lit(val) => AArch64ArgKind::Lit(val),
        _ => AArch64ArgKind::Lit(0), // Handle any future variants
    }
}

/// Converts x86-64 RegisterClass to AArch64 RegisterClass.
fn convert_register_class(reg_class: portal_solutions_asm_x86_64::RegisterClass) -> crate::RegisterClass {
    use portal_solutions_asm_x86_64::RegisterClass as X64RegClass;
    use crate::RegisterClass as AArch64RegClass;
    
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
pub trait WriterShimExt: crate::out::Writer<ShimLabel> {
    /// Loads a value to a register, using LDR if source is memory.
    ///
    /// If `src` is already a register, this performs a MOV.
    /// If `src` is a memory location, this performs an LDR.
    fn load_to_reg(
        &mut self,
        cfg: crate::AArch64Arch,
        dest: &Reg,
        src: &(dyn crate::out::arg::MemArg + '_),
    ) -> Result<(), <Self as crate::out::WriterCore>::Error> {
        use crate::out::arg::MemArgKind;
        
        let src_kind = src.concrete_mem_kind();
        match src_kind {
            MemArgKind::NoMem(_) => {
                // Source is a register or immediate, use MOV
                self.mov(cfg, dest, src)
            }
            MemArgKind::Mem { .. } => {
                // Source is memory, use LDR
                self.ldr(cfg, dest, src)
            }
        }
    }
    
    /// Stores a value from a register, using STR if destination is memory.
    ///
    /// If `dest` is already a register, this performs a MOV.
    /// If `dest` is a memory location, this performs a STR.
    fn store_from_reg(
        &mut self,
        cfg: crate::AArch64Arch,
        dest: &(dyn crate::out::arg::MemArg + '_),
        src: &Reg,
    ) -> Result<(), <Self as crate::out::WriterCore>::Error> {
        use crate::out::arg::MemArgKind;
        
        let dest_kind = dest.concrete_mem_kind();
        match dest_kind {
            MemArgKind::NoMem(_) => {
                // Destination is a register, use MOV
                self.mov(cfg, dest, src)
            }
            MemArgKind::Mem { .. } => {
                // Destination is memory, use STR
                self.str(cfg, src, dest)
            }
        }
    }
}

// Blanket implementation for all types that implement Writer<ShimLabel>
impl<W: crate::out::Writer<ShimLabel>> WriterShimExt for W {}

/// Helper macro to handle two-operand instructions with memory operands.
///
/// Pattern: INSTR a, b where a = INSTR(a, b)
/// Handles all combinations of register/memory operands using LDR/STR as needed.
macro_rules! handle_two_operand_instr {
    ($self:expr, $a:expr, $b:expr, $instr:ident) => {{
        use crate::out::arg::MemArgKind;
        
        let a_adapter = MemArgAdapter::new($a, _cfg);
        let b_adapter = MemArgAdapter::new($b, _cfg);
        
        let a_kind = a_adapter.concrete_mem_kind();
        let b_kind = b_adapter.concrete_mem_kind();
        
        match (a_kind, b_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both are registers/immediates - direct operation
                $self.inner.$instr($self.aarch64_cfg, &a_adapter, &a_adapter, &b_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // a is memory, b is register - LDR a, INSTR, STR a
                let temp = Reg(16); // x16
                $self.inner.ldr($self.aarch64_cfg, &temp, &a_adapter)?;
                $self.inner.$instr($self.aarch64_cfg, &temp, &temp, &b_adapter)?;
                $self.inner.str($self.aarch64_cfg, &temp, &a_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // a is register, b is memory - LDR b into temp, then INSTR
                let temp = Reg(17); // x17
                $self.inner.ldr($self.aarch64_cfg, &temp, &b_adapter)?;
                $self.inner.$instr($self.aarch64_cfg, &a_adapter, &a_adapter, &temp)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Both are memory - LDR both, INSTR, STR result
                let temp_a = Reg(16); // x16
                let temp_b = Reg(17); // x17
                $self.inner.ldr($self.aarch64_cfg, &temp_a, &a_adapter)?;
                $self.inner.ldr($self.aarch64_cfg, &temp_b, &b_adapter)?;
                $self.inner.$instr($self.aarch64_cfg, &temp_a, &temp_a, &temp_b)?;
                $self.inner.str($self.aarch64_cfg, &temp_a, &a_adapter)
            }
        }
    }};
}

/// Helper for two-operand instructions where result overwrites first operand (like SUB in x86).
/// For SUB, the AArch64 instruction is: SUB a, a, b (but only takes 2 args in trait)
macro_rules! handle_two_operand_instr_2arg {
    ($self:expr, $a:expr, $b:expr, $instr:ident) => {{
        use crate::out::arg::MemArgKind;
        
        let a_adapter = MemArgAdapter::new($a, _cfg);
        let b_adapter = MemArgAdapter::new($b, _cfg);
        
        let a_kind = a_adapter.concrete_mem_kind();
        let b_kind = b_adapter.concrete_mem_kind();
        
        match (a_kind, b_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both are registers/immediates - direct operation with dest=a
                $self.inner.$instr($self.aarch64_cfg, &a_adapter, &a_adapter, &b_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // a is memory, b is register - LDR a, INSTR, STR a
                let temp = Reg(16); // x16
                $self.inner.ldr($self.aarch64_cfg, &temp, &a_adapter)?;
                $self.inner.$instr($self.aarch64_cfg, &temp, &temp, &b_adapter)?;
                $self.inner.str($self.aarch64_cfg, &temp, &a_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // a is register, b is memory - LDR b into temp, then INSTR with dest=a
                let temp = Reg(17); // x17
                $self.inner.ldr($self.aarch64_cfg, &temp, &b_adapter)?;
                $self.inner.$instr($self.aarch64_cfg, &a_adapter, &a_adapter, &temp)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Both are memory - LDR both, INSTR, STR result
                let temp_a = Reg(16); // x16
                let temp_b = Reg(17); // x17
                $self.inner.ldr($self.aarch64_cfg, &temp_a, &a_adapter)?;
                $self.inner.ldr($self.aarch64_cfg, &temp_b, &b_adapter)?;
                $self.inner.$instr($self.aarch64_cfg, &temp_a, &temp_a, &temp_b)?;
                $self.inner.str($self.aarch64_cfg, &temp_a, &a_adapter)
            }
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
    pub aarch64_cfg: crate::AArch64Arch,
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
    pub fn with_config(inner: W, aarch64_cfg: crate::AArch64Arch) -> Self {
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
pub fn translate_condition(cc: X64ConditionCode) -> crate::ConditionCode {
    match cc {
        X64ConditionCode::E => crate::ConditionCode::EQ,  // Equal
        X64ConditionCode::NE => crate::ConditionCode::NE, // Not equal
        X64ConditionCode::B => crate::ConditionCode::LO,  // Unsigned less (below)
        X64ConditionCode::NB => crate::ConditionCode::HS, // Unsigned greater or equal (not below)
        X64ConditionCode::A => crate::ConditionCode::HI,  // Unsigned greater (above)
        X64ConditionCode::NA => crate::ConditionCode::LS, // Unsigned less or equal (not above)
        X64ConditionCode::L => crate::ConditionCode::LT,  // Signed less
        X64ConditionCode::NL => crate::ConditionCode::GE, // Signed greater or equal
        X64ConditionCode::G => crate::ConditionCode::GT,  // Signed greater
        X64ConditionCode::NG => crate::ConditionCode::LE, // Signed less or equal
        X64ConditionCode::O => crate::ConditionCode::VS,  // Overflow
        X64ConditionCode::NO => crate::ConditionCode::VC, // No overflow
        X64ConditionCode::S => crate::ConditionCode::MI,  // Sign (negative)
        X64ConditionCode::NS => crate::ConditionCode::PL, // No sign (positive)
        X64ConditionCode::P => crate::ConditionCode::AL,  // Parity - no direct equivalent, use always
        X64ConditionCode::NP => crate::ConditionCode::AL, // No parity - no direct equivalent
        _ => crate::ConditionCode::AL, // Catch-all for any future variants
    }
}

impl<W: crate::out::Writer<ShimLabel>> X64WriterCore for X64ToAArch64Shim<W> {
    type Error = W::Error;

    fn hlt(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 HLT -> AArch64 BRK
        self.inner.brk(self.aarch64_cfg, 0)
    }

    fn xchg(
        &mut self,
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
        self.inner.mov(self.aarch64_cfg, &temp, &dest_adapter)?;
        self.inner.mov(self.aarch64_cfg, &dest_adapter, &src_adapter)?;
        self.inner.mov(self.aarch64_cfg, &src_adapter, &temp)?;
        Ok(())
    }

    fn mov(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOV -> AArch64 MOV/LDR/STR depending on operands
        use crate::out::arg::MemArgKind;
        
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        
        let dest_kind = dest_adapter.concrete_mem_kind();
        let src_kind = src_adapter.concrete_mem_kind();
        
        match (dest_kind, src_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Register to register or immediate to register - use MOV
                self.inner.mov(self.aarch64_cfg, &dest_adapter, &src_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // Memory to register - use LDR
                self.inner.ldr(self.aarch64_cfg, &dest_adapter, &src_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // Register to memory - use STR
                self.inner.str(self.aarch64_cfg, &src_adapter, &dest_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Memory to memory - need temporary register
                // Use x16 (IP0) as temporary
                let temp = Reg(16);
                self.inner.ldr(self.aarch64_cfg, &temp, &src_adapter)?;
                self.inner.str(self.aarch64_cfg, &temp, &dest_adapter)
            }
        }
    }

    fn sub(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SUB a, b (a = a - b) -> AArch64 SUB a, a, b
        // Handle memory operands with LDR/STR
        handle_two_operand_instr_2arg!(self, a, b, sub)
    }

    fn add(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 ADD a, b (a = a + b) -> AArch64 ADD a, a, b
        // Handle memory operands with LDR/STR
        handle_two_operand_instr_2arg!(self, a, b, add)
    }

    fn movsx(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOVSX -> AArch64 SXTB/SXTH/SXTW (handle memory operands)
        use crate::out::arg::MemArgKind;
        
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        let src_kind = src_adapter.concrete_mem_kind();
        
        match src_kind {
            MemArgKind::NoMem(_) => {
                // Source is register - direct SXT, then store if needed
                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner.sxt(self.aarch64_cfg, &dest_adapter, &src_adapter)
                    }
                    MemArgKind::Mem { .. } => {
                        let temp = Reg(16); // x16
                        self.inner.sxt(self.aarch64_cfg, &temp, &src_adapter)?;
                        self.inner.str(self.aarch64_cfg, &temp, &dest_adapter)
                    }
                }
            }
            MemArgKind::Mem { .. } => {
                // Source is memory - LDR, SXT, store if needed
                let temp = Reg(16); // x16
                self.inner.ldr(self.aarch64_cfg, &temp, &src_adapter)?;
                let temp2 = Reg(17); // x17 for result
                self.inner.sxt(self.aarch64_cfg, &temp2, &temp)?;
                
                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner.mov(self.aarch64_cfg, &dest_adapter, &temp2)
                    }
                    MemArgKind::Mem { .. } => {
                        self.inner.str(self.aarch64_cfg, &temp2, &dest_adapter)
                    }
                }
            }
        }
    }

    fn movzx(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOVZX -> AArch64 UXTB/UXTH (handle memory operands)
        use crate::out::arg::MemArgKind;
        
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        let src_kind = src_adapter.concrete_mem_kind();
        
        match src_kind {
            MemArgKind::NoMem(_) => {
                // Source is register - direct UXT, then store if needed
                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner.uxt(self.aarch64_cfg, &dest_adapter, &src_adapter)
                    }
                    MemArgKind::Mem { .. } => {
                        let temp = Reg(16); // x16
                        self.inner.uxt(self.aarch64_cfg, &temp, &src_adapter)?;
                        self.inner.str(self.aarch64_cfg, &temp, &dest_adapter)
                    }
                }
            }
            MemArgKind::Mem { .. } => {
                // Source is memory - LDR, UXT, store if needed
                let temp = Reg(16); // x16
                self.inner.ldr(self.aarch64_cfg, &temp, &src_adapter)?;
                let temp2 = Reg(17); // x17 for result
                self.inner.uxt(self.aarch64_cfg, &temp2, &temp)?;
                
                let dest_kind = dest_adapter.concrete_mem_kind();
                match dest_kind {
                    MemArgKind::NoMem(_) => {
                        self.inner.mov(self.aarch64_cfg, &dest_adapter, &temp2)
                    }
                    MemArgKind::Mem { .. } => {
                        self.inner.str(self.aarch64_cfg, &temp2, &dest_adapter)
                    }
                }
            }
        }
    }

    fn push(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 PUSH -> AArch64 STR with pre-indexed addressing
        // [sp, #-8]! means: sp = sp - 8, then str to [sp]
        let sp = Reg(31); // SP
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.str(
            self.aarch64_cfg,
            &op_adapter,
            &crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: -8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PreIndex,
            },
        )
    }

    fn pop(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 POP -> AArch64 LDR with post-indexed addressing
        // [sp], #8 means: ldr from [sp], then sp = sp + 8
        let sp = Reg(31); // SP
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.ldr(
            self.aarch64_cfg,
            &op_adapter,
            &crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PostIndex,
            },
        )
    }

    fn pushf(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 PUSHF -> AArch64 MRS NZCV + STR with pre-indexed addressing
        // Store NZCV flags using MRS
        let temp = Reg(16); // x16
        let sp = Reg(31);
        // Read NZCV flags into temp register
        self.inner.mrs_nzcv(self.aarch64_cfg, &temp)?;
        // Store flags to stack with pre-decrement: [sp, #-8]!
        self.inner.str(
            self.aarch64_cfg,
            &temp,
            &crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: -8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PreIndex,
            },
        )
    }

    fn popf(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 POPF -> AArch64 LDR with post-indexed addressing + MSR NZCV
        let temp = Reg(16); // x16
        let sp = Reg(31);
        // Load flags from stack with post-increment: [sp], #8
        self.inner.ldr(
            self.aarch64_cfg,
            &temp,
            &crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PostIndex,
            },
        )?;
        // Write flags back to NZCV
        self.inner.msr_nzcv(self.aarch64_cfg, &temp)
    }

    fn call(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 CALL -> AArch64 call shim using labels
        // Strategy: Branch to a shim that pushes LR and branches to the target
        // The shim is emitted inline with a jump over it to ensure correctness
        
        let sp = Reg(31); // SP
        let lr = Reg(30); // LR (x30)
        
        // Generate unique labels
        let shim_label = self.next_shim_label();
        let skip_label = self.next_shim_label();
        
        // Jump over the shim to skip_label
        self.inner.b_label(self.aarch64_cfg, skip_label)?;
        
        // Emit the call shim inline
        self.inner.set_label(self.aarch64_cfg, shim_label)?;
        
        // Push LR onto stack with pre-indexed addressing: [sp, #-8]!
        self.inner.str(
            self.aarch64_cfg,
            &lr,
            &crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: -8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PreIndex,
            },
        )?;
        
        // Branch to the target
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.b(self.aarch64_cfg, &op_adapter)?;
        
        // Set skip label (execution continues here)
        self.inner.set_label(self.aarch64_cfg, skip_label)?;
        
        // Branch and link to the shim
        self.inner.bl_label(self.aarch64_cfg, shim_label)?;
        
        Ok(())
    }

    fn jmp(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 JMP -> AArch64 B or BR
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.b(self.aarch64_cfg, &op_adapter)
    }

    fn cmp(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 CMP -> AArch64 CMP (handle memory operands)
        use crate::out::arg::MemArgKind;
        
        let a_adapter = MemArgAdapter::new(a, _cfg);
        let b_adapter = MemArgAdapter::new(b, _cfg);
        
        let a_kind = a_adapter.concrete_mem_kind();
        let b_kind = b_adapter.concrete_mem_kind();
        
        match (a_kind, b_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both are registers/immediates - direct CMP
                self.inner.cmp(self.aarch64_cfg, &a_adapter, &b_adapter)
            }
            (MemArgKind::Mem { .. }, _) => {
                // a is memory - LDR into temp, then CMP
                let temp = Reg(16); // x16
                self.inner.ldr(self.aarch64_cfg, &temp, &a_adapter)?;
                if matches!(b_kind, MemArgKind::Mem { .. }) {
                    let temp_b = Reg(17); // x17
                    self.inner.ldr(self.aarch64_cfg, &temp_b, &b_adapter)?;
                    self.inner.cmp(self.aarch64_cfg, &temp, &temp_b)
                } else {
                    self.inner.cmp(self.aarch64_cfg, &temp, &b_adapter)
                }
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // b is memory - LDR into temp, then CMP
                let temp = Reg(17); // x17
                self.inner.ldr(self.aarch64_cfg, &temp, &b_adapter)?;
                self.inner.cmp(self.aarch64_cfg, &a_adapter, &temp)
            }
        }
    }

    fn cmp0(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 CMP op, 0 -> AArch64 CMP op, #0 (handle memory operands)
        use crate::out::arg::MemArgKind;
        
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let op_kind = op_adapter.concrete_mem_kind();
        
        match op_kind {
            MemArgKind::NoMem(_) => {
                // Register/immediate - direct CMP
                self.inner.cmp(self.aarch64_cfg, &op_adapter, &0u64)
            }
            MemArgKind::Mem { .. } => {
                // Memory - LDR into temp, then CMP
                let temp = Reg(16); // x16
                self.inner.ldr(self.aarch64_cfg, &temp, &op_adapter)?;
                self.inner.cmp(self.aarch64_cfg, &temp, &0u64)
            }
        }
    }

    fn cmovcc64(
        &mut self,
        _cfg: X64Arch,
        cond: X64ConditionCode,
        op: &(dyn X64MemArg + '_),
        val: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 CMOVcc -> AArch64 CSEL (handle memory operands)
        use crate::out::arg::MemArgKind;
        
        let aarch64_cond = translate_condition(cond);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let val_adapter = MemArgAdapter::new(val, _cfg);
        
        let op_kind = op_adapter.concrete_mem_kind();
        let val_kind = val_adapter.concrete_mem_kind();
        
        match (op_kind, val_kind) {
            (MemArgKind::NoMem(_), MemArgKind::NoMem(_)) => {
                // Both registers - direct CSEL
                self.inner.csel(self.aarch64_cfg, aarch64_cond, &op_adapter, &val_adapter, &op_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::NoMem(_)) => {
                // op is memory - LDR, CSEL, STR
                let temp = Reg(16); // x16
                self.inner.ldr(self.aarch64_cfg, &temp, &op_adapter)?;
                self.inner.csel(self.aarch64_cfg, aarch64_cond, &temp, &val_adapter, &temp)?;
                self.inner.str(self.aarch64_cfg, &temp, &op_adapter)
            }
            (MemArgKind::NoMem(_), MemArgKind::Mem { .. }) => {
                // val is memory - LDR val, then CSEL
                let temp = Reg(17); // x17
                self.inner.ldr(self.aarch64_cfg, &temp, &val_adapter)?;
                self.inner.csel(self.aarch64_cfg, aarch64_cond, &op_adapter, &temp, &op_adapter)
            }
            (MemArgKind::Mem { .. }, MemArgKind::Mem { .. }) => {
                // Both memory - LDR both, CSEL, STR
                let temp_op = Reg(16); // x16
                let temp_val = Reg(17); // x17
                self.inner.ldr(self.aarch64_cfg, &temp_op, &op_adapter)?;
                self.inner.ldr(self.aarch64_cfg, &temp_val, &val_adapter)?;
                self.inner.csel(self.aarch64_cfg, aarch64_cond, &temp_op, &temp_val, &temp_op)?;
                self.inner.str(self.aarch64_cfg, &temp_op, &op_adapter)
            }
        }
    }

    fn jcc(
        &mut self,
        _cfg: X64Arch,
        cond: X64ConditionCode,
        op: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 Jcc -> AArch64 B.cond
        let aarch64_cond = translate_condition(cond);
        let op_adapter = MemArgAdapter::new(op, _cfg);
        self.inner.bcond(self.aarch64_cfg, aarch64_cond, &op_adapter)
    }

    fn u32(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 AND op, 0xffffffff -> AArch64 AND op, op, #0xffffffff (handle memory)
        use crate::out::arg::MemArgKind;
        
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let op_kind = op_adapter.concrete_mem_kind();
        let temp = Reg(16); // Use temp register for immediate
        let temp2 = Reg(17); // For result if memory
        
        self.inner.mov_imm(self.aarch64_cfg, &temp, 0xffffffff)?;
        
        match op_kind {
            MemArgKind::NoMem(_) => {
                // Register - direct AND
                self.inner.and(self.aarch64_cfg, &op_adapter, &op_adapter, &temp)
            }
            MemArgKind::Mem { .. } => {
                // Memory - LDR, AND, STR
                self.inner.ldr(self.aarch64_cfg, &temp2, &op_adapter)?;
                self.inner.and(self.aarch64_cfg, &temp2, &temp2, &temp)?;
                self.inner.str(self.aarch64_cfg, &temp2, &op_adapter)
            }
        }
    }

    fn not(&mut self, _cfg: X64Arch, op: &(dyn X64MemArg + '_)) -> Result<(), Self::Error> {
        // x86-64 NOT -> AArch64 MVN (handle memory operands)
        use crate::out::arg::MemArgKind;
        
        let op_adapter = MemArgAdapter::new(op, _cfg);
        let op_kind = op_adapter.concrete_mem_kind();
        
        match op_kind {
            MemArgKind::NoMem(_) => {
                // Register - direct MVN
                self.inner.mvn(self.aarch64_cfg, &op_adapter, &op_adapter)
            }
            MemArgKind::Mem { .. } => {
                // Memory - LDR, MVN, STR
                let temp = Reg(16); // x16
                self.inner.ldr(self.aarch64_cfg, &temp, &op_adapter)?;
                self.inner.mvn(self.aarch64_cfg, &temp, &temp)?;
                self.inner.str(self.aarch64_cfg, &temp, &op_adapter)
            }
        }
    }

    fn lea(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 LEA -> AArch64 ADD/ADR (depending on context)
        // For simplicity, use ADR for now
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.adr(self.aarch64_cfg, &dest_adapter, &src_adapter)
    }

    fn get_ip(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 get IP (typically via CALL trick) -> AArch64 ADR
        // PERFORMANCE: Different approach than x86-64
        let pc_reg = Reg(30); // LR (link register)
        self.inner.adr(self.aarch64_cfg, &pc_reg, &0u64)
    }

    fn ret(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
        // x86-64 RET -> AArch64 ret shim (inline, no jump)
        // Directly emit: pop return address from stack, then return
        
        let sp = Reg(31); // SP
        let lr = Reg(30); // LR (x30)
        
        // Pop return address from stack with post-indexed addressing: [sp], #8
        self.inner.ldr(
            self.aarch64_cfg,
            &lr,
            &crate::out::arg::MemArgKind::Mem {
                base: crate::out::arg::ArgKind::Reg { reg: sp, size: MemorySize::_64 },
                offset: None,
                disp: 8,
                size: MemorySize::_64,
                reg_class: crate::RegisterClass::Gpr,
                mode: crate::out::arg::AddressingMode::PostIndex,
            },
        )?;
        
        // Return
        self.inner.ret(self.aarch64_cfg)
    }

    fn mov64(
        &mut self,
        _cfg: X64Arch,
        r: &(dyn X64MemArg + '_),
        val: u64,
    ) -> Result<(), Self::Error> {
        // x86-64 MOV r, imm64 -> AArch64 MOVZ/MOVK sequence
        let r_adapter = MemArgAdapter::new(r, _cfg);
        self.inner.mov_imm(self.aarch64_cfg, &r_adapter, val)
    }

    fn mul(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MUL a, b -> AArch64 MUL a, a, b
        handle_two_operand_instr!(self, a, b, mul)
    }

    fn div(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 DIV a, b -> AArch64 UDIV a, a, b
        handle_two_operand_instr!(self, a, b, udiv)
    }

    fn idiv(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 IDIV a, b -> AArch64 SDIV a, a, b
        handle_two_operand_instr!(self, a, b, sdiv)
    }

    fn and(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 AND a, b -> AArch64 AND a, a, b
        handle_two_operand_instr!(self, a, b, and)
    }

    fn or(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 OR a, b -> AArch64 ORR a, a, b
        handle_two_operand_instr!(self, a, b, orr)
    }

    fn eor(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 XOR a, b -> AArch64 EOR a, a, b
        handle_two_operand_instr!(self, a, b, eor)
    }

    fn shl(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SHL a, b -> AArch64 LSL a, a, b
        handle_two_operand_instr!(self, a, b, lsl)
    }

    fn shr(
        &mut self,
        _cfg: X64Arch,
        a: &(dyn X64MemArg + '_),
        b: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SHR a, b -> AArch64 LSR a, a, b
        handle_two_operand_instr!(self, a, b, lsr)
    }

    fn fadd(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 ADDSD -> AArch64 FADD
        handle_two_operand_instr!(self, dest, src, fadd)
    }

    fn fsub(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 SUBSD -> AArch64 FSUB
        handle_two_operand_instr!(self, dest, src, fsub)
    }

    fn fmul(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MULSD -> AArch64 FMUL
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fmul(self.aarch64_cfg, &dest_adapter, &dest_adapter, &src_adapter)
    }

    fn fdiv(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 DIVSD -> AArch64 FDIV
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fdiv(self.aarch64_cfg, &dest_adapter, &dest_adapter, &src_adapter)
    }

    fn fmov(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        src: &(dyn X64MemArg + '_),
    ) -> Result<(), Self::Error> {
        // x86-64 MOVSD -> AArch64 FMOV
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        let src_adapter = MemArgAdapter::new(src, _cfg);
        self.inner.fmov(self.aarch64_cfg, &dest_adapter, &src_adapter)
    }
}

impl<W: crate::out::Writer<ShimLabel>, L> X64Writer<L> for X64ToAArch64Shim<W>
where
    W: crate::out::Writer<L>,
{
    fn set_label(&mut self, _cfg: X64Arch, s: L) -> Result<(), Self::Error> {
        self.inner.set_label(self.aarch64_cfg, s)
    }

    fn lea_label(
        &mut self,
        _cfg: X64Arch,
        dest: &(dyn X64MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let dest_adapter = MemArgAdapter::new(dest, _cfg);
        self.inner.adr_label(self.aarch64_cfg, &dest_adapter, label)
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
