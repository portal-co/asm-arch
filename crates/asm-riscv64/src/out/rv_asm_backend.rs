extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use rv_asm::{Imm, Inst, Reg as RvReg, Xlen};

use crate::out::arg::{ArgKind, MemArgKind};
use crate::out::MemArg;

/// Placeholder label type for [`RvAsmWriter`] when no label tracking is needed.
///
/// This is an uninhabited type — it can never be constructed — so a
/// `BTreeMap<NoLabel, usize>` is always empty and zero-cost.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NoLabel {}

fn to_rv_reg(arg: &dyn MemArg) -> RvReg {
    match arg.concrete_mem_kind() {
        MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => RvReg(reg.0 as u8),
        _ => RvReg::ZERO,
    }
}

fn mem_base_offset(arg: &dyn MemArg) -> (RvReg, Imm) {
    match arg.concrete_mem_kind() {
        MemArgKind::Mem { base: ArgKind::Reg { reg, .. }, disp, .. } => {
            (RvReg(reg.0 as u8), Imm::new_i32(disp as i32))
        }
        MemArgKind::NoMem(ArgKind::Reg { reg, .. }) => (RvReg(reg.0 as u8), Imm::ZERO),
        _ => (RvReg::ZERO, Imm::ZERO),
    }
}

fn lit_as_imm(arg: &dyn MemArg) -> Imm {
    match arg.concrete_mem_kind() {
        MemArgKind::NoMem(ArgKind::Lit(v)) => Imm::new_i32(v as i32),
        _ => Imm::ZERO,
    }
}

/// Binary assembler backend for RISC-V 64 using `rv_asm::Inst::encode_normal`.
///
/// The type parameter `L` is the label type used with [`Writer<L, Context>`].
/// It defaults to [`NoLabel`], which means label tracking is compiled away at
/// zero cost. Specify a concrete `L` (e.g. `u32` or a custom enum) to record
/// label→byte-offset mappings via [`set_label`](crate::out::Writer::set_label).
/// Kind-specific data for a pending label fixup.
enum RvFixupKind {
    /// JAL rd, #offset — 4 bytes, J-type immediate.
    Jal { rd: u32 },
    /// LA rd, label — 8 bytes (AUIPC rd, hi + ADDI rd, rd, lo).
    La { rd: u32 },
    /// Conditional branch — 4 bytes, B-type immediate.
    /// `rs1`/`rs2` are already in emission order (may have been swapped for pseudo-conditions).
    BCond { rs1: u32, rs2: u32, funct3: u32 },
}

struct RvFixup<L> {
    instr_offset: usize,
    label: L,
    kind: RvFixupKind,
}

/// Encode a J-type (JAL) instruction with a signed byte offset.
fn encode_jal(rd: u32, delta: i32) -> u32 {
    let d = delta as u32;
    let imm20    = (d >> 20) & 1;
    let imm10_1  = (d >> 1)  & 0x3FF;
    let imm11    = (d >> 11) & 1;
    let imm19_12 = (d >> 12) & 0xFF;
    (imm20 << 31) | (imm10_1 << 21) | (imm11 << 20) | (imm19_12 << 12) | (rd << 7) | 0x6F
}

/// Encode a B-type conditional branch with a signed byte offset.
fn encode_branch(rs1: u32, rs2: u32, funct3: u32, delta: i32) -> u32 {
    let d = delta as u32;
    let imm12   = (d >> 12) & 1;
    let imm10_5 = (d >> 5)  & 0x3F;
    let imm4_1  = (d >> 1)  & 0xF;
    let imm11   = (d >> 11) & 1;
    (imm12 << 31) | (imm10_5 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm4_1 << 8) | (imm11 << 7) | 0x63
}

/// Returns `(funct3, swap_operands)`.  RISC-V pseudo-conditions GT/LE/GTU/LEU
/// are encoded as the reverse comparison with rs1 and rs2 swapped.
fn funct3_for_cond(cond: crate::ConditionCode) -> (u32, bool) {
    use crate::ConditionCode::*;
    match cond {
        EQ  => (0b000, false),
        NE  => (0b001, false),
        LT  => (0b100, false),
        GE  => (0b101, false),
        LTU => (0b110, false),
        GEU => (0b111, false),
        GT  => (0b100, true),   // BGT rs1,rs2 = BLT rs2,rs1
        LE  => (0b101, true),   // BLE rs1,rs2 = BGE rs2,rs1
        GTU => (0b110, true),   // BGTU rs1,rs2 = BLTU rs2,rs1
        LEU => (0b111, true),   // BLEU rs1,rs2 = BGEU rs2,rs1
    }
}

impl<L> RvFixup<L> {
    fn apply(&self, buf: &mut Vec<u8>, target: usize) {
        let delta = (target as i64 - self.instr_offset as i64) as i32;
        match &self.kind {
            RvFixupKind::Jal { rd } => {
                let word = encode_jal(*rd, delta);
                buf[self.instr_offset..self.instr_offset + 4].copy_from_slice(&word.to_le_bytes());
            }
            RvFixupKind::La { rd } => {
                // delta is byte offset from AUIPC to label.
                let hi20 = ((delta as i64 + 0x800) >> 12) as i32;
                let lo12 = delta - (hi20 << 12);
                let auipc = ((hi20 as u32) << 12) | (rd << 7) | 0x17;
                let addi  = (((lo12 as u32) & 0xFFF) << 20) | (rd << 15) | (rd << 7) | 0x13;
                buf[self.instr_offset..self.instr_offset + 4].copy_from_slice(&auipc.to_le_bytes());
                buf[self.instr_offset + 4..self.instr_offset + 8].copy_from_slice(&addi.to_le_bytes());
            }
            RvFixupKind::BCond { rs1, rs2, funct3 } => {
                let word = encode_branch(*rs1, *rs2, *funct3, delta);
                buf[self.instr_offset..self.instr_offset + 4].copy_from_slice(&word.to_le_bytes());
            }
        }
    }
}

pub struct RvAsmWriter<L = NoLabel> {
    buf: Vec<u8>,
    labels: BTreeMap<L, usize>,
    pending_fixups: Vec<RvFixup<L>>,
}

impl<L> RvAsmWriter<L> {
    pub fn new() -> Self {
        Self { buf: Vec::new(), labels: BTreeMap::new(), pending_fixups: Vec::new() }
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

    fn emit(&mut self, inst: Inst) {
        let word = inst.encode_normal(Xlen::Rv64);
        self.buf.extend_from_slice(&word.to_le_bytes());
    }
}

impl<L> Default for RvAsmWriter<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L, Context> crate::out::WriterCore<Context> for RvAsmWriter<L> {
    type Error = core::convert::Infallible;

    fn ebreak(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch) -> Result<(), Self::Error> {
        self.emit(Inst::Ebreak);
        Ok(())
    }

    fn mv(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Addi {
            dest: to_rv_reg(dest),
            src1: to_rv_reg(src),
            imm: Imm::ZERO,
        });
        Ok(())
    }

    fn sd(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, src: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Sd { src: to_rv_reg(src), base, offset });
        Ok(())
    }

    fn ld(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Ld { dest: to_rv_reg(dest), base, offset });
        Ok(())
    }

    fn lw(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Lw { dest: to_rv_reg(dest), base, offset });
        Ok(())
    }

    fn sw(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, src: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Sw { src: to_rv_reg(src), base, offset });
        Ok(())
    }

    fn lb(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Lb { dest: to_rv_reg(dest), base, offset });
        Ok(())
    }

    fn sb(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, src: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Sb { src: to_rv_reg(src), base, offset });
        Ok(())
    }

    fn lh(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Lh { dest: to_rv_reg(dest), base, offset });
        Ok(())
    }

    fn sh(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, src: &(dyn MemArg + '_), mem: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        let (base, offset) = mem_base_offset(mem);
        self.emit(Inst::Sh { src: to_rv_reg(src), base, offset });
        Ok(())
    }

    fn jalr(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), base: &(dyn MemArg + '_), offset: i32) -> Result<(), Self::Error> {
        self.emit(Inst::Jalr {
            dest: to_rv_reg(dest),
            base: to_rv_reg(base),
            offset: Imm::new_i32(offset),
        });
        Ok(())
    }

    fn jal(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Jal {
            dest: to_rv_reg(dest),
            offset: lit_as_imm(target),
        });
        Ok(())
    }

    fn beq(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Beq { src1: to_rv_reg(a), src2: to_rv_reg(b), offset: lit_as_imm(target) });
        Ok(())
    }

    fn bne(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Bne { src1: to_rv_reg(a), src2: to_rv_reg(b), offset: lit_as_imm(target) });
        Ok(())
    }

    fn blt(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Blt { src1: to_rv_reg(a), src2: to_rv_reg(b), offset: lit_as_imm(target) });
        Ok(())
    }

    fn bge(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Bge { src1: to_rv_reg(a), src2: to_rv_reg(b), offset: lit_as_imm(target) });
        Ok(())
    }

    fn bltu(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Bltu { src1: to_rv_reg(a), src2: to_rv_reg(b), offset: lit_as_imm(target) });
        Ok(())
    }

    fn bgeu(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_), target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Bgeu { src1: to_rv_reg(a), src2: to_rv_reg(b), offset: lit_as_imm(target) });
        Ok(())
    }

    fn add(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Add { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn sub(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Sub { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn addi(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), src: &(dyn MemArg + '_), imm: i32) -> Result<(), Self::Error> {
        self.emit(Inst::Addi { dest: to_rv_reg(dest), src1: to_rv_reg(src), imm: Imm::new_i32(imm) });
        Ok(())
    }

    fn and(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::And { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn or(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Or { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn xor(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Xor { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn sll(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Sll { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn srl(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Srl { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn sra(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Sra { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn slt(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Slt { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn sltu(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Sltu { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn lui(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), imm: u32) -> Result<(), Self::Error> {
        self.emit(Inst::Lui { dest: to_rv_reg(dest), uimm: Imm::new_u32(imm << 12) });
        Ok(())
    }

    fn auipc(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), imm: u32) -> Result<(), Self::Error> {
        self.emit(Inst::Auipc { dest: to_rv_reg(dest), uimm: Imm::new_u32(imm << 12) });
        Ok(())
    }

    fn li(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error> {
        let rd = to_rv_reg(dest);
        let v64 = val as i64;

        // Fast path: fits in sign-extended 12-bit immediate.
        if v64 >= -2048 && v64 <= 2047 {
            self.emit(Inst::Addi { dest: rd, src1: RvReg::ZERO, imm: Imm::new_i32(v64 as i32) });
            return Ok(());
        }

        // Medium path: fits in sign-extended 32-bit (LUI + optional ADDI).
        if v64 == v64 as i32 as i64 {
            let hi = ((v64 as i32).wrapping_add(0x800) >> 12) as u32;
            let lo = v64 as i32 - ((hi as i32) << 12);
            self.emit(Inst::Lui { dest: rd, uimm: Imm::new_u32(hi << 12) });
            if lo != 0 {
                self.emit(Inst::Addi { dest: rd, src1: rd, imm: Imm::new_i32(lo) });
            }
            return Ok(());
        }

        // Full 64-bit: split into up to six 12-bit signed chunks (from LSB) with
        // carry propagation to compensate for ADDI sign-extension, then emit
        // ADDI/SLLI chains from MSB down to LSB.
        let mut chunks = [0i32; 6];
        let mut v = val;
        for chunk in chunks.iter_mut().take(5) {
            *chunk = (v & 0xFFF) as i32;
            v >>= 12;
        }
        chunks[5] = v as i32; // at most 4 bits remain for 64-bit input

        // Carry propagation: if a chunk's value would be sign-extended negative by
        // ADDI, compensate by adding 1 to the next higher chunk.
        for i in 0..5 {
            if chunks[i] >= 0x800 {
                chunks[i] -= 0x1000;
                chunks[i + 1] += 1;
            }
        }

        // Find highest non-zero chunk.
        let mut top = 5usize;
        while top > 0 && chunks[top] == 0 {
            top -= 1;
        }

        // Load top chunk into rd.
        self.emit(Inst::Addi { dest: rd, src1: RvReg::ZERO, imm: Imm::new_i32(chunks[top]) });

        // Shift left 12 bits and add each subsequent chunk from high to low.
        for i in (0..top).rev() {
            self.emit(Inst::Slli { dest: rd, src1: rd, imm: Imm::new_i32(12) });
            if chunks[i] != 0 {
                self.emit(Inst::Addi { dest: rd, src1: rd, imm: Imm::new_i32(chunks[i]) });
            }
        }

        Ok(())
    }

    fn ret(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch) -> Result<(), Self::Error> {
        self.emit(Inst::Jalr { dest: RvReg::ZERO, base: RvReg::RA, offset: Imm::ZERO });
        Ok(())
    }

    fn call(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Jalr { dest: RvReg::RA, base: to_rv_reg(target), offset: Imm::ZERO });
        Ok(())
    }

    fn j(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, target: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Jal { dest: RvReg::ZERO, offset: lit_as_imm(target) });
        Ok(())
    }

    fn mul(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Mul { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn mulh(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Mulh { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn div(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Div { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn divu(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Divu { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn rem(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Rem { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }

    fn remu(&mut self, _ctx: &mut Context, _cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error> {
        self.emit(Inst::Remu { dest: to_rv_reg(dest), src1: to_rv_reg(a), src2: to_rv_reg(b) });
        Ok(())
    }
}

// ── Writer implementation ────────────────────────────────────────────────────

impl<L: Ord, Context> crate::out::Writer<L, Context> for RvAsmWriter<L> {
    fn set_label(
        &mut self,
        _ctx: &mut Context,
        _cfg: crate::RiscV64Arch,
        s: L,
    ) -> Result<(), Self::Error> {
        let target = self.buf.len();
        let mut i = 0;
        while i < self.pending_fixups.len() {
            if self.pending_fixups[i].label == s {
                let fix = self.pending_fixups.swap_remove(i);
                fix.apply(&mut self.buf, target);
            } else {
                i += 1;
            }
        }
        self.labels.insert(s, target);
        Ok(())
    }

    fn jal_label(
        &mut self,
        _ctx: &mut Context,
        _cfg: crate::RiscV64Arch,
        dest: &(dyn crate::out::MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let rd = to_rv_reg(dest).0 as u32;
        let instr_offset = self.buf.len();
        if let Some(&target) = self.labels.get(&label) {
            let delta = (target as i64 - instr_offset as i64) as i32;
            self.buf.extend_from_slice(&encode_jal(rd, delta).to_le_bytes());
        } else {
            // Placeholder JAL rd, 0
            self.buf.extend_from_slice(&encode_jal(rd, 0).to_le_bytes());
            self.pending_fixups.push(RvFixup { instr_offset, label, kind: RvFixupKind::Jal { rd } });
        }
        Ok(())
    }

    fn la_label(
        &mut self,
        _ctx: &mut Context,
        _cfg: crate::RiscV64Arch,
        dest: &(dyn crate::out::MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let rd = to_rv_reg(dest).0 as u32;
        let instr_offset = self.buf.len();
        if let Some(&target) = self.labels.get(&label) {
            let delta = (target as i64 - instr_offset as i64) as i32;
            let hi20 = ((delta as i64 + 0x800) >> 12) as i32;
            let lo12 = delta - (hi20 << 12);
            let auipc = ((hi20 as u32) << 12) | (rd << 7) | 0x17;
            let addi  = (((lo12 as u32) & 0xFFF) << 20) | (rd << 15) | (rd << 7) | 0x13;
            self.buf.extend_from_slice(&auipc.to_le_bytes());
            self.buf.extend_from_slice(&addi.to_le_bytes());
        } else {
            // Placeholder: AUIPC rd, 0 + ADDI rd, rd, 0
            let auipc = (rd << 7) | 0x17;
            let addi  = (rd << 15) | (rd << 7) | 0x13;
            self.buf.extend_from_slice(&auipc.to_le_bytes());
            self.buf.extend_from_slice(&addi.to_le_bytes());
            self.pending_fixups.push(RvFixup { instr_offset, label, kind: RvFixupKind::La { rd } });
        }
        Ok(())
    }

    fn bcond_label(
        &mut self,
        _ctx: &mut Context,
        _cfg: crate::RiscV64Arch,
        cond: crate::ConditionCode,
        a: &(dyn crate::out::MemArg + '_),
        b: &(dyn crate::out::MemArg + '_),
        label: L,
    ) -> Result<(), Self::Error> {
        let (funct3, swap) = funct3_for_cond(cond);
        let (rs1_arg, rs2_arg) = (to_rv_reg(a).0 as u32, to_rv_reg(b).0 as u32);
        let (rs1, rs2) = if swap { (rs2_arg, rs1_arg) } else { (rs1_arg, rs2_arg) };
        let instr_offset = self.buf.len();
        if let Some(&target) = self.labels.get(&label) {
            let delta = (target as i64 - instr_offset as i64) as i32;
            self.buf.extend_from_slice(&encode_branch(rs1, rs2, funct3, delta).to_le_bytes());
        } else {
            self.buf.extend_from_slice(&encode_branch(rs1, rs2, funct3, 0).to_le_bytes());
            self.pending_fixups.push(RvFixup { instr_offset, label, kind: RvFixupKind::BCond { rs1, rs2, funct3 } });
        }
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
        let arch = crate::RiscV64Arch::default();
        let mut ctx = ();
        let mut w: RvAsmWriter<u32> = RvAsmWriter::new();

        // Emit two 4-byte instructions (EBREAK)
        w.emit(Inst::Ebreak);
        w.emit(Inst::Ebreak);
        assert_eq!(w.offset(), 8);

        // Record label 7 at offset 8
        w.set_label(&mut ctx, arch, 7u32).unwrap();

        // Emit one more instruction
        w.emit(Inst::Ebreak);

        let (bytes, labels) = w.into_parts();
        assert_eq!(bytes.len(), 12);
        assert_eq!(labels[&7u32], 8);
    }
}
