use super::*;
use core::fmt::{Display, Formatter, Write};
#[macro_export]
macro_rules! writers {
    ($($ty:ty),*) => {
        const _: () = {
            $(
            impl $crate::out::WriterCore for $ty{
                type Error = $crate::__::core::fmt::Error;
                fn hlt(&mut self) -> $crate::__::core::result::Result<(),Self::Error>{
                    $crate::__::core::write!(self,"hlt\n")
                }
                fn xchg(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_), mem: $crate::__::core::option::Option<isize>) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display($crate::X64Arch::default());
                    let src = src.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"xchg {dest}, ")?;
                    match mem{
                        None => $crate::__::core::write!(self,"{src}\n"),
                        Some(i) => $crate::__::core::write!(self,"qword ptr [{src}+{i}]\n")
                    }
                }
                fn push(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"push {op}\n")
                }
                fn pop(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"pop {op}\n")
                }
                fn call(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"call {op}\n")
                }
                 fn jmp(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"jmp {op}\n")
                }
                fn cmp0(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"cmp {op}, 0\n")
                }
                fn cmovcc64(&mut self,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_),val:&(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                     let op = op.mem_display($crate::X64Arch::default());
                     let val = val.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"cmov{cc} {op}, {val}\n")
                }
                fn jcc(&mut self,cc: $crate::ConditionCode, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"j{cc} {op}\n")
                }
                fn u32(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"and {op}, 0xffffffff\n")
                }
                fn lea(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_), offset: isize, off_reg: $crate::__::core::option::Option<(&(dyn $crate::out::arg::MemArg + '_),usize)>) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display($crate::X64Arch::default());
                    let src = src.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"lea {dest}, [{src}")?;
                    if let Some((r,m)) = off_reg{
                        let r = r.mem_display($crate::X64Arch::default());
                        $crate::__::core::write!(self,"+{r}*{m}")?;
                    }
                    $crate::__::core::write!(self,"+{offset}]\n")
                }
                fn mov(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_), mem: $crate::__::core::option::Option<isize>) -> $crate::__::core::result::Result<(), Self::Error>{
                     let dest = dest.mem_display($crate::X64Arch::default());
                    let src = src.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"mov {dest}, ")?;
                    match mem{
                        None => $crate::__::core::write!(self,"{src}\n"),
                        Some(i) => $crate::__::core::write!(self,"qword ptr [{src}+{i}]\n")
                    }
                }

                fn get_ip(&mut self) -> $crate::__::core::result::Result<(),Self::Error>{
                //   let dest = dest.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"call 1f\n1:\n")
                }
                fn ret(&mut self) -> $crate::__::core::result::Result<(), Self::Error>{
                    $crate::__::core::write!(self,"ret\n")
                }
                fn mov64(&mut self, r: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                    let r = r.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"mov {r}, {val}\n")
                }
                fn not(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let op = op.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"not {op}\n")
                }
                fn mul(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"mul {a},{b}\n")
                }
                fn div(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"div {a},{b}\n")
                }
                fn idiv(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"idiv {a},{b}\n")
                }
                fn and(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"and {a},{b}\n")
                }
                fn or(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"or {a},{b}\n")
                }
                fn eor(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"eor {a},{b}\n")
                }
                fn shl(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"shl {a},{b}\n")
                }
                fn shr(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                    let a = a.mem_display($crate::X64Arch::default());
                    let b = b.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"shr {a},{b}\n")
                }
            }
            impl<L: Display> Writer<L> for $ty {
                 fn set_label(&mut self, s: L) -> $crate::__::core::result::Result<(), Self::Error> {
                    $crate::__::core::write!(self, "{s}:\n")
                }
                 fn lea_label(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), label: L) -> $crate::__::core::result::Result<(),Self::Error>{
                    let dest = dest.mem_display($crate::X64Arch::default());
                    $crate::__::core::write!(self,"lea {dest}, {label}\n")
                }

            })*
        };
    };
}
writers!(Formatter<'_>, (dyn Write + '_));
