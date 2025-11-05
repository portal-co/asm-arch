#![no_std]
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub extern crate alloc;
#[doc(hidden)]
pub use core;
use core::ops::Deref;

use alloc::{boxed::Box, vec::Vec};
use portal_pc_asm_common::types::{Arith, Cmp, reg::Reg};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum ArgKind {
    FixedReg(Reg),
    Slot(u8),
    Previous(u32),
    Lit(u64),
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum MemArgKind {
    Arg(ArgKind),
    Deref { base: Arg },
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Arg<K = ArgKind> {
    pub kind: K,
    pub bit_start: u8,
    pub bit_length: BitLength,
}
pub type MemArg = Arg<MemArgKind>;
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Semantic<
    V: ValJust = Vec<(Val, BitLength)>,
    C: CagedPredicateTree<Val = V> = PredicateBox<V>,
    W: SemanticStore<Val = V, Cage = C> = Vec<(MemArg, PredicateTree<V, C>)>,
> {
    pub wrapped: W,
}
pub trait SemanticStore: Deref<Target = [(MemArg, PredicateTree<Self::Val, Self::Cage>)]> {
    type Val: ValJust;
    type Cage: CagedPredicateTree<Val = Self::Val>;
}
impl<
    V: ValJust,
    C: CagedPredicateTree<Val = V>,
    T: Deref<Target = [(MemArg, PredicateTree<V, C>)]> + ?Sized,
> SemanticStore for T
{
    type Val = V;

    type Cage = C;
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PredicateTree<
    V: ValJust = Vec<(Val, BitLength)>,
    C: CagedPredicateTree<Val = V> = PredicateBox<V>,
> {
    Just(V),
    Compare {
        left: Arg,
        op: Cmp,
        right: Arg,
        if_true: C,
        if_false: C,
    },
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PredicateBox<V: ValJust>(pub Box<PredicateTree<V>>);
impl<V: ValJust> Deref for PredicateBox<V> {
    type Target = PredicateTree<V>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PredicateRef<'a, V: ValJust>(pub &'a PredicateTree<V, Self>);
impl<'a, V: ValJust> Deref for PredicateRef<'a, V> {
    type Target = PredicateTree<V, Self>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
pub trait ValJust: Deref<Target = [(Val, BitLength)]> {}
impl<T: Deref<Target = [(Val, BitLength)]> + ?Sized> ValJust for T {}
pub trait CagedPredicateTree: Deref<Target = PredicateTree<Self::Val, Self>> + Sized {
    type Val: ValJust;
}
impl<V: ValJust, T: Deref<Target = PredicateTree<V, T>>> CagedPredicateTree for T {
    type Val = V;
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Val {
    Bin { left: Arg, op: Arith, right: Arg },
    Jmp { target: Arg },
    Just { value: Arg },
    Deref { mem: Arg, offset: Option<Arg> },
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct BitLength(pub u8);
