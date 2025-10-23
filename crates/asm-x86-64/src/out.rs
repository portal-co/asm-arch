use core::error::Error;

use crate::{out::arg::Arg, *};
// use alloc::boxed::Box;
pub mod arg;
pub mod asm;
pub trait WriterCore {
    type Error: Error;

    fn xchg(
        &mut self,
        dest: &(dyn Arg + '_),
        src: &(dyn Arg + '_),
        mem: Option<isize>,
    ) -> Result<(), Self::Error>;
    fn mov(
        &mut self,
        dest: &(dyn Arg + '_),
        src: &(dyn Arg + '_),
        mem: Option<isize>,
    ) -> Result<(), Self::Error>;
    fn push(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn pop(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn call(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn jmp(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn cmp0(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn cmovz64(&mut self, op: &(dyn Arg + '_), val: u64) -> Result<(), Self::Error>;
    fn jz(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn u32(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn not(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn lea(
        &mut self,
        dest: &(dyn Arg + '_),
        src: &(dyn Arg + '_),
        offset: isize,
        off_reg: Option<(&(dyn Arg + '_), usize)>,
    ) -> Result<(), Self::Error>;

    fn get_ip(&mut self) -> Result<(), Self::Error>;
    fn ret(&mut self) -> Result<(), Self::Error>;
    fn mov64(&mut self, r: &(dyn Arg + '_), val: u64) -> Result<(), Self::Error>;
    fn mul(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn div(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn idiv(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn and(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn or(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn eor(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn shl(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
    fn shr(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>;
}
pub trait Writer<L>: WriterCore {
    fn set_label(&mut self, s: L) -> Result<(), Self::Error>;
    fn lea_label(&mut self, dest: &(dyn Arg + '_), label: L) -> Result<(), Self::Error>;
}
macro_rules! writer_dispatch {
    ($( [ $($t:tt)* ] [$($u:tt)*] $ty:ty => $e:ty [$l:ty]),*) => {
        const _: () = {
            $(
                impl<$($u)*> WriterCore for $ty{
                type Error = $e;
                    fn xchg(&mut self, dest: &(dyn Arg + '_), src: &(dyn Arg + '_), mem: Option<isize>) -> Result<(), Self::Error> {
                        WriterCore::xchg(&mut **self, dest, src, mem)
                    }
                    fn push(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error> {
                        WriterCore::push(&mut **self, op)
                    }
                    fn pop(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error> {
                        WriterCore::pop(&mut **self, op)
                    }
                    fn call(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::call(&mut **self,op)
                    }
                    fn jmp(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::jmp(&mut **self,op)
                    }
                    fn cmp0(&mut self, op: &(dyn Arg + '_)) -> Result<(),Self::Error>{
                        WriterCore::cmp0(&mut **self,op)
                    }
                    fn cmovz64(&mut self, op: &(dyn Arg + '_),val:u64) -> Result<(), Self::Error>{
                        WriterCore::cmovz64(&mut **self,op,val)
                    }
                    fn jz(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::jz(&mut **self,op)
                    }
                    fn lea(
                        &mut self,
                        dest: &(dyn Arg + '_),
                        src: &(dyn Arg + '_),
                        offset: isize,
                        off_reg: Option<(&(dyn Arg + '_), usize)>,
                    ) -> Result<(), Self::Error> {
                        WriterCore::lea(&mut **self, dest, src, offset, off_reg)
                    }

                    fn get_ip(&mut self) -> Result<(), Self::Error>{
                        WriterCore::get_ip(&mut **self)
                    }
                    fn ret(&mut self) -> Result<(), Self::Error>{
                        WriterCore::ret(&mut **self)
                    }
                    fn mov64(&mut self, r: &(dyn Arg + '_), val: u64) -> Result<(),Self::Error>{
                        WriterCore::mov64(&mut **self,r,val)
                    }
                    fn mov(&mut self, dest: &(dyn Arg + '_), src: &(dyn Arg + '_), mem: Option<isize>) -> Result<(), Self::Error>{
                        WriterCore::mov(&mut **self,dest,src,mem)
                    }
                    fn u32(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::u32(&mut **self,op)
                    }
                    fn not(&mut self, op: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::not(&mut **self,op)
                    }
                    fn mul(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::mul(&mut **self,a,b)
                    }
                    fn div(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::div(&mut **self,a,b)
                    }
                    fn idiv(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::idiv(&mut **self,a,b)
                    }
                    fn and(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::and(&mut **self,a,b)
                    }
                    fn or(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::or(&mut **self,a,b)
                    }
                    fn eor(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::eor(&mut **self,a,b)
                    }
                    fn shl(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::shl(&mut **self,a,b)
                    }
                    fn shr(&mut self, a: &(dyn Arg + '_), b: &(dyn Arg + '_)) -> Result<(), Self::Error>{
                        WriterCore::shr(&mut **self,a,b)
                    }
                }
                impl<$($t)*> Writer<$l> for $ty{

                fn set_label(&mut self, s: $l) -> Result<(), Self::Error> {
                    Writer::set_label(&mut **self, s)
                }
                  fn lea_label(&mut self, dest: &(dyn Arg + '_), label: $l) -> Result<(), Self::Error> {
                    Writer::lea_label(&mut **self, dest, label)
                }

            })*
        };
    };
}
writer_dispatch!(
    [ T: Writer<L> + ?Sized,L ] [T: WriterCore + ?Sized] &'_ mut T => T::Error [L]
    // [ T: Writer<L> + ?Sized,L ] Box<T> => T::Error [L]
);
#[cfg(feature = "alloc")]
writer_dispatch!(
    [ T: Writer<L> + ?Sized,L ] [T: WriterCore + ?Sized] ::alloc::boxed::Box<T> => T::Error [L]
);
