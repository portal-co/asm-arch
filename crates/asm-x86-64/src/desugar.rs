//! Desugaring wrapper for x86-64 memory operands.
//!
//! This module provides a wrapper around `WriterCore` implementations that
//! validates and desugars memory operands (and other invalid operand forms)
//! into forms that are valid for x86-64 instruction encodings.
//!
//! The wrapper ensures:
//! - Memory operands use a register base (literal bases are loaded into temps)
//! - Index/scale pairs use a register index and a scale of 1/2/4/8 (other scales
//!   are materialized into registers)
//! - Displacements fit into a signed 32-bit immediate; otherwise they are
//!   added to the base register and a zero displacement is used
//! - Literal operands used where a register is required are loaded into temps
//! - Mem-to-mem moves/ops are broken into register temporaries
//!
//! Usage: wrap any `WriterCore` with `DesugaringWriter` to automatically apply
//! these fixes before forwarding to the underlying writer.

use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};

use crate::{
    out::{arg::{ArgKind, MemArg, MemArgKind}, WriterCore},
    X64Arch,
};

/// Configuration for the desugaring wrapper.
#[derive(Clone, Copy, Debug)]
pub struct DesugarConfig {
    /// Primary temporary register to use for address calculations.
    pub temp_reg: Reg,
    /// Secondary temporary register to use when primary is in use.
    pub temp_reg2: Reg,
    /// Tertiary temporary register for large immediates.
    pub temp_reg3: Reg,
}

impl Default for DesugarConfig {
    fn default() -> Self {
        Self {
            temp_reg: Reg(15),  // r15 as a high-numbered temp
            temp_reg2: Reg(14), // r14
            temp_reg3: Reg(13), // r13
        }
    }
}

/// Wrapper around WriterCore that desugars/validates complex memory operands.
pub struct DesugaringWriter<'a, W: WriterCore + ?Sized> {
    writer: &'a mut W,
    config: DesugarConfig,
}

impl<'a, W: WriterCore + ?Sized> DesugaringWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer, config: DesugarConfig::default() }
    }
    pub fn with_config(writer: &'a mut W, config: DesugarConfig) -> Self {
        Self { writer, config }
    }

    /// Checks if a displacement fits in a signed 32-bit immediate.
    fn fits_in_i32(d: u64) -> bool {
        d <= i32::MAX as u64
    }

    /// Checks whether a scale is valid for x86 addressing (1,2,4,8).
    fn valid_scale(scale: u32) -> bool {
        matches!(scale, 1 | 2 | 4 | 8)
    }

    /// Desugars a memory operand into a simple base+disp form.
    /// Returns (base_reg, disp) where the returned `disp` is a u32 displacement
    /// suitable for `MemArgKind::Mem`. If the displacement is too large it will
    /// be folded into the returned `base_reg` (and disp==0).
    fn desugar_mem_operand(
        &mut self,
        arch: X64Arch,
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
                        let temp = self.config.temp_reg;
                        self.writer.mov64(arch, &temp, *val)?;
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
                            let tmp = if base_reg == self.config.temp_reg { self.config.temp_reg2 } else { self.config.temp_reg };
                            self.writer.mov64(arch, &tmp, *val)?;
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
                        let mut result_reg = self.config.temp_reg;

                        // Avoid clobbering offset_reg or base_reg
                        if result_reg == offset_reg || result_reg == base_reg {
                            result_reg = self.config.temp_reg2;
                        }

                        // Move offset into result_reg first
                        self.writer.mov(arch, &result_reg, &offset_reg)?;

                        // If scale is power of two, shift
                        if *scale != 0 && (*scale & (*scale - 1)) == 0 {
                            // compute log2(scale)
                            let mut s = *scale;
                            let mut shift = 0u64;
                            while s > 1 { s >>= 1; shift += 1; }
                            // Use shl with immediate shift
                            self.writer.shl(arch, &result_reg, &ArgKind::Lit(shift))?;
                        } else {
                            // Use mul to compute scaled value: result = result_reg * scale
                            // Load scale into temp_reg3
                            let scale_reg = if base_reg == self.config.temp_reg3 || offset_reg == self.config.temp_reg3 {
                                self.config.temp_reg2
                            } else {
                                self.config.temp_reg3
                            };
                            self.writer.mov64(arch, &scale_reg, *scale as u64)?;
                            self.writer.mul(arch, &result_reg, &MemArgKind::NoMem(ArgKind::Reg { reg: scale_reg, size: MemorySize::_64 }))?;
                        }

                        result_reg
                    };

                    // Now compute base + scaled_index into a temp register
                    let result_reg = if self.config.temp_reg == base_reg || self.config.temp_reg == scaled_index_reg {
                        self.config.temp_reg2
                    } else {
                        self.config.temp_reg
                    };
                    // Move base into result_reg then add scaled index
                    self.writer.mov(arch, &result_reg, &base_reg)?;
                    self.writer.add(arch, &result_reg, &MemArgKind::NoMem(ArgKind::Reg { reg: scaled_index_reg, size: MemorySize::_64 }))?;
                    result_reg
                } else {
                    base_reg
                };

                // Handle large displacement: x86 uses signed 32-bit displacement
                if Self::fits_in_i32(*disp as u64) {
                    Ok((effective_base, *disp))
                } else {
                    // Fold displacement into base
                    let temp = if effective_base == self.config.temp_reg { self.config.temp_reg2 } else { self.config.temp_reg };
                    self.writer.mov64(arch, &temp, *disp as u64)?;
                    self.writer.mov(arch, &temp, &effective_base)?;
                    self.writer.add(arch, &temp, &MemArgKind::NoMem(ArgKind::Reg { reg: effective_base, size: MemorySize::_64 }))?;
                    Ok((temp, 0))
                }
            }
        }
    }

    fn simple_mem(base: Reg, disp: u32, size: MemorySize, reg_class: crate::RegisterClass) -> MemArgKind<ArgKind> {
        MemArgKind::Mem { base: ArgKind::Reg { reg: base, size }, offset: None, disp, size, reg_class }
    }

    fn desugar_mem_arg(&mut self, arch: X64Arch, mem_arg: &(dyn MemArg + '_)) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = mem_arg.concrete_mem_kind();
        match concrete {
            MemArgKind::NoMem(_) => Ok(concrete),
            MemArgKind::Mem { base: ArgKind::Lit(_), offset, disp, size, reg_class, .. } => {
                // literal base - load into temp (and fold any index/disp as necessary)
                let (base_reg, new_disp) = self.desugar_mem_operand(arch, &MemArgKind::Mem { base: ArgKind::Lit(0), offset, disp, size, reg_class })?;
                Ok(Self::simple_mem(base_reg, new_disp, size, reg_class))
            }
            MemArgKind::Mem { offset: Some((_, scale)), disp, size, reg_class, .. } if !Self::valid_scale(scale) => {
                // invalid scale - needs materialization
                let (base, new_disp) = self.desugar_mem_operand(arch, &MemArgKind::Mem { base: ArgKind::Reg { reg: Reg(0), size }, offset: Some((ArgKind::Lit(0), scale)), disp, size, reg_class })?;
                Ok(Self::simple_mem(base, new_disp, size, reg_class))
            }
            MemArgKind::Mem { offset: None, disp, size, reg_class, .. } if !Self::fits_in_i32(disp as u64) => {
                // large displacement - fold into base
                let (base, new_disp) = self.desugar_mem_operand(arch, &MemArgKind::Mem { base: ArgKind::Reg { reg: Reg(0), size }, offset: None, disp, size, reg_class })?;
                Ok(Self::simple_mem(base, new_disp, size, reg_class))
            }
            m => Ok(m),
        }
    }

    fn desugar_operand(&mut self, arch: X64Arch, operand: &(dyn MemArg + '_)) -> Result<MemArgKind<ArgKind>, W::Error> {
        let concrete = operand.concrete_mem_kind();
        match concrete {
            MemArgKind::NoMem(ArgKind::Reg { .. }) => Ok(concrete),
            MemArgKind::NoMem(ArgKind::Lit(val)) => {
                // Load literal into temp
                let temp = self.config.temp_reg;
                self.writer.mov64(arch, &temp, val)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg { reg: temp, size: MemorySize::_64 }))
            }
            MemArgKind::Mem { size, .. } => {
                // Load memory operand into temp
                let temp = self.config.temp_reg;
                let desugared = self.desugar_mem_arg(arch, operand)?;
                // Use mov to load from memory into temp
                self.writer.mov(arch, &temp, &desugared)?;
                Ok(MemArgKind::NoMem(ArgKind::Reg { reg: temp, size }))
            }
        }
    }

    /// Helper for binary ops of the form op(a, b) where `a` is both destination and first source.
    fn binary_op<F>(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), op: F) -> Result<(), W::Error>
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
                let desugared_a = self.desugar_mem_arg(cfg, a)?;
                op(self.writer, cfg, &desugared_a, b)
            }
            (false, true) => {
                let desugared_b = self.desugar_operand(cfg, b)?; // ensure b is register or literal handled
                op(self.writer, cfg, a, &desugared_b)
            }
            (true, true) => {
                // both memory - load b into temp and use that
                let temp_b = self.config.temp_reg2;
                let desugared_b_mem = self.desugar_mem_arg(cfg, b)?;
                self.writer.mov(cfg, &temp_b, &desugared_b_mem)?;
                let desugared_a = self.desugar_mem_arg(cfg, a)?;
                op(self.writer, cfg, &desugared_a, &MemArgKind::NoMem(ArgKind::Reg { reg: temp_b, size: MemorySize::_64 }))
            }
        }
    }

    /// Helper for two-operand comparisons where neither operand is a destination.
    fn binary_op_no_dest<F>(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), op: F) -> Result<(), W::Error>
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
                let da = self.desugar_operand(cfg, a)?;
                op(self.writer, cfg, &da, b)
            }
            (false, true) => {
                let db = self.desugar_operand(cfg, b)?;
                op(self.writer, cfg, a, &db)
            }
            (true, true) => {
                // both memory - load one into temp
                let temp_a = self.config.temp_reg;
                let desugared_a = self.desugar_mem_arg(cfg, a)?;
                self.writer.mov(cfg, &temp_a, &desugared_a)?;
                let desugared_b = self.desugar_mem_arg(cfg, b)?;
                op(self.writer, cfg, &MemArgKind::NoMem(ArgKind::Reg { reg: temp_a, size: MemorySize::_64 }), &desugared_b)
            }
        }
    }
}

impl<'a, W: WriterCore + ?Sized> WriterCore for DesugaringWriter<'a, W> {
    type Error = W::Error;

    fn mov(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let dest_conc = dest.concrete_mem_kind();
        let src_conc = src.concrete_mem_kind();

        let dest_is_mem = matches!(dest_conc, MemArgKind::Mem { .. });
        let src_is_mem = matches!(src_conc, MemArgKind::Mem { .. });

        match (dest_is_mem, src_is_mem) {
            (true, true) => {
                // mem-to-mem not allowed: load src into temp then mov
                let temp = self.config.temp_reg;
                let desugared_src = self.desugar_mem_arg(cfg, src)?;
                self.writer.mov(cfg, &temp, &desugared_src)?;
                let desugared_dest = self.desugar_mem_arg(cfg, dest)?;
                self.writer.mov(cfg, &desugared_dest, &MemArgKind::NoMem(ArgKind::Reg { reg: temp, size: MemorySize::_64 }))
            }
            (true, false) => {
                // dest is mem - ensure src is a valid operand (reg or lit)
                if let MemArgKind::NoMem(ArgKind::Lit(v)) = src_conc {
                    // mov can take immediate via mov64 - forward directly
                    let desugared_dest = self.desugar_mem_arg(cfg, dest)?;
                    return self.writer.mov64(cfg, &desugared_dest, v);
                }
                let desugared_src = self.desugar_operand(cfg, src)?;
                let desugared_dest = self.desugar_mem_arg(cfg, dest)?;
                self.writer.mov(cfg, &desugared_dest, &desugared_src)
            }
            (false, true) => {
                // src is mem - load into a temp then mov to dest.
                let desugared_src = self.desugar_mem_arg(cfg, src)?;
                // Choose a load temp that doesn't clobber any registers used by the address calculation
                let load_temp = match &desugared_src {
                    MemArgKind::Mem { base: ArgKind::Reg { reg, .. }, .. } if *reg == self.config.temp_reg => self.config.temp_reg2,
                    _ => self.config.temp_reg,
                };
                self.writer.mov(cfg, &load_temp, &desugared_src)?;
                let desugared_dest = self.desugar_operand(cfg, dest)?;
                self.writer.mov(cfg, &desugared_dest, &MemArgKind::NoMem(ArgKind::Reg { reg: load_temp, size: MemorySize::_64 }))
            }
            (false, false) => {
                // both no-mem: if src is literal prefer mov64
                if let MemArgKind::NoMem(ArgKind::Lit(v)) = src_conc {
                    // mov64 supports immediate
                    return self.writer.mov64(cfg, dest, v);
                }
                // otherwise forward directly
                self.writer.mov(cfg, dest, src)
            }
        }
    }

    fn xchg(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // xchg cannot be mem->mem; desugar similarly to mov
        let dest_conc = dest.concrete_mem_kind();
        let src_conc = src.concrete_mem_kind();
        let dest_is_mem = matches!(dest_conc, MemArgKind::Mem { .. });
        let src_is_mem = matches!(src_conc, MemArgKind::Mem { .. });
        if dest_is_mem && src_is_mem {
            let temp = self.config.temp_reg;
            let desugared_src = self.desugar_mem_arg(cfg, src)?;
            self.writer.mov(cfg, &temp, &desugared_src)?;
            let desugared_dest = self.desugar_mem_arg(cfg, dest)?;
            self.writer.xchg(cfg, &desugared_dest, &MemArgKind::NoMem(ArgKind::Reg { reg: temp, size: MemorySize::_64 }))
        } else {
            let d = if dest_is_mem { self.desugar_mem_arg(cfg, dest)? } else { dest.concrete_mem_kind() };
            let s = if src_is_mem { self.desugar_mem_arg(cfg, src)? } else { src.concrete_mem_kind() };
            self.writer.xchg(cfg, &d, &s)
        }
    }

    // Arithmetic and bitwise ops - map to binary_op helper
    fn add(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.add(cfg, a, b))
    }
    fn sub(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.sub(cfg, a, b))
    }
    fn mul(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.mul(cfg, a, b))
    }
    fn div(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.div(cfg, a, b))
    }
    fn idiv(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.idiv(cfg, a, b))
    }
    fn and(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.and(cfg, a, b))
    }
    fn or(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.or(cfg, a, b))
    }
    fn eor(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.eor(cfg, a, b))
    }
    fn shl(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.shl(cfg, a, b))
    }
    fn shr(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.shr(cfg, a, b))
    }
    fn sar(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op(cfg, a, b, |w, cfg, a, b| w.sar(cfg, a, b))
    }

    // Compare operations
    fn cmp(&mut self, cfg: X64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.binary_op_no_dest(cfg, a, b, |w, cfg, a, b| w.cmp(cfg, a, b))
    }
    fn cmp0(&mut self, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(cfg, op)?;
        self.writer.cmp0(cfg, &desugared)
    }

    fn cmovcc64(&mut self, cfg: X64Arch, cc: crate::ConditionCode, op: &(dyn MemArg + '_), val: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // cmovcc has restrictions similar to others: handle mem/mem
        let op_conc = op.concrete_mem_kind();
        let val_conc = val.concrete_mem_kind();
        if matches!(op_conc, MemArgKind::Mem { .. }) && matches!(val_conc, MemArgKind::Mem { .. }) {
            let temp = self.config.temp_reg;
            let desugared_val = self.desugar_mem_arg(cfg, val)?;
            self.writer.mov(cfg, &temp, &desugared_val)?;
            let desugared_op = self.desugar_mem_arg(cfg, op)?;
            self.writer.cmovcc64(cfg, cc, &desugared_op, &MemArgKind::NoMem(ArgKind::Reg { reg: temp, size: MemorySize::_64 }))
        } else {
            let d = if matches!(op_conc, MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, op)? } else { op.concrete_mem_kind() };
            let v = if matches!(val_conc, MemArgKind::Mem { .. }) { self.desugar_operand(cfg, val)? } else { val.concrete_mem_kind() };
            self.writer.cmovcc64(cfg, cc, &d, &v)
        }
    }

    fn jcc(&mut self, cfg: X64Arch, cc: crate::ConditionCode, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(cfg, op)?;
        self.writer.jcc(cfg, cc, &desugared)
    }

    fn call(&mut self, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(cfg, op)?;
        self.writer.call(cfg, &desugared)
    }

    fn jmp(&mut self, cfg: X64Arch, op: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let desugared = self.desugar_operand(cfg, op)?;
        self.writer.jmp(cfg, &desugared)
    }

    fn lea(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        // lea expects memory-like src; ensure src mem forms are valid
        let src_conc = src.concrete_mem_kind();
        let src_fixed = if matches!(src_conc, MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, src)? } else { src_conc };
        self.writer.lea(cfg, dest, &src_fixed)
    }

    fn get_ip(&mut self, cfg: X64Arch) -> Result<(), Self::Error> { self.writer.get_ip(cfg) }
    fn ret(&mut self, cfg: X64Arch) -> Result<(), Self::Error> { self.writer.ret(cfg) }
    fn mov64(&mut self, cfg: X64Arch, r: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error> { self.writer.mov64(cfg, r, val) }

    // Floating and other ops: ensure operands are valid via desugar_operand where appropriate
    fn fadd(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(cfg, src)?;
        self.writer.fadd(cfg, &d, &s)
    }

    fn fsub(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(cfg, src)?;
        self.writer.fsub(cfg, &d, &s)
    }

    fn fmul(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(cfg, src)?;
        self.writer.fmul(cfg, &d, &s)
    }

    fn fdiv(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, dest)? } else { dest.concrete_mem_kind() };
        let s = self.desugar_operand(cfg, src)?;
        self.writer.fdiv(cfg, &d, &s)
    }

    fn fmov(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let s = self.desugar_operand(cfg, src)?;
        let d = if matches!(dest.concrete_mem_kind(), MemArgKind::Mem { .. }) { self.desugar_mem_arg(cfg, dest)? } else { dest.concrete_mem_kind() };
        self.writer.fmov(cfg, &d, &s)
    }

    fn db(&mut self, cfg: X64Arch, bytes: &[u8]) -> Result<(), Self::Error> { self.writer.db(cfg, bytes) }
}

impl<'a, W, L> crate::out::Writer<L> for DesugaringWriter<'a, W>
where
    W: crate::out::Writer<L> + ?Sized,
{
    fn set_label(&mut self, cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.set_label(cfg, label)
    }
    fn lea_label(&mut self, cfg: X64Arch, dest: &(dyn MemArg + '_), label: L) -> Result<(), Self::Error> {
        self.writer.lea_label(cfg, dest, label)
    }
    fn call_label(&mut self, cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.call_label(cfg, label)
    }
    fn jmp_label(&mut self, cfg: X64Arch, label: L) -> Result<(), Self::Error> {
        self.writer.jmp_label(cfg, label)
    }
    fn jcc_label(&mut self, cfg: X64Arch, cc: crate::ConditionCode, label: L) -> Result<(), Self::Error> {
        self.writer.jcc_label(cfg, cc, label)
    }
}
