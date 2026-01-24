//! Argument types for instruction operands.
//!
//! This module defines types for representing instruction operands, including
//! registers, immediate values, and memory references for RISC-V 64-bit.

use portal_pc_asm_common::types::{mem::MemorySized, reg::Reg};

use super::*;
use crate::reg::{RegDisplay, RiscV64Reg};
use core::{
    convert::Infallible,
    fmt::{Display, Formatter},
    mem::transmute,
};
use typeid;

/// Represents a concrete argument kind (register or literal).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum ArgKind {
    /// A register with a specific size.
    Reg {
        /// The register.
        reg: Reg,
        /// The operand size.
        size: MemorySize,
    },
    /// A literal 64-bit value.
    Lit(u64),
}

impl ArgKind {
    /// Creates a displayable representation of this argument kind.
    pub fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        match self {
            ArgKind::Reg { reg, size } => ArgKindDisplay::Reg(RiscV64Reg::display(
                reg,
                RegFormatOpts::with_reg_class(opts.arch, *size, opts.reg_class),
            )),
            ArgKind::Lit(i) => ArgKindDisplay::Lit(*i),
        }
    }
}

/// Displayable representation of an argument kind.
#[non_exhaustive]
pub enum ArgKindDisplay {
    /// A register display.
    Reg(RegDisplay),
    /// A literal value.
    Lit(u64),
}

impl Display for ArgKindDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ArgKindDisplay::Reg(reg_display) => write!(f, "{reg_display}"),
            ArgKindDisplay::Lit(i) => write!(f, "{i}"), // RISC-V immediates don't use # prefix
        }
    }
}

impl Display for ArgKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.display(Default::default()))
    }
}

/// Represents a memory argument kind.
///
/// Can be either a direct operand or a memory reference with base and displacement.
/// RISC-V uses simple base+displacement addressing (no pre/post-indexing).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum MemArgKind<A = ArgKind> {
    /// A direct operand (not a memory reference).
    NoMem(A),
    /// A memory reference with base register and displacement.
    Mem {
        /// The base register.
        base: A,
        /// Optional scaled index (operand, scale factor) - for shim compatibility.
        offset: Option<(A, u32)>,
        /// Displacement added to the address (12-bit signed immediate in RISC-V).
        disp: i32,
        /// Size of the memory access.
        size: MemorySize,
        /// Register class for the memory access.
        reg_class: crate::RegisterClass,
    },
}

impl<A: Arg> MemArgKind<A> {
    /// Creates a displayable representation of this memory argument kind.
    pub fn display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        match self {
            MemArgKind::NoMem(a) => {
                // For non-memory operands, use the provided opts
                MemArgKind::NoMem(a.display(opts))
            }
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => {
                // For memory operands, force base and offset to be GPRs
                let gpr_opts = crate::DisplayOpts::new(opts.arch);
                MemArgKind::Mem {
                    base: base.display(gpr_opts),
                    offset: offset
                        .as_ref()
                        .map(|(a, scale)| (a.display(gpr_opts), *scale)),
                    disp: *disp,
                    size: *size,
                    reg_class: *reg_class,
                }
            }
        }
    }
}

impl<T: Display> Display for MemArgKind<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MemArgKind::NoMem(a) => write!(f, "{a}"),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size: _,
                reg_class: _,
            } => {
                // RISC-V uses disp(base) format
                // If there's a scaled offset, we'll need to handle it with extra instructions
                if let Some((off, scale)) = offset {
                    // This case needs special handling - indicated by offset presence
                    write!(f, "{}({}+{}<<{})", disp, base, off, scale)
                } else {
                    write!(f, "{}({})", disp, base)
                }
            }
        }
    }
}

impl<A> MemArgKind<A> {
    /// Returns a reference view of this memory argument kind.
    pub fn as_ref<'a>(&'a self) -> MemArgKind<&'a A> {
        match self {
            MemArgKind::NoMem(a) => MemArgKind::NoMem(a),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => MemArgKind::Mem {
                base,
                offset: offset.as_ref().map(|(a, b)| (a, *b)),
                disp: *disp,
                size: *size,
                reg_class: *reg_class,
            },
        }
    }

    /// Returns a mutable reference view of this memory argument kind.
    pub fn as_mut<'a>(&'a mut self) -> MemArgKind<&'a mut A> {
        match self {
            MemArgKind::NoMem(a) => MemArgKind::NoMem(a),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => MemArgKind::Mem {
                base,
                offset: offset.as_mut().map(|(a, b)| (a, *b)),
                disp: *disp,
                size: *size,
                reg_class: *reg_class,
            },
        }
    }

    /// Maps the argument type using the provided function.
    pub fn map<B, E>(
        self,
        go: &mut (dyn FnMut(A) -> Result<B, E> + '_),
    ) -> Result<MemArgKind<B>, E> {
        Ok(match self {
            MemArgKind::NoMem(a) => MemArgKind::NoMem(go(a)?),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => MemArgKind::Mem {
                base: go(base)?,
                offset: match offset {
                    None => None,
                    Some((a, b)) => Some((go(a)?, b)),
                },
                disp,
                size,
                reg_class,
            },
        })
    }
}

/// Trait for types that can be used as memory arguments.
///
/// Memory arguments can represent either direct operands or memory references.
pub trait MemArg {
    /// Invokes the callback with the memory argument kind.
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_));

    /// Returns the concrete memory argument kind.
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        let mut m = None;
        self.mem_kind(&mut |a| {
            m = Some(a.map(&mut |a| Ok::<_, Infallible>(a.kind())));
        });
        m.unwrap().unwrap()
    }

    /// Creates a displayable representation of this memory argument.
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        let mut m = None;
        self.mem_kind(&mut |a| {
            m = Some(a.display(opts));
        });
        m.unwrap()
    }

    /// Formats this memory argument.
    fn mem_fmt(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        write!(f, "{}", self.mem_display(opts))
    }
}

/// Trait for types that can be used as direct arguments (not memory references).
///
/// Direct arguments include registers and immediate values.
pub trait Arg: MemArg {
    /// Returns the argument kind for this argument.
    fn kind(&self) -> ArgKind;

    /// Creates a displayable representation of this argument.
    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        self.kind().display(opts)
    }

    /// Returns an iterator over the registers used by this argument.
    #[cfg(feature = "alloc")]
    fn regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        use core::iter::empty;

        match self.kind() {
            ArgKind::Reg { reg, size: _ } => ::alloc::boxed::Box::new([reg].into_iter()),
            ArgKind::Lit(_) => ::alloc::boxed::Box::new(empty()),
        }
    }
}

impl Arg for ArgKind {
    fn kind(&self) -> ArgKind {
        *self
    }
}

impl MemArg for ArgKind {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        go(MemArgKind::NoMem(self))
    }
}

impl<T: Arg + ?Sized> Arg for &'_ T {
    fn kind(&self) -> ArgKind {
        (&**self).kind()
    }

    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        (&**self).display(opts)
    }

    #[cfg(feature = "alloc")]
    fn regs<'a>(&'a self) -> alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        (&**self).regs()
    }
}

impl<T: MemArg + ?Sized> MemArg for &'_ T {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        (&**self).mem_kind(go);
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        (&**self).concrete_mem_kind()
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        (&**self).mem_display(opts)
    }
    fn mem_fmt(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        (&**self).mem_fmt(f, opts)
    }
}

impl<T: MemArg + ?Sized> MemArg for &'_ mut T {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        (&**self).mem_kind(go);
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        (&**self).concrete_mem_kind()
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        (&**self).mem_display(opts)
    }
    fn mem_fmt(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        (&**self).mem_fmt(f, opts)
    }
}

impl Arg for Reg {
    fn kind(&self) -> ArgKind {
        ArgKind::Reg {
            reg: *self,
            size: MemorySize::_64,
        }
    }

    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        ArgKindDisplay::Reg(RiscV64Reg::display(
            self,
            RegFormatOpts::with_reg_class(opts.arch, Default::default(), opts.reg_class),
        ))
    }

    #[cfg(feature = "alloc")]
    fn regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        ::alloc::boxed::Box::new([*self].into_iter())
    }
}

impl MemArg for Reg {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        go(MemArgKind::NoMem(self))
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        MemArgKind::NoMem(self.kind())
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        MemArgKind::NoMem(Arg::display(self, opts))
    }
    fn mem_fmt(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        write!(f, "{}", Arg::display(self, opts))
    }
}

// Implement Arg for primitive integer types
macro_rules! impl_arg_for_int {
    ($($ty:ty),*) => {
        $(
            impl Arg for $ty {
                fn kind(&self) -> ArgKind {
                    ArgKind::Lit(*self as u64)
                }
            }

            impl MemArg for $ty {
                fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
                    go(MemArgKind::NoMem(self))
                }
            }
        )*
    };
}

impl_arg_for_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

impl<T: Arg> Arg for MemorySized<T> {
    fn kind(&self) -> ArgKind {
        let MemorySized { value, size } = self;
        if typeid::of::<T>() == typeid::of::<Reg>() {
            ArgKind::Reg {
                reg: unsafe { transmute::<&T, &Reg>(value) }.clone(),
                size: size.clone(),
            }
        } else {
            match value.kind() {
                ArgKind::Reg { reg, size: _ } => ArgKind::Reg { reg, size: *size },
                a => a,
            }
        }
    }

    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        self.kind().display(opts)
    }

    #[cfg(feature = "alloc")]
    fn regs<'a>(&'a self) -> alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        self.value.regs()
    }
}

impl<T: Arg> MemArg for MemorySized<T> {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        self.value.mem_kind(go);
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        self.value.concrete_mem_kind()
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        self.value.mem_display(opts)
    }
    fn mem_fmt(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        self.value.mem_fmt(f, opts)
    }
}

impl<A: MemArg + Arg> MemArg for MemArgKind<A> {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        match self {
            MemArgKind::NoMem(a) => a.mem_kind(go),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => {
                let base_kind = base.kind();
                let offset_kind = offset.as_ref().map(|(a, scale)| (a.kind(), *scale));
                go(MemArgKind::Mem {
                    base: &base_kind,
                    offset: offset_kind
                        .as_ref()
                        .map(|(a, scale)| (a as &dyn Arg, *scale)),
                    disp: *disp,
                    size: *size,
                    reg_class: *reg_class,
                });
            }
        }
    }
}
