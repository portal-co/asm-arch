use core::error::Error;

use crate::{out::arg::{Arg,MemArg}, *};
// use alloc::boxed::Box;
pub mod arg;
pub mod asm;
pub trait WriterCore {
    type Error: Error;

    fn hlt(&mut self) -> Result<(), Self::Error>;
    fn xchg(
        &mut self,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
        mem: Option<isize>,
    ) -> Result<(), Self::Error>;
    fn mov(
        &mut self,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
        mem: Option<isize>,
    ) -> Result<(), Self::Error>;
    fn push(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn pop(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn call(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn jmp(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn cmp0(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn cmovz64(&mut self, op: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error>;
    fn jz(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn u32(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn not(&mut self, op: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn lea(
        &mut self,
        dest: &(dyn MemArg + '_),
        src: &(dyn MemArg + '_),
        offset: isize,
        off_reg: Option<(&(dyn MemArg + '_), usize)>,
    ) -> Result<(), Self::Error>;

    fn get_ip(&mut self) -> Result<(), Self::Error>;
    fn ret(&mut self) -> Result<(), Self::Error>;
    fn mov64(&mut self, r: &(dyn MemArg + '_), val: u64) -> Result<(), Self::Error>;
    fn mul(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn div(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn idiv(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn and(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn or(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn eor(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn shl(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
    fn shr(&mut self, a: &(dyn MemArg + '_), b: &(dyn MemArg + '_)) -> Result<(), Self::Error>;
}
pub trait Writer<L>: WriterCore {
    fn set_label(&mut self, s: L) -> Result<(), Self::Error>;
    fn lea_label(&mut self, dest: &(dyn MemArg + '_), label: L) -> Result<(), Self::Error>;
}
#[macro_export]
macro_rules! writer_dispatch {
    ($( [ $($t:tt)* ] [$($u:tt)*] $ty:ty => $e:ty [$l:ty]),*) => {
        const _: () = {
            $(
                impl<$($u)*> $crate::out::WriterCore for $ty{
                    type Error = $e;
                    fn hlt(&mut self) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::hlt(&mut **self)
                    }
                    fn xchg(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_), mem: $crate::__::core::option::Option<isize>) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::xchg(&mut **self, dest, src, mem)
                    }
                    fn push(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::push(&mut **self, op)
                    }
                    fn pop(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::pop(&mut **self, op)
                    }
                    fn call(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::call(&mut **self,op)
                    }
                    fn jmp(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::jmp(&mut **self,op)
                    }
                    fn cmp0(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::cmp0(&mut **self,op)
                    }
                    fn cmovz64(&mut self, op: &(dyn $crate::out::arg::MemArg + '_),val:u64) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::cmovz64(&mut **self,op,val)
                    }
                    fn jz(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::jz(&mut **self,op)
                    }
                    fn lea(
                        &mut self,
                        dest: &(dyn $crate::out::arg::MemArg + '_),
                        src: &(dyn $crate::out::arg::MemArg + '_),
                        offset: isize,
                        off_reg: $crate::__::core::option::Option<(&(dyn $crate::out::arg::MemArg + '_), usize)>,
                    ) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::WriterCore::lea(&mut **self, dest, src, offset, off_reg)
                    }

                    fn get_ip(&mut self) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::get_ip(&mut **self)
                    }
                    fn ret(&mut self) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::ret(&mut **self)
                    }
                    fn mov64(&mut self, r: &(dyn $crate::out::arg::MemArg + '_), val: u64) -> $crate::__::core::result::Result<(),Self::Error>{
                        $crate::out::WriterCore::mov64(&mut **self,r,val)
                    }
                    fn mov(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), src: &(dyn $crate::out::arg::MemArg + '_), mem: $crate::__::core::option::Option<isize>) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::mov(&mut **self,dest,src,mem)
                    }
                    fn u32(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::u32(&mut **self,op)
                    }
                    fn not(&mut self, op: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::not(&mut **self,op)
                    }
                    fn mul(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::mul(&mut **self,a,b)
                    }
                    fn div(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::div(&mut **self,a,b)
                    }
                    fn idiv(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::idiv(&mut **self,a,b)
                    }
                    fn and(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::and(&mut **self,a,b)
                    }
                    fn or(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::or(&mut **self,a,b)
                    }
                    fn eor(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::eor(&mut **self,a,b)
                    }
                    fn shl(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::shl(&mut **self,a,b)
                    }
                    fn shr(&mut self, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
                        $crate::out::WriterCore::shr(&mut **self,a,b)
                    }
                }
                impl<$($t)*>$crate::out:: Writer<$l> for $ty{

                    fn set_label(&mut self, s: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                        $crate::out::Writer::set_label(&mut **self, s)
                    }
                    fn lea_label(&mut self, dest: &(dyn $crate::out::arg::MemArg + '_), label: $l) -> $crate::__::core::result::Result<(), Self::Error> {
                       $crate::out:: Writer::lea_label(&mut **self, dest, label)
                    }

                }
            )*
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
