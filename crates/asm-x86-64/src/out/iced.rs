//! iced-x86 backend for assembling machine code.
#![allow(unused)]

#[cfg(feature = "iced")]
mod _inner {
    extern crate alloc;
    use alloc::collections::BTreeMap;
    use core::fmt::Display;

    use crate::out::{WriterCore, Writer};
    use crate::X64Arch;

    pub struct IcedX86Writer<L> {
        pub asm: iced_x86::code_asm::CodeAssembler,
        pub labels: BTreeMap<L, iced_x86::code_asm::CodeLabel>,
    }

    impl<L: Ord + Clone> IcedX86Writer<L> {
        pub fn new() -> Self {
            Self {
                asm: iced_x86::code_asm::CodeAssembler::new(64).expect("failed to create assembler"),
                labels: BTreeMap::new(),
            }
        }
    }

    use portal_pc_asm_common::types::{mem::MemorySize, reg::Reg};
    use crate::out::arg::{MemArgKind, ArgKind};

    // Helper: map our Reg to iced Register
    fn reg_to_register(r: Reg) -> Option<iced_x86::Register> {
        iced_x86::Register::try_from(r.0 as u32).ok()
    }

    // Helper: map our Reg to iced registers
    fn map_gpr64(r: Reg) -> Option<iced_x86::code_asm::reg::AsmRegister64> {
        reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr64::get_gpr64(reg))
    }
    fn map_gpr32(r: Reg) -> Option<iced_x86::code_asm::reg::AsmRegister32> {
        reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr32::get_gpr32(reg))
    }
    fn map_gpr16(r: Reg) -> Option<iced_x86::code_asm::reg::AsmRegister16> {
        reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr16::get_gpr16(reg))
    }
    fn map_gpr8(r: Reg) -> Option<iced_x86::code_asm::reg::AsmRegister8> {
        reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::gpr8::get_gpr8(reg))
    }
    fn map_xmm(r: Reg) -> Option<iced_x86::code_asm::reg::AsmRegisterXmm> {
        reg_to_register(r).and_then(|reg| iced_x86::code_asm::registers::xmm::get_xmm(reg))
    }

    // Helper to convert our MemArgKind to iced Operand
    fn to_iced_operand(&self, mem: &MemArgKind<ArgKind>) -> iced_x86::code_asm::Operand {
        match mem {
            MemArgKind::NoMem(ArgKind::Reg { reg, size }) => {
                match size {
                    MemorySize::_64 => map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax()).into(),
                    MemorySize::_32 => map_gpr32(*reg).unwrap_or(iced_x86::code_asm::registers::gpr32::eax()).into(),
                    MemorySize::_16 => map_gpr16(*reg).unwrap_or(iced_x86::code_asm::registers::gpr16::ax()).into(),
                    MemorySize::_8 => map_gpr8(*reg).unwrap_or(iced_x86::code_asm::registers::gpr8::al()).into(),
                    _ => map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax()).into(),
                }
            }
            MemArgKind::NoMem(ArgKind::Lit(v)) => (*v).into(),
            MemArgKind::Mem { base, offset, disp, size, reg_class } => {
                let base_r = if let ArgKind::Reg { reg, .. } = base {
                    map_gpr64(*reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax())
                } else {
                    iced_x86::code_asm::registers::gpr64::rax()
                };
                let mem_operand = if let Some((off, scale)) = offset {
                    if let ArgKind::Reg { reg: off_reg, .. } = off {
                        let off_r = map_gpr64(*off_reg).unwrap_or(iced_x86::code_asm::registers::gpr64::rax());
                        base_r + off_r * (*scale as i32) + (*disp as i32)
                    } else {
                        base_r + (*disp as i32)
                    }
                } else {
                    base_r + (*disp as i32)
                };
                match (size, reg_class) {
                    (MemorySize::_8, _) => iced_x86::code_asm::byte_ptr(mem_operand).into(),
                    (MemorySize::_16, _) => iced_x86::code_asm::word_ptr(mem_operand).into(),
                    (MemorySize::_32, _) => iced_x86::code_asm::dword_ptr(mem_operand).into(),
                    (MemorySize::_64, _) => iced_x86::code_asm::qword_ptr(mem_operand).into(),
                    (_, &crate::RegisterClass::Xmm) => iced_x86::code_asm::xmmword_ptr(mem_operand).into(),
                    _ => iced_x86::code_asm::qword_ptr(mem_operand).into(),
                }
            }
        }
    }

    impl<L: Ord + Clone + Display> Writer<L> for IcedX86Writer<L> {
        type Error = iced_x86::IcedError;
        fn set_label(&mut self, _cfg: X64Arch, s: L) -> Result<(), Self::Error> {
            let mut lbl = self.asm.create_label();
            self.labels.insert(s, lbl);
            self.asm.set_label(&mut lbl);
            Ok(())
        }

        fn lea_label(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), label: L) -> Result<(), Self::Error> {
            let mem = dest.concrete_mem_kind();
            let iced_dest = self.to_iced_operand(&mem);
            if let Some(&lbl) = self.labels.get(&label) {
                self.asm.lea(iced_dest, lbl)?;
            }
            Ok(())
        }

        fn jmp_label(&mut self, _cfg: X64Arch, label: L) -> Result<(), Self::Error> {
            if let Some(&lbl) = self.labels.get(&label) {
                self.asm.jmp(lbl)?;
            }
            Ok(())
        }

        fn jcc_label(&mut self, _cfg: X64Arch, cc: crate::ConditionCode, label: L) -> Result<(), Self::Error> {
            use crate::ConditionCode as CC;
            if let Some(&lbl) = self.labels.get(&label) {
                match cc {
                    CC::E => self.asm.je(lbl)?,
                    CC::NE => self.asm.jne(lbl)?,
                    CC::B => self.asm.jb(lbl)?,
                    CC::NB => self.asm.jnb(lbl)?,
                    CC::A => self.asm.ja(lbl)?,
                    CC::NA => self.asm.jna(lbl)?,
                    CC::L => self.asm.jl(lbl)?,
                    CC::NL => self.asm.jnl(lbl)?,
                    CC::G => self.asm.jg(lbl)?,
                    CC::NG => self.asm.jng(lbl)?,
                    CC::O => self.asm.jo(lbl)?,
                    CC::NO => self.asm.jno(lbl)?,
                    CC::S => self.asm.js(lbl)?,
                    CC::NS => self.asm.jns(lbl)?,
                    CC::P => self.asm.jp(lbl)?,
                    CC::NP => self.asm.jnp(lbl)?,
                    _ => self.asm.jmp(lbl)?,
                }
            }
            Ok(())
        }
    }

    // -- WriterCore instruction implementations --
    impl<L: Ord + Clone + Display> WriterCore for IcedX86Writer<L> {
        type Error = iced_x86::IcedError;

        fn hlt(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
            self.asm.hlt()?;
            Ok(())
        }

        fn xchg(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.xchg(iced_d, iced_s)?;
            Ok(())
        }

        fn mov(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.mov(iced_d, iced_s)?;
            Ok(())
        }

        fn sub(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.sub(iced_a, iced_b)?;
            Ok(())
        }

        fn add(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.add(iced_a, iced_b)?;
            Ok(())
        }

        fn movsx(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.movsx(iced_d, iced_s)?;
            Ok(())
        }

        fn movzx(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.movzx(iced_d, iced_s)?;
            Ok(())
        }

        fn push(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.push(iced_o)?;
            Ok(())
        }

        fn pop(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.pop(iced_o)?;
            Ok(())
        }

        fn pushf(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
            self.asm.pushf()?;
            Ok(())
        }

        fn popf(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
            self.asm.popf()?;
            Ok(())
        }

        fn call(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.call(iced_o)?;
            Ok(())
        }

        fn jmp(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.jmp(iced_o)?;
            Ok(())
        }

        fn cmp(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.cmp(iced_a, iced_b)?;
            Ok(())
        }

        fn cmp0(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.cmp(iced_o, 0u64)?;
            Ok(())
        }

        fn cmovcc64(&mut self, _cfg: X64Arch, cond: crate::ConditionCode, op: &(dyn crate::out::arg::MemArg + '_), val: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let a = op.concrete_mem_kind();
            let b = val.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&a);
            let iced_b = self.to_iced_operand(&b);
            // map cond to conditional move - use cmov<conds>
            use crate::ConditionCode as CC;
            match cond {
                CC::E => self.asm.cmove(iced_a, iced_b)?,
                CC::NE => self.asm.cmovne(iced_a, iced_b)?,
                CC::B => self.asm.cmovb(iced_a, iced_b)?,
                CC::NB => self.asm.cmovnb(iced_a, iced_b)?,
                CC::A => self.asm.cmova(iced_a, iced_b)?,
                CC::NA => self.asm.cmovna(iced_a, iced_b)?,
                CC::L => self.asm.cmovl(iced_a, iced_b)?,
                CC::NL => self.asm.cmovnl(iced_a, iced_b)?,
                CC::G => self.asm.cmovg(iced_a, iced_b)?,
                CC::NG => self.asm.cmovng(iced_a, iced_b)?,
                CC::O => self.asm.cmovo(iced_a, iced_b)?,
                CC::NO => self.asm.cmovno(iced_a, iced_b)?,
                CC::S => self.asm.cmovs(iced_a, iced_b)?,
                CC::NS => self.asm.cmovns(iced_a, iced_b)?,
                CC::P => self.asm.cmovp(iced_a, iced_b)?,
                CC::NP => self.asm.cmovnp(iced_a, iced_b)?,
            }
            Ok(())
        }

        fn jcc(&mut self, _cfg: X64Arch, cond: crate::ConditionCode, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            use crate::ConditionCode as CC;
            match cond {
                CC::E => self.asm.je(iced_o)?,
                CC::NE => self.asm.jne(iced_o)?,
                CC::B => self.asm.jb(iced_o)?,
                CC::NB => self.asm.jnb(iced_o)?,
                CC::A => self.asm.ja(iced_o)?,
                CC::NA => self.asm.jna(iced_o)?,
                CC::L => self.asm.jl(iced_o)?,
                CC::NL => self.asm.jnl(iced_o)?,
                CC::G => self.asm.jg(iced_o)?,
                CC::NG => self.asm.jng(iced_o)?,
                CC::O => self.asm.jo(iced_o)?,
                CC::NO => self.asm.jno(iced_o)?,
                CC::S => self.asm.js(iced_o)?,
                CC::NS => self.asm.jns(iced_o)?,
                CC::P => self.asm.jp(iced_o)?,
                CC::NP => self.asm.jnp(iced_o)?,
            }
            Ok(())
        }

        fn u32(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            // and op, 0xffffffff
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.and(iced_o, iced_o, 0xffffffffu64)?;
            Ok(())
        }

        fn not(&mut self, _cfg: X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let o = op.concrete_mem_kind();
            let iced_o = self.to_iced_operand(&o);
            self.asm.not(iced_o)?;
            Ok(())
        }

        fn lea(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.lea(iced_d, iced_s)?;
            Ok(())
        }

        fn get_ip(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
            // use call/pop trick: create label and lea into reg? For simplicity, emit call 1f; 1: ; but CodeAssembler supports call with label
            let mut lbl = self.asm.create_label();
            self.asm.call(lbl)?;
            self.asm.set_label(&mut lbl);
            Ok(())
        }

        fn ret(&mut self, _cfg: X64Arch) -> Result<(), Self::Error> {
            self.asm.ret()?;
            Ok(())
        }

        fn mov64(&mut self, _cfg: X64Arch, r: &(dyn crate::out::arg::MemArg + '_), val: u64) -> Result<(), Self::Error> {
            let reg_kind = r.concrete_mem_kind();
            let iced_r = self.to_iced_operand(&reg_kind);
            self.asm.mov(iced_r, val)?;
            Ok(())
        }

        fn mul(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.imul(iced_a, iced_b)?;
            Ok(())
        }

        fn div(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.idiv(iced_b)?;
            Ok(())
        }

        fn idiv(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.idiv(iced_b)?;
            Ok(())
        }

        fn and(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.and(iced_a, iced_a, iced_b)?;
            Ok(())
        }

        fn or(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.or(iced_a, iced_a, iced_b)?;
            Ok(())
        }

        fn eor(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.xor(iced_a, iced_a, iced_b)?;
            Ok(())
        }

        fn shl(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.shl(iced_a, iced_b)?;
            Ok(())
        }

        fn shr(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.shr(iced_a, iced_b)?;
            Ok(())
        }

        fn sar(&mut self, _cfg: X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let A = a.concrete_mem_kind();
            let B = b.concrete_mem_kind();
            let iced_a = self.to_iced_operand(&A);
            let iced_b = self.to_iced_operand(&B);
            self.asm.sar(iced_a, iced_b)?;
            Ok(())
        }

        fn fadd(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.addsd(iced_d, iced_s)?;
            Ok(())
        }

        fn fsub(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.subsd(iced_d, iced_s)?;
            Ok(())
        }

        fn fmul(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.mulsd(iced_d, iced_s)?;
            Ok(())
        }

        fn fdiv(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.divsd(iced_d, iced_s)?;
            Ok(())
        }

        fn fmov(&mut self, _cfg: X64Arch, dest: &(dyn crate::out::arg::MemArg + '_), src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            let d = dest.concrete_mem_kind();
            let s = src.concrete_mem_kind();
            let iced_d = self.to_iced_operand(&d);
            let iced_s = self.to_iced_operand(&s);
            self.asm.movsd(iced_d, iced_s)?;
            Ok(())
        }

        fn db(&mut self, _cfg: X64Arch, bytes: &[u8]) -> Result<(), Self::Error> {
            for &b in bytes {
                self.asm.byte(b)?;
            }
            Ok(())
        }
    }
    impl<L: Ord + Clone + Display> WriterCore for IcedX86Writer<L> {
        type Error = iced_x86::IcedError;
    }
}

#[cfg(feature = "iced")]
pub use _inner::IcedX86Writer;
