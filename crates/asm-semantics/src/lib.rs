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
pub struct Arg {
    pub kind: ArgKind,
    pub bit_start: u8,
    pub bit_length: BitLength,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Semantic<
    V: ValJust = Vec<(Val, BitLength)>,
    C: CagedPredicateTree<V> = PredicateBox<V>,
    W: SemanticStore<Val = V, Cage = C> = Vec<(Arg, PredicateTree<V, C>)>,
> {
    pub wrapped: W,
}
pub trait SemanticStore: Deref<Target = [(Arg, PredicateTree<Self::Val, Self::Cage>)]> {
    type Val: ValJust;
    type Cage: CagedPredicateTree<Self::Val>;
}
impl<V: ValJust, C: CagedPredicateTree<V>, T: Deref<Target = [(Arg, PredicateTree<V, C>)]> + ?Sized>
    SemanticStore for T
{
    type Val = V;

    type Cage = C;
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PredicateTree<
    V: ValJust = Vec<(Val, BitLength)>,
    C: CagedPredicateTree<V> = PredicateBox<V>,
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
pub trait CagedPredicateTree<V: ValJust>: Deref<Target = PredicateTree<V, Self>> + Sized {}
impl<V: ValJust, T: Deref<Target = PredicateTree<V, T>>> CagedPredicateTree<V> for T {}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Val {
    Bin { left: Arg, op: Arith, right: Arg },
    Jmp { target: Arg },
    Just { value: Arg },
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct BitLength(pub u8);
