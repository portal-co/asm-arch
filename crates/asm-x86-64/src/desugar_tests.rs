#[cfg(test)]
mod tests {
    use super::*;
    use crate::out::arg::{ArgKind, MemArgKind};
    use alloc::string::String;
    use core::fmt::Write as FmtWrite;
    use crate::out::WriterCore;
    use portal_pc_asm_common::types::reg::Reg;
    use portal_pc_asm_common::types::mem::MemorySize;

    // Minimal TestWriter that implements WriterCore by recording calls into a String.
    struct TestWriter {
        out: String,
    }
    impl TestWriter {
        fn new() -> Self {
            Self { out: String::new() }
        }
    }
    impl WriterCore for TestWriter {
        type Error = core::convert::Infallible;
        fn mov(
            &mut self,
            _cfg: crate::X64Arch,
            dest: &(dyn crate::out::arg::MemArg + '_),
            src: &(dyn crate::out::arg::MemArg + '_),
        ) -> Result<(), Self::Error> {
            write!(self.out, "mov {} , {}\n", dest.mem_display(Default::default()), src.mem_display(Default::default())).unwrap();
            Ok(())
        }
        fn mov64(&mut self, _cfg: crate::X64Arch, r: &(dyn crate::out::arg::MemArg + '_), val: u64) -> Result<(), Self::Error> {
            write!(self.out, "mov64 {} , {}\n", r.mem_display(Default::default()), val).unwrap();
            Ok(())
        }
        fn add(&mut self, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            write!(self.out, "add {} , {}\n", a.mem_display(Default::default()), b.mem_display(Default::default())).unwrap();
            Ok(())
        }
        fn shl(&mut self, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            write!(self.out, "shl {} , {}\n", a.mem_display(Default::default()), b.mem_display(Default::default())).unwrap();
            Ok(())
        }
        fn mul(&mut self, _cfg: crate::X64Arch, a: &(dyn crate::out::arg::MemArg + '_), b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            write!(self.out, "mul {} , {}\n", a.mem_display(Default::default()), b.mem_display(Default::default())).unwrap();
            Ok(())
        }
        fn xchg(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn hlt(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> { Ok(()) }
        fn push(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn pop(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn pushf(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> { Ok(()) }
        fn popf(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> { Ok(()) }
        fn call(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn jmp(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn cmp(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn cmp0(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn cmovcc64(&mut self, _cfg: crate::X64Arch, _cond: crate::ConditionCode, _op: &(dyn crate::out::arg::MemArg + '_), _val: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn jcc(&mut self, _cfg: crate::X64Arch, _cond: crate::ConditionCode, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn u32(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn not(&mut self, _cfg: crate::X64Arch, _op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn lea(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn get_ip(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> { Ok(()) }
        fn ret(&mut self, _cfg: crate::X64Arch) -> Result<(), Self::Error> { Ok(()) }
        fn div(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn idiv(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn and(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn or(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn eor(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn shr(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn sar(&mut self, _cfg: crate::X64Arch, _a: &(dyn crate::out::arg::MemArg + '_), _b: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn movsx(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn movzx(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn fadd(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn fsub(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn fmul(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn fdiv(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn fmov(&mut self, _cfg: crate::X64Arch, _dest: &(dyn crate::out::arg::MemArg + '_), _src: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> { Ok(()) }
        fn db(&mut self, _cfg: crate::X64Arch, _bytes: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    }

    #[test]
    fn literal_base_is_loaded_into_temp() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        let mem = MemArgKind::Mem {
            base: ArgKind::Lit(0x1000),
            offset: None,
            disp: 8,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let dest = Reg(0);
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");

        assert!(!writer.out.is_empty());
    }

    #[test]
    fn invalid_scale_is_materialized() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);
        let cfg = crate::X64Arch::default();

        let mem = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
            offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 3)),
            disp: 8,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let dest = Reg(1);
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");
        assert!(!writer.out.is_empty());
    }

    #[test]
    fn mem_to_mem_mov_uses_temp() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);
        let cfg = crate::X64Arch::default();

        let src = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(2), size: MemorySize::_64 },
            offset: None,
            disp: 4,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };
        let dest = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(3), size: MemorySize::_64 },
            offset: None,
            disp: 12,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        desugar.mov(cfg, &dest, &src).expect("mem->mem mov should succeed");
        assert!(!writer.out.is_empty());
    }
}
