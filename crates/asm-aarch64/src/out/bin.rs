extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use portal_pc_asm_common::types::mem::MemorySize;

use crate::out::arg::{ArgKind, MemArgKind};
use crate::out::MemArg;

/// Placeholder label type for [`AArch64Writer`] when no label tracking is needed.
///
/// This is an uninhabited type — it can never be constructed — so a
/// `BTreeMap<NoLabel, usize>` is always empty and zero-cost.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NoLabel {}

// ── register extraction ──────────────────────────────────────────────────────

fn to_reg(arg: &dyn MemArg) -> u32 {
    match arg.concrete_mem_kind() {
        MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => reg.0 as u32,
        _ => 0,
    }
}

fn to_reg_size(arg: &dyn MemArg) -> (u32, MemorySize) {
    match arg.concrete_mem_kind() {
        MemArgKind::NoMem(ArgKind::Reg { reg, size }) => (reg.0 as u32, size),
        _ => (0, MemorySize::_64),
    }
}

fn mem_base_disp(mem: &dyn MemArg) -> (u32, u32) {
    match mem.concrete_mem_kind() {
        MemArgKind::Mem { base: ArgKind::Reg { reg, .. }, disp, .. } => (reg.0 as u32, disp as u32),
        MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => (reg.0 as u32, 0),
        _ => (0, 0),
    }
}

fn lit_value(arg: &dyn MemArg) -> Option<u64> {
    match arg.concrete_mem_kind() {
        MemArgKind::NoMem(ArgKind::Lit(v)) => Some(v),
        _ => None,
    }
}

// ── binary assembler ─────────────────────────────────────────────────────────

/// Binary assembler backend for AArch64 using manual 32-bit instruction encoding.
///
/// All AArch64 instructions are fixed-width 32-bit little-endian. Suitable for
/// AOT compilation and WASM targets (no JIT allocator or FFI dependency).
///
/// The type parameter `L` is the label type used with [`Writer<L, Context>`].
/// It defaults to [`NoLabel`], which means label tracking is compiled away at
/// zero cost. Specify a concrete `L` (e.g. `u32` or a custom enum) to record
/// label→byte-offset mappings via [`set_label`](crate::out::Writer::set_label).
pub struct AArch64Writer<L = NoLabel> {
    buf: Vec<u8>,
    labels: BTreeMap<L, usize>,
}

impl<L> AArch64Writer<L> {
    pub fn new() -> Self {
        Self { buf: Vec::new(), labels: BTreeMap::new() }
    }

    /// Return the assembled bytes, discarding any recorded label offsets.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Return the assembled bytes and the recorded label→offset map.
    pub fn into_parts(self) -> (Vec<u8>, BTreeMap<L, usize>) {
        (self.buf, self.labels)
    }

    /// Current byte offset (number of bytes assembled so far).
    pub fn offset(&self) -> usize {
        self.buf.len()
    }

    #[inline(always)]
    fn emit(&mut self, word: u32) {
        self.buf.extend_from_slice(&word.to_le_bytes());
    }
}

impl<L> Default for AArch64Writer<L> {
    fn default() -> Self {
        Self::new()
    }
}

// ── WriterCore implementation ────────────────────────────────────────────────

impl<L, Context> crate::out::WriterCore<Context> for AArch64Writer<L> {
    type Error = core::convert::Infallible;

    fn brk(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, imm: u16) -> Result<(), Self::Error> {
        // BRK #imm16 = 0xD4200000 | (imm16 << 5)
        self.emit(0xD420_0000 | ((imm as u32) << 5));
        Ok(())
    }

    fn ret(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch) -> Result<(), Self::Error> {
        // RET X30 = 0xD65F03C0
        self.emit(0xD65F_03C0);
        Ok(())
    }

    fn mov(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (rd, size) = to_reg_size(dest);
        if size == MemorySize::_64 || size == MemorySize::_32 {
            if let Some(v) = lit_value(src) {
                // MOVZ Xd/Wd, #imm16 (zero the rest)
                let sf = if size == MemorySize::_64 { 0x8000_0000u32 } else { 0 };
                self.emit(sf | 0x5280_0000 | (((v & 0xFFFF) as u32) << 5) | rd);
            } else {
                let rm = to_reg(src);
                if size == MemorySize::_64 {
                    // MOV Xd, Xn = ORR Xd, XZR, Xn
                    self.emit(0xAA00_03E0 | (rm << 16) | rd);
                } else {
                    // MOV Wd, Wn = ORR Wd, WZR, Wn
                    self.emit(0x2A00_03E0 | (rm << 16) | rd);
                }
            }
        } else {
            // Fallback: 64-bit move
            let rm = to_reg(src);
            self.emit(0xAA00_03E0 | (rm << 16) | rd);
        }
        Ok(())
    }

    fn mov_imm(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        // MOVZ Xd, #(val[15:0])
        self.emit(0xD280_0000 | (((val & 0xFFFF) as u32) << 5) | rd);
        if val > 0xFFFF {
            // MOVK Xd, #(val[31:16]), LSL #16
            self.emit(0xF2A0_0000 | ((((val >> 16) & 0xFFFF) as u32) << 5) | rd);
        }
        if val > 0xFFFF_FFFF {
            // MOVK Xd, #(val[47:32]), LSL #32
            self.emit(0xF2C0_0000 | ((((val >> 32) & 0xFFFF) as u32) << 5) | rd);
        }
        if val > 0xFFFF_FFFF_FFFF {
            // MOVK Xd, #(val[63:48]), LSL #48
            self.emit(0xF2E0_0000 | ((((val >> 48) & 0xFFFF) as u32) << 5) | rd);
        }
        Ok(())
    }

    fn add(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        if let Some(v) = lit_value(b) {
            // ADD Xd, Xn, #imm12
            self.emit(0x9100_0000 | ((v as u32 & 0xFFF) << 10) | (rn << 5) | rd);
        } else {
            let rm = to_reg(b);
            // ADD Xd, Xn, Xm
            self.emit(0x8B00_0000 | (rm << 16) | (rn << 5) | rd);
        }
        Ok(())
    }

    fn sub(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        if let Some(v) = lit_value(b) {
            // SUB Xd, Xn, #imm12
            self.emit(0xD100_0000 | ((v as u32 & 0xFFF) << 10) | (rn << 5) | rd);
        } else {
            let rm = to_reg(b);
            // SUB Xd, Xn, Xm
            self.emit(0xCB00_0000 | (rm << 16) | (rn << 5) | rd);
        }
        Ok(())
    }

    fn and(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        let rm = to_reg(b);
        // AND Xd, Xn, Xm (shifted register, LSL #0)
        self.emit(0x8A00_0000 | (rm << 16) | (rn << 5) | rd);
        Ok(())
    }

    fn orr(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        let rm = to_reg(b);
        // ORR Xd, Xn, Xm
        self.emit(0xAA00_0000 | (rm << 16) | (rn << 5) | rd);
        Ok(())
    }

    fn eor(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        let rm = to_reg(b);
        // EOR Xd, Xn, Xm
        self.emit(0xCA00_0000 | (rm << 16) | (rn << 5) | rd);
        Ok(())
    }

    fn lsl(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        if let Some(v) = lit_value(b) {
            // LSL Xd, Xn, #shift = UBFM Xd, Xn, #(-shift MOD 64), #(63-shift)
            let shift = (v & 0x3F) as u32;
            let immr = (64 - shift) & 0x3F;
            let imms = 63 - shift;
            self.emit(0xD340_0000 | (immr << 16) | (imms << 10) | (rn << 5) | rd);
        } else {
            let rm = to_reg(b);
            // LSLV Xd, Xn, Xm
            self.emit(0x9AC0_2000 | (rm << 16) | (rn << 5) | rd);
        }
        Ok(())
    }

    fn lsr(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        if let Some(v) = lit_value(b) {
            // LSR Xd, Xn, #shift = UBFM Xd, Xn, #shift, #63
            let shift = (v & 0x3F) as u32;
            self.emit(0xD340_0000 | (shift << 16) | (0x3F << 10) | (rn << 5) | rd);
        } else {
            let rm = to_reg(b);
            // LSRV Xd, Xn, Xm
            self.emit(0x9AC0_2400 | (rm << 16) | (rn << 5) | rd);
        }
        Ok(())
    }

    fn asr(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        if let Some(v) = lit_value(b) {
            // ASR Xd, Xn, #shift = SBFM Xd, Xn, #shift, #63
            let shift = (v & 0x3F) as u32;
            self.emit(0x9340_0000 | (shift << 16) | (0x3F << 10) | (rn << 5) | rd);
        } else {
            let rm = to_reg(b);
            // ASRV Xd, Xn, Xm
            self.emit(0x9AC0_2800 | (rm << 16) | (rn << 5) | rd);
        }
        Ok(())
    }

    fn mvn(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rm = to_reg(src);
        // MVN Xd, Xm = ORN Xd, XZR, Xm
        self.emit(0xAA20_03E0 | (rm << 16) | rd);
        Ok(())
    }

    fn mul(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        let rm = to_reg(b);
        // MUL Xd, Xn, Xm = MADD Xd, Xn, Xm, XZR
        self.emit(0x9B00_7C00 | (rm << 16) | (rn << 5) | rd);
        Ok(())
    }

    fn udiv(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        let rm = to_reg(b);
        // UDIV Xd, Xn, Xm
        self.emit(0x9AC0_0800 | (rm << 16) | (rn << 5) | rd);
        Ok(())
    }

    fn sdiv(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(a);
        let rm = to_reg(b);
        // SDIV Xd, Xn, Xm
        self.emit(0x9AC0_0C00 | (rm << 16) | (rn << 5) | rd);
        Ok(())
    }

    fn cmp(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rn = to_reg(a);
        if let Some(v) = lit_value(b) {
            // CMP Xn, #imm12 = SUBS XZR, Xn, #imm
            self.emit(0xF100_001F | ((v as u32 & 0xFFF) << 10) | (rn << 5));
        } else {
            let rm = to_reg(b);
            // CMP Xn, Xm = SUBS XZR, Xn, Xm
            self.emit(0xEB00_001F | (rm << 16) | (rn << 5));
        }
        Ok(())
    }

    fn csel(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, cond: crate::ConditionCode, dest: &(dyn MemArg + '_), true_val: &(dyn MemArg + '_), false_val: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let rn = to_reg(true_val);
        let rm = to_reg(false_val);
        // CSEL Xd, Xn, Xm, cond
        self.emit(0x9A80_0000 | (rm << 16) | ((cond as u32) << 12) | (rn << 5) | rd);
        Ok(())
    }

    fn sxt(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let (rn, size) = to_reg_size(src);
        let instr = match size {
            MemorySize::_8  => 0x9340_1C00 | (rn << 5) | rd, // SXTB Xd, Wn
            MemorySize::_16 => 0x9340_3C00 | (rn << 5) | rd, // SXTH Xd, Wn
            _               => 0x9340_7C00 | (rn << 5) | rd, // SXTW Xd, Wn
        };
        self.emit(instr);
        Ok(())
    }

    fn uxt(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let (rn, size) = to_reg_size(src);
        let instr = match size {
            MemorySize::_8  => 0x5300_1C00 | (rn << 5) | rd, // UXTB Wd, Wn
            _               => 0x5300_3C00 | (rn << 5) | rd, // UXTH Wd, Wn
        };
        self.emit(instr);
        Ok(())
    }

    fn ldr(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (rt, size) = to_reg_size(dest);
        let (rn, disp) = mem_base_disp(mem);
        let instr = match size {
            MemorySize::_64 => 0xF940_0000 | (((disp / 8) & 0xFFF) << 10) | (rn << 5) | rt,
            MemorySize::_32 => 0xB940_0000 | (((disp / 4) & 0xFFF) << 10) | (rn << 5) | rt,
            MemorySize::_16 => 0x7940_0000 | (((disp / 2) & 0xFFF) << 10) | (rn << 5) | rt,
            _               => 0x3940_0000 | ((disp & 0xFFF) << 10) | (rn << 5) | rt,
        };
        self.emit(instr);
        Ok(())
    }

    fn str(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, src: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (rt, size) = to_reg_size(src);
        let (rn, disp) = mem_base_disp(mem);
        let instr = match size {
            MemorySize::_64 => 0xF900_0000 | (((disp / 8) & 0xFFF) << 10) | (rn << 5) | rt,
            MemorySize::_32 => 0xB900_0000 | (((disp / 4) & 0xFFF) << 10) | (rn << 5) | rt,
            MemorySize::_16 => 0x7900_0000 | (((disp / 2) & 0xFFF) << 10) | (rn << 5) | rt,
            _               => 0x3900_0000 | ((disp & 0xFFF) << 10) | (rn << 5) | rt,
        };
        self.emit(instr);
        Ok(())
    }

    fn ldp(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest1: &(dyn MemArg + '_), dest2: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rt1 = to_reg(dest1);
        let rt2 = to_reg(dest2);
        let (rn, disp) = mem_base_disp(mem);
        let simm7 = ((disp as i32 / 8) & 0x7F) as u32;
        // LDP Xt1, Xt2, [Xn, #imm] (signed offset)
        self.emit(0xA940_0000 | (simm7 << 15) | (rt2 << 10) | (rn << 5) | rt1);
        Ok(())
    }

    fn stp(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, src1: &(dyn MemArg + '_), src2: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rt1 = to_reg(src1);
        let rt2 = to_reg(src2);
        let (rn, disp) = mem_base_disp(mem);
        let simm7 = ((disp as i32 / 8) & 0x7F) as u32;
        // STP Xt1, Xt2, [Xn, #imm] (signed offset)
        self.emit(0xA900_0000 | (simm7 << 15) | (rt2 << 10) | (rn << 5) | rt1);
        Ok(())
    }

    fn bl(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        if let Some(v) = lit_value(target) {
            // BL #imm26 (PC-relative, in units of 4 bytes)
            self.emit(0x9400_0000 | ((v as u32 >> 2) & 0x3FF_FFFF));
        } else {
            let rn = to_reg(target);
            // BLR Xn
            self.emit(0xD63F_0000 | (rn << 5));
        }
        Ok(())
    }

    fn br(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rn = to_reg(target);
        // BR Xn
        self.emit(0xD61F_0000 | (rn << 5));
        Ok(())
    }

    fn b(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        if let Some(v) = lit_value(target) {
            // B #imm26
            self.emit(0x1400_0000 | ((v as u32 >> 2) & 0x3FF_FFFF));
        } else {
            let rn = to_reg(target);
            // BR Xn (fallback)
            self.emit(0xD61F_0000 | (rn << 5));
        }
        Ok(())
    }

    fn bcond(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, cond: crate::ConditionCode, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let imm19 = if let Some(v) = lit_value(target) {
            ((v as i64 >> 2) as u32) & 0x7_FFFF
        } else {
            0
        };
        // B.cond #imm19
        self.emit(0x5400_0000 | (imm19 << 5) | (cond as u32));
        Ok(())
    }

    fn adr(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        let imm21 = if let Some(v) = lit_value(src) { v as u32 } else { 0 };
        // ADR Xd, #imm21 (PC-relative)
        let immlo = imm21 & 0x3;
        let immhi = (imm21 >> 2) & 0x7_FFFF;
        self.emit(0x1000_0000 | (immlo << 29) | (immhi << 5) | rd);
        Ok(())
    }

    fn mrs_nzcv(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rd = to_reg(dest);
        // MRS Xd, NZCV = 0xD53B4200 | Rd
        self.emit(0xD53B_4200 | rd);
        Ok(())
    }

    fn msr_nzcv(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let rn = to_reg(src);
        // MSR NZCV, Xn = 0xD51B4200 | Rn
        self.emit(0xD51B_4200 | rn);
        Ok(())
    }

    // ── Floating-point (D registers, double-precision) ───────────────────────

    fn fadd(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let fd = to_reg(dest);
        let fn_ = to_reg(a);
        let fm = to_reg(b);
        // FADD Dd, Dn, Dm
        self.emit(0x1E60_2800 | (fm << 16) | (fn_ << 5) | fd);
        Ok(())
    }

    fn fsub(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let fd = to_reg(dest);
        let fn_ = to_reg(a);
        let fm = to_reg(b);
        // FSUB Dd, Dn, Dm
        self.emit(0x1E60_3800 | (fm << 16) | (fn_ << 5) | fd);
        Ok(())
    }

    fn fmul(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let fd = to_reg(dest);
        let fn_ = to_reg(a);
        let fm = to_reg(b);
        // FMUL Dd, Dn, Dm
        self.emit(0x1E60_0800 | (fm << 16) | (fn_ << 5) | fd);
        Ok(())
    }

    fn fdiv(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let fd = to_reg(dest);
        let fn_ = to_reg(a);
        let fm = to_reg(b);
        // FDIV Dd, Dn, Dm
        self.emit(0x1E60_1800 | (fm << 16) | (fn_ << 5) | fd);
        Ok(())
    }

    fn fmov(&mut self, _ctx: &mut Context, _cfg: crate::AArch64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let fd = to_reg(dest);
        let fn_ = to_reg(src);
        // FMOV Dd, Dn
        self.emit(0x1E60_4000 | (fn_ << 5) | fd);
        Ok(())
    }
}

// ── Writer implementation ────────────────────────────────────────────────────

impl<L: Ord, Context> crate::out::Writer<L, Context> for AArch64Writer<L> {
    fn set_label(
        &mut self,
        _ctx: &mut Context,
        _cfg: crate::AArch64Arch,
        s: L,
    ) -> Result<(), Self::Error> {
        self.labels.insert(s, self.buf.len());
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::out::Writer as _;

    #[test]
    fn set_label_records_byte_offset() {
        let arch = crate::AArch64Arch::default();
        let mut ctx = ();
        let mut w: AArch64Writer<u32> = AArch64Writer::new();

        // Emit two 4-byte instructions (RET = 0xD65F03C0)
        w.emit(0xD65F_03C0);
        w.emit(0xD65F_03C0);
        assert_eq!(w.offset(), 8);

        // Record label 42 at offset 8
        w.set_label(&mut ctx, arch, 42u32).unwrap();

        // Emit one more instruction
        w.emit(0xD420_0000);

        let (bytes, labels) = w.into_parts();
        assert_eq!(bytes.len(), 12);
        assert_eq!(labels[&42u32], 8);
    }
}
