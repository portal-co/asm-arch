//! Register allocation utilities for stack-based virtual machines.
//!
//! This crate provides a register allocator that manages the mapping between
//! virtual stack values and physical registers, with support for spilling to
//! the native stack and local variables.
//!
//! # Features
//!
//! - `alloc`: Enables heap allocation support for dynamic collections
//!
//! # Overview
//!
//! The register allocator tracks which registers hold stack values, local
//! variables, or are free for use. It generates commands for pushing, popping,
//! and managing locals that can be interpreted by a code generator.

#![no_std]
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub extern crate alloc;
#[doc(hidden)]
pub use core;
use core::{mem::replace, ops::IndexMut};

/// A register target with kind information.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Target<K> {
    /// The register number.
    pub reg: u8,
    /// The kind of register (e.g., integer, float).
    pub kind: K,
}

/// The register allocator.
///
/// Manages register assignments for a stack-based virtual machine.
///
/// # Type Parameters
///
/// - `K`: The register kind type
/// - `N`: The number of registers per kind
/// - `I`: The frame storage type
pub struct RegAlloc<K, const N: usize, I> {
    /// The register frames, indexed by kind.
    pub frames: I,
    /// The current top-of-stack register, if any.
    pub tos: Option<Target<K>>,
}

/// The state of a register in a frame.
pub enum RegAllocFrame<K> {
    /// Register is reserved and cannot be used.
    Reserved,
    /// Register is empty and available.
    Empty,
    /// Register holds a stack element.
    Stack {
        /// The stack element.
        elem: StackElement<K>,
    },
    /// Register holds a local variable.
    Local(u32),
}

/// A stack element.
pub enum StackElement<K> {
    /// This element is above another stack element in the given register.
    Above(Target<K>),
    /// This element is at the native stack level.
    Native,
}

/// A command emitted by the register allocator.
pub enum Cmd<K> {
    /// Push a register to the native stack.
    Push(Target<K>),
    /// Pop from the native stack into a register.
    Pop(Target<K>),
    /// Load a local variable into a register.
    GetLocal {
        /// Destination register.
        dest: Target<K>,
        /// Local variable index.
        local: u32,
    },
    /// Store a register into a local variable.
    SetLocal {
        /// Source register.
        src: Target<K>,
        /// Local variable index.
        local: u32,
    },
}

/// Trait for types that have a length.
pub trait Length {
    /// Returns the number of elements.
    fn len(&self) -> usize;
}
impl<T> Length for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T, const N: usize> Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}
impl<
    K: Clone + Eq + TryFrom<usize>,
    const N: usize,
    I: IndexMut<K, Output = [RegAllocFrame<K>; N]> + Length,
> RegAlloc<K, N, I>
{
    fn evict(
        &mut self,
    ) -> Result<
        (Target<K>, impl Iterator<Item = Cmd<K>> + use<N, K, I>),
        <K as TryFrom<usize>>::Error,
    > {
        let mut i = 0;
        let mut c = None;
        loop {
            for j in 0..self.frames.len() {
                let f = &mut self.frames[K::try_from(j)?][i & ((N - 1) & 0xff)];

                match f {
                    RegAllocFrame::Reserved => {
                        i += 1;
                        continue;
                    }
                    RegAllocFrame::Empty => {
                        return Ok((
                            Target {
                                reg: i as u8,
                                kind: K::try_from(j)?,
                            },
                            c.into_iter(),
                        ));
                    }
                    RegAllocFrame::Stack { elem } => {
                        let StackElement::Native = elem else {
                            i += 1;
                            continue;
                        };
                        if let Some(t) = self.tos.as_ref() {
                            if *t
                                == (Target {
                                    reg: i as u8,
                                    kind: K::try_from(j)?,
                                })
                            {
                                i += 1;
                                continue;
                            }
                        }
                        c = Some(Cmd::Push(Target {
                            reg: i as u8,
                            kind: K::try_from(j)?,
                        }));
                        *f = RegAllocFrame::Empty;
                        for k in 0..self.frames.len() {
                            for f in self.frames[K::try_from(k)?].iter_mut() {
                                if let RegAllocFrame::Stack { elem } = f {
                                    if let StackElement::Above(v) = elem {
                                        if *v
                                            == (Target {
                                                reg: i as u8,
                                                kind: K::try_from(j)?,
                                            })
                                        {
                                            *elem = StackElement::Native;
                                        }
                                    }
                                }
                            }
                        }
                        return Ok((
                            Target {
                                reg: i as u8,
                                kind: K::try_from(j)?,
                            },
                            c.into_iter(),
                        ));
                    }
                    RegAllocFrame::Local(l) => {
                        c = Some(Cmd::SetLocal {
                            src: Target {
                                reg: i as u8,
                                kind: K::try_from(j)?,
                            },
                            local: *l,
                        });
                        *f = RegAllocFrame::Empty;
                        return Ok((
                            Target {
                                reg: i as u8,
                                kind: K::try_from(j)?,
                            },
                            c.into_iter(),
                        ));
                    }
                }
            }
        }
    }

    /// Pushes a new value onto the virtual stack.
    ///
    /// Allocates a register of the specified kind for the new value.
    /// Returns the allocated register number and any commands needed to make room.
    pub fn push(
        &mut self,
        k: K,
    ) -> Result<(u8, impl Iterator<Item = Cmd<K>>), <K as TryFrom<usize>>::Error> {
        let mut c = None;
        let mut e = None;
        loop {
            let mut i = 0;
            while let Some(a) = self.frames[k.clone()].get_mut(i) {
                if let RegAllocFrame::Empty = a {
                    *a = RegAllocFrame::Stack {
                        elem: match replace(
                            &mut self.tos,
                            Some(Target {
                                reg: i as u8,
                                kind: k.clone(),
                            }),
                        ) {
                            None => StackElement::Native,
                            Some(a) => StackElement::Above(a),
                        },
                    };
                    return Ok((i as u8, e.into_iter().flatten().chain(c.into_iter())));
                }
                i += 1;
                i = i & ((N - 1) & 0xff);
            }
            let i = self.tos.as_mut();
            if let Some(i) = i {
                let mut i3 = 0u8;
                let i2 = loop {
                    let f = &self.frames[i.kind.clone()][i.reg as usize & ((N - 1) & 0xff)];
                    match f {
                        RegAllocFrame::Stack { elem } => match elem {
                            StackElement::Above(a) => {
                                i.reg = a.reg;
                                i3 = i.reg;
                            }
                            StackElement::Native => {
                                break i;
                            }
                        },
                        _ => todo!(),
                    }
                };
                c = Some(Cmd::Push(i2.clone()));
                self.frames[i2.kind.clone()][i2.reg as usize] = RegAllocFrame::Empty;
                self.frames[i2.kind.clone()][i3 as usize] = RegAllocFrame::Stack {
                    elem: StackElement::Native,
                };
            } else {
                let (_, v) = self.evict()?;
                e = Some(v);
            }
        }
    }

    /// Pushes an existing register onto the virtual stack.
    ///
    /// Marks the specified register as holding a stack value.
    pub fn push_existing(&mut self, a: Target<K>) -> impl Iterator<Item = Cmd<K>> {
        let c: Option<Cmd<K>> = None;
        if let RegAllocFrame::Empty = &self.frames[a.kind.clone()][a.reg as usize] {
            self.frames[a.kind.clone()][a.reg as usize] = RegAllocFrame::Stack {
                elem: match replace(&mut self.tos, Some(a.clone())) {
                    None => StackElement::Native,
                    Some(a) => StackElement::Above(a.clone()),
                },
            };
            return c.into_iter();
        }
        todo!()
    }

    /// Pops a value from the virtual stack.
    ///
    /// Returns the register containing the value and any commands needed.
    pub fn pop(&mut self, kind: K) -> (Target<K>, impl Iterator<Item = Cmd<K>>) {
        let mut c = None;
        'a: loop {
            match self.tos.take() {
                Some(i) => {
                    let a = &mut self.frames[i.kind.clone()][i.reg as usize & ((N - 1) & 0xff)];
                    if let RegAllocFrame::Stack { elem } = replace(a, RegAllocFrame::Empty) {
                        self.tos = match elem {
                            StackElement::Above(v) => Some(v),
                            StackElement::Native => None,
                        };
                        return (i, c.into_iter());
                    }
                }
                None => {
                    let mut i = 0;
                    while let Some(a) = self.frames[kind.clone()].get_mut(i) {
                        if let RegAllocFrame::Empty = a {
                            c = Some(Cmd::Pop(Target {
                                reg: i as u8,
                                kind: kind.clone(),
                            }));
                            self.tos = Some(Target {
                                reg: i as u8,
                                kind: kind.clone(),
                            });
                            continue 'a;
                        }
                        i += 1;
                        i = i & ((N - 1) & 0xff);
                    }
                }
            }
        }
    }

    /// Pops a value from the virtual stack into a local variable.
    ///
    /// The value is stored in the specified local variable slot.
    pub fn pop_local(&mut self, kind: K, target: u32) -> impl Iterator<Item = Cmd<K>> {
        let mut c = None;
        'a: loop {
            match self.tos.take() {
                Some(i) => {
                    let a = &mut self.frames[i.kind][i.reg as usize];
                    if let RegAllocFrame::Stack { elem } = replace(a, RegAllocFrame::Local(target))
                    {
                        self.tos = match elem {
                            StackElement::Above(v) => Some(v),
                            StackElement::Native => None,
                        };
                        return c.into_iter();
                    }
                }
                None => {
                    let mut i = 0;
                    while let Some(a) = self.frames[kind.clone()].get_mut(i) {
                        if let RegAllocFrame::Empty = a {
                            c = Some(Cmd::Pop(Target {
                                reg: i as u8,
                                kind: kind.clone(),
                            }));
                            self.tos = Some(Target {
                                reg: i as u8,
                                kind: kind.clone(),
                            });
                            continue 'a;
                        }
                        i += 1;
                        i = i & ((N - 1) & 0xff);
                    }
                }
            }
        }
    }

    /// Pushes a local variable onto the virtual stack.
    ///
    /// Loads the value from the specified local variable slot.
    pub fn push_local(
        &mut self,
        kind: K,
        src: u32,
    ) -> Result<impl Iterator<Item = Cmd<K>>, <K as TryFrom<usize>>::Error> {
        let mut c = None;
        let mut e = None;
        'a: loop {
            let mut i = 0;
            while let Some(a) = self.frames[kind.clone()].get_mut(i) {
                if let RegAllocFrame::Local(l) = a {
                    if *l == src {
                        *a = RegAllocFrame::Stack {
                            elem: match replace(
                                &mut self.tos,
                                Some(Target {
                                    reg: i as u8,
                                    kind: kind.clone(),
                                }),
                            ) {
                                None => StackElement::Native,
                                Some(a) => StackElement::Above(a),
                            },
                        };
                        return Ok(e.into_iter().flatten().chain(c.into_iter()));
                    }
                }
                i += 1;
                i = i & ((N - 1) & 0xff);
            }
            i = 0;
            while let Some(a) = self.frames[kind.clone()].get_mut(i) {
                if let RegAllocFrame::Empty = a {
                    *a = RegAllocFrame::Local(src);
                    c = Some(Cmd::GetLocal {
                        dest: Target {
                            reg: i as u8,
                            kind: kind.clone(),
                        },
                        local: src,
                    });
                    continue 'a;
                }
                i += 1;
                i = i & ((N - 1) & 0xff);
            }
            let (_, v) = self.evict()?;
            e = Some(v);
        }
    }

    /// Flushes all registers to their backing stores.
    ///
    /// Emits commands to push stack values and store locals to their
    /// backing storage locations.
    pub fn flush(&mut self) -> impl Iterator<Item = Cmd<K>> {
        let mut i = 0u8;
        core::iter::from_fn(move || {
            for k in 0..self.frames.len() {
                let Ok(k) = K::try_from(k) else {
                    continue;
                };
                for _ in 0u8..=(((N - 1) & 0xff) as u8) {
                    let o = i;
                    i = i.wrapping_add(1);
                    match &self.frames[k.clone()][i as usize & ((N - 1) & 0xff)] {
                        RegAllocFrame::Reserved => {}
                        RegAllocFrame::Empty => {}
                        RegAllocFrame::Stack { elem } => match elem {
                            StackElement::Above(_) => {}
                            StackElement::Native => {
                                self.frames[k.clone()][i as usize] = RegAllocFrame::Empty;
                                for k2 in 0..self.frames.len() {
                                    let Ok(k2) = K::try_from(k2) else {
                                        continue;
                                    };
                                    for f in self.frames[k2.clone()].iter_mut() {
                                        if let RegAllocFrame::Stack { elem } = f {
                                            if let StackElement::Above(v) = elem {
                                                if v.reg == i && v.kind == k {
                                                    *elem = StackElement::Native;
                                                }
                                            }
                                        }
                                    }
                                }
                                if self.tos.as_ref().is_some_and(|a| a.reg == i && a.kind == k) {
                                    self.tos = None;
                                }
                                return Some(Cmd::Push(Target {
                                    reg: i,
                                    kind: k.clone(),
                                }));
                            }
                        },
                        RegAllocFrame::Local(l) => {
                            let l = *l;
                            self.frames[k.clone()][i as usize] = RegAllocFrame::Empty;
                            return Some(Cmd::SetLocal {
                                src: Target {
                                    reg: i,
                                    kind: k.clone(),
                                },
                                local: l,
                            });
                        }
                    }
                }
            }
            return None;
        })
    }
}
