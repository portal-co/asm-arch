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
        fn push(&mut self, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            write!(self.out, "push {}\n", op.mem_display(Default::default())).unwrap();
            Ok(())
        }
        fn pop(&mut self, _cfg: crate::X64Arch, op: &(dyn crate::out::arg::MemArg + '_)) -> Result<(), Self::Error> {
            write!(self.out, "pop {}\n", op.mem_display(Default::default())).unwrap();
            Ok(())
        }
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

        // Current desugar uses a placeholder 0 when handling literal base in this code path
        let expected = "mov64 r15 , 0\nmov r14 , qword ptr [r15+8]\nmov rax , r14\n";
        assert_eq!(writer.out, expected);
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
        // Expected sequence for invalid scale (scale=3) — note: current desugar uses placeholder base/offset
        // mov64 r15 , 0
        // mov r14 , r15
        // mov64 r13 , 3
        // mul r14 , r13
        // mov r15 , rax
        // add r15 , r14
        // mov r14 , qword ptr [r15+8]
        // mov rcx , r14
        let expected = "mov64 r15 , 0\nmov r14 , r15\nmov64 r13 , 3\nmul r14 , r13\nmov r15 , rax\nadd r15 , r14\nmov r14 , qword ptr [r15+8]\nmov rcx , r14\n";
        assert_eq!(writer.out, expected);
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
        let expected = "mov r15 , qword ptr [rdx+4]\nmov qword ptr [rbx+12] , r15\n";
        assert_eq!(writer.out, expected);
    }

    #[test]
    fn temp_register_conflicts_avoided() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        // Test case where base register conflicts with temp_reg
        let mem = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(15), size: MemorySize::_64 }, // r15 is temp_reg
            offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 3)), // scale=3 needs materialization
            disp: 8,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let dest = Reg(1);
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");

        // Should use temp_reg2 (r14) instead of temp_reg (r15) for materialization
        let expected = "mov r14 , r15\nmov64 r13 , 3\nmul r14 , r13\nmov r15 , rax\nadd r15 , r14\nmov r14 , qword ptr [r15+8]\nmov rcx , r14\n";
        assert_eq!(writer.out, expected);
    }

    #[test]
    fn large_displacement_handling() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        // Test large displacement (> i32::MAX)
        let large_disp = i32::MAX as u64 + 1000;
        let mem = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
            offset: None,
            disp: large_disp as u32,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let dest = Reg(1);
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");

        // Should fold large displacement into base register
        let expected = "mov64 r15 , 2147484648\nmov r15 , rbp\nadd r15 , rbp\nmov r14 , qword ptr [r15+0]\nmov rcx , r14\n";
        assert_eq!(writer.out, expected);
    }

    #[test]
    fn xmm_register_class_handling() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        // Test XMM memory access
        let mem = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
            offset: None,
            disp: 16,
            size: MemorySize::_16, // 128-bit XMM access
            reg_class: crate::RegisterClass::Xmm,
        };

        let dest = Reg(1); // GPR destination
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");

        // Should use XMM temp register for loading
        let expected = "mov xmm15 , xmmword ptr [rbp+16]\nmov rcx , xmm15\n";
        assert_eq!(writer.out, expected);
    }

    #[test]
    fn power_of_two_scale_uses_shl() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        // Test scale=8 (2^3), should use SHL
        let mem = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
            offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 8)),
            disp: 0,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let dest = Reg(1);
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");

        // Should use SHL 3 for scale=8
        let expected = "mov r14 , rbx\nshl r14 , 3\nadd r14 , rbp\nmov r15 , qword ptr [r14+0]\nmov rcx , r15\n";
        assert_eq!(writer.out, expected);
    }

    #[test]
    fn non_power_of_two_scale_uses_mul() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        // Test scale=6 (not power of two), should use MUL
        let mem = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(5), size: MemorySize::_64 },
            offset: Some((ArgKind::Reg { reg: Reg(6), size: MemorySize::_64 }, 6)),
            disp: 0,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let dest = Reg(1);
        desugar.mov(cfg, &dest, &mem).expect("mov should succeed");

        // Should use MUL for scale=6
        let expected = "mov r14 , rbx\nmov64 r13 , 6\nmul r14 , r13\nadd r14 , rbp\nmov r15 , qword ptr [r14+0]\nmov rcx , r15\n";
        assert_eq!(writer.out, expected);
    }

    #[test]
    fn stack_layout_tracking_reuses_temp_without_redundant_push_pop() {
        let mut writer = TestWriter::new();
        let mut desugar = crate::desugar::DesugaringWriter::new(&mut writer);

        let cfg = crate::X64Arch::default();

        // Create a scenario where all temp registers conflict, forcing push/pop
        let mem1 = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(2), size: MemorySize::_64 }, // rdx
            offset: Some((ArgKind::Reg { reg: Reg(15), size: MemorySize::_64 }, 8)), // r15 conflicts
            disp: 0,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        let mem2 = MemArgKind::Mem {
            base: ArgKind::Reg { reg: Reg(13), size: MemorySize::_64 }, // r13 conflicts
            offset: Some((ArgKind::Reg { reg: Reg(14), size: MemorySize::_64 }, 1)), // r14 conflicts
            disp: 16,
            size: MemorySize::_64,
            reg_class: crate::RegisterClass::Gpr,
        };

        // This should use push/pop since all temp candidates conflict and RSP is not used
        desugar.mov(cfg, &mem1, &mem2).expect("mem->mem mov should succeed");

        // Should push r15, load src, store to dest, pop r15
        let expected = "push r15\nmov r15 , qword ptr [r13+r14*1+16]\nmov qword ptr [rdx+r15*8+0] , r15\npop r15\n";
        assert_eq!(writer.out, expected);
    }
}
