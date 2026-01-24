//! Semantic representation of assembly operations.
//!
//! This crate provides types for representing the semantics of assembly operations,
//! including arguments, memory references, conditional expressions, and value computations.
//!
//! # Features
//!
//! - `alloc`: Enables heap allocation support for dynamic collections

#![no_std]
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
#[doc(hidden)]
pub use core;
use core::ops::Deref;
use portal_pc_asm_common::types::{Arith, Cmp, reg::Reg};

/// Represents the kind of an argument.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum ArgKind {
    /// A fixed physical register.
    FixedReg(Reg),
    /// A stack slot.
    Slot(u8),
    /// Reference to a previous value.
    Previous(u32),
    /// A literal value.
    Lit(u64),
}

/// Represents the kind of a memory argument.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum MemArgKind {
    /// A direct argument.
    Arg(ArgKind),
    /// A memory dereference.
    Deref {
        /// The base argument to dereference.
        base: Arg,
    },
}

/// An argument with bit range information.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Arg<K = ArgKind> {
    /// The kind of argument.
    pub kind: K,
    /// Starting bit position within the value.
    pub bit_start: u8,
    /// Length of the bit range.
    pub bit_length: BitLength,
}

/// A memory argument (argument with memory argument kind).
pub type MemArg = Arg<MemArgKind>;

/// A semantic representation of instruction effects.
///
/// Contains a collection of memory locations and their computed values.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Semantic<
    V: ValJust = Vec<(Val, BitLength)>,
    C: CagedPredicateTree<Val = V> = PredicateBox<V>,
    W: SemanticStore<Val = V, Cage = C> = Vec<(MemArg, PredicateTree<V, C>)>,
> {
    /// The wrapped semantic store.
    pub wrapped: W,
}

/// Trait for types that can store semantic information.
pub trait SemanticStore: Deref<Target = [(MemArg, PredicateTree<Self::Val, Self::Cage>)]> {
    /// The value type stored in predicate trees.
    type Val: ValJust;
    /// The container type for predicate trees.
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

/// A conditional value tree.
///
/// Represents either a direct value or a conditional expression that
/// branches based on a comparison.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PredicateTree<
    V: ValJust = Vec<(Val, BitLength)>,
    C: CagedPredicateTree<Val = V> = PredicateBox<V>,
> {
    /// A direct value.
    Just(V),
    /// A conditional expression.
    Compare {
        /// Left operand of the comparison.
        left: Arg,
        /// Comparison operator.
        op: Cmp,
        /// Right operand of the comparison.
        right: Arg,
        /// Value if comparison is true.
        if_true: C,
        /// Value if comparison is false.
        if_false: C,
    },
}

/// A boxed predicate tree.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PredicateBox<V: ValJust>(pub Box<PredicateTree<V>>);

impl<V: ValJust> Deref for PredicateBox<V> {
    type Target = PredicateTree<V>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// A reference to a predicate tree.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PredicateRef<'a, V: ValJust>(pub &'a PredicateTree<V, Self>);

impl<'a, V: ValJust> Deref for PredicateRef<'a, V> {
    type Target = PredicateTree<V, Self>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// Trait for types that can hold value slices.
pub trait ValJust: Deref<Target = [(Val, BitLength)]> {}
impl<T: Deref<Target = [(Val, BitLength)]> + ?Sized> ValJust for T {}

/// Trait for types that contain predicate trees.
pub trait CagedPredicateTree: Deref<Target = PredicateTree<Self::Val, Self>> + Sized {
    /// The value type stored in the predicate tree.
    type Val: ValJust;
}
impl<V: ValJust, T: Deref<Target = PredicateTree<V, T>>> CagedPredicateTree for T {
    type Val = V;
}

/// A computed value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Val {
    /// A binary arithmetic operation.
    Bin {
        /// Left operand.
        left: Arg,
        /// Arithmetic operator.
        op: Arith,
        /// Right operand.
        right: Arg,
    },
    /// A jump to a target address.
    ///
    /// This always breaks out of the current machine instruction being lowered,
    /// transferring control to an external address.
    Jmp {
        /// Jump target.
        target: Arg,
    },
    /// A direct value.
    Just {
        /// The value.
        value: Arg,
    },
    /// A memory dereference.
    Deref {
        /// Memory address.
        mem: Arg,
        /// Optional offset.
        offset: Option<Arg>,
    },
    /// A backward jump within the lowered instruction's implementation.
    ///
    /// Unlike [`Jmp`](Val::Jmp), this does not break out of the current machine
    /// instruction. Instead, it jumps backwards within the instruction's lowered
    /// implementation to enable looping instructions such as x86-64 `rep` prefixed
    /// instructions.
    Rewind {
        /// Rewind target.
        target: Arg,
        /// Rewind limit.
        limit: u32,
    },
}

/// Bit length wrapper.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct BitLength(pub u8);
