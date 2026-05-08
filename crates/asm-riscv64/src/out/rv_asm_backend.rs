extern crate alloc;

use alloc::vec::Vec;
use rv_asm::{Imm, Inst, Reg as RvReg, Xlen};

use crate::out::arg::{ArgKind, MemArgKind};
use crate::out::MemArg;

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
pub struct RvAsmWriter {
    buf: Vec<u8>,
}

impl RvAsmWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    fn emit(&mut self, inst: Inst) {
        let word = inst.encode_normal(Xlen::Rv64);
        self.buf.extend_from_slice(&word.to_le_bytes());
    }
}

impl Default for RvAsmWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl<Context> crate::out::WriterCore<Context> for RvAsmWriter {
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

    fn li(&mut self, _ctx: &mut Context, cfg: crate::RiscV64Arch, dest: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error> {
        let rd = to_rv_reg(dest);
        if val as i64 >= -2048 && val as i64 <= 2047 {
            self.emit(Inst::Addi { dest: rd, src1: RvReg::ZERO, imm: Imm::new_i32(val as i32) });
        } else {
            let hi = ((val as i32).wrapping_add(0x800) >> 12) as u32;
            let lo = (val as i32) - ((hi as i32) << 12);
            self.emit(Inst::Lui { dest: rd, uimm: Imm::new_u32(hi << 12) });
            self.emit(Inst::Addi { dest: rd, src1: rd, imm: Imm::new_i32(lo) });
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
