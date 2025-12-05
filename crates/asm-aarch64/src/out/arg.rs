//! Argument types for instruction operands.
//!
//! This module defines types for representing instruction operands, including
//! registers, immediate values, and memory references.

use portal_pc_asm_common::types::{mem::MemorySized, reg::Reg};

use super::*;
use crate::reg::{RegDisplay, AArch64Reg};
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
        size: MemorySize 
    },
    /// A literal 64-bit value.
    Lit(u64),
}

impl ArgKind {
    /// Creates a displayable representation of this argument kind.
    pub fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        match self {
            ArgKind::Reg { reg, size } => ArgKindDisplay::Reg(AArch64Reg::display(
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
            ArgKindDisplay::Lit(i) => write!(f, "#{i}"),  // AArch64 uses # for immediates
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
/// Can be either a direct operand or a memory reference with base, offset,
/// displacement, and size.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum MemArgKind<A = ArgKind> {
    /// A direct operand (not a memory reference).
    NoMem(A),
    /// A memory reference.
    Mem {
        /// The base operand.
        base: A,
        /// Optional scaled index (operand, scale factor).
        offset: Option<(A, u32)>,
        /// Displacement added to the address (signed for AArch64).
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
            MemArgKind::Mem { base, offset, disp, size, reg_class } => {
                // For memory operands, force base and offset to be GPRs
                let gpr_opts = crate::DisplayOpts::new(opts.arch);
                MemArgKind::Mem {
                    base: base.display(gpr_opts),
                    offset: offset.as_ref().map(|(a, scale)| (a.display(gpr_opts), *scale)),
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
                // AArch64 memory addressing: [base, #disp] or [base, offset, LSL #scale]
                write!(f, "[")?;
                write!(f, "{base}")?;
                
                if let Some((off, scale)) = offset {
                    write!(f, ", {off}")?;
                    if *scale > 0 {
                        write!(f, ", LSL #{scale}")?;
                    }
                }
                
                if *disp != 0 {
                    if *disp > 0 {
                        write!(f, ", #{disp}")?;
                    } else {
                        write!(f, ", #-{}", -disp)?;
                    }
                }
                
                write!(f, "]")
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
    fn mem_format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        write!(f, "{}", self.mem_display(opts))
    }
    
    /// Returns an iterator over the registers used by this memory argument.
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        let mut m = None;
        self.mem_kind(&mut |a| {
            let regs = match a {
                MemArgKind::NoMem(a) => a.regs().collect::<::alloc::vec::Vec<_>>(),
                MemArgKind::Mem {
                    base,
                    offset,
                    disp: _,
                    size: _,
                    reg_class: _,
                } => base
                    .regs()
                    .chain(offset.iter().flat_map(|(a, _)| a.regs()))
                    .collect(),
            };
            m = Some(alloc::boxed::Box::new(regs.into_iter()));
        });
        m.unwrap()
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
    fn mem_format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        (&**self).mem_format(f, opts)
    }
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        (&**self).mem_regs()
    }
}

/// Trait for types that can be used as direct instruction arguments.
///
/// This trait extends [`MemArg`] with methods specific to direct operands.
pub trait Arg: MemArg {
    /// Returns the concrete argument kind.
    fn kind(&self) -> ArgKind;
    
    /// Formats this argument.
    fn format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        write!(f, "{}", self.display(opts))
    }
    
    /// Creates a displayable representation of this argument.
    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        return self.kind().display(opts);
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

impl<T: Arg + ?Sized> Arg for &'_ T {
    fn format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        (&**self).format(f, opts)
    }

    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        (&**self).display(opts)
    }
    
    #[cfg(feature = "alloc")]
    fn regs<'a>(&'a self) -> alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        (&**self).regs()
    }

    fn kind(&self) -> ArgKind {
        (&**self).kind()
    }
}

impl Arg for Reg {
    fn kind(&self) -> ArgKind {
        ArgKind::Reg {
            reg: self.clone(),
            size: Default::default(),
        }
    }
    fn display(&self, opts: crate::DisplayOpts) -> ArgKindDisplay {
        ArgKindDisplay::Reg(AArch64Reg::display(
            self,
            RegFormatOpts::with_reg_class(opts.arch, Default::default(), opts.reg_class),
        ))
    }
    fn format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        AArch64Reg::format(self, f, &RegFormatOpts::with_reg_class(opts.arch, Default::default(), opts.reg_class))
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
    fn mem_format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        Arg::format(self, f, opts)
    }
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        Arg::regs(self)
    }
}

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
        let MemorySized { value, size } = self;
        if typeid::of::<T>() == typeid::of::<Reg>() {
            ArgKindDisplay::Reg(AArch64Reg::display(
                unsafe { transmute::<&T, &Reg>(value) },
                RegFormatOpts::with_reg_class(opts.arch, *size, opts.reg_class),
            ))
        } else {
            self.kind().display(opts)
        }
    }
    fn format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        let MemorySized { value, size } = self;
        if typeid::of::<T>() == typeid::of::<Reg>() {
            AArch64Reg::format(
                unsafe { transmute::<&T, &Reg>(value) },
                f,
                &RegFormatOpts::with_reg_class(opts.arch, *size, opts.reg_class),
            )
        } else {
            write!(f, "{}", self.display(opts))
        }
    }
}

impl<T: MemArg> MemArg for MemorySized<T> {
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        match self.value.concrete_mem_kind() {
            MemArgKind::NoMem(m) => MemArgKind::NoMem(match m {
                ArgKind::Reg { reg, size: _ } => ArgKind::Reg {
                    reg,
                    size: self.size,
                },
                ArgKind::Lit(l) => ArgKind::Lit(l),
            }),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size: _,
                reg_class,
            } => MemArgKind::Mem {
                base,
                offset,
                disp,
                size: self.size,
                reg_class,
            },
        }
    }
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        self.value.mem_kind(&mut |a| match a {
            MemArgKind::NoMem(n) => go(MemArgKind::NoMem(&MemorySized {
                value: n,
                size: self.size,
            })),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size: _,
                reg_class,
            } => go(MemArgKind::Mem {
                base,
                offset,
                disp,
                size: self.size,
                reg_class,
            }),
        });
    }
}

impl Arg for ArgKind {
    fn kind(&self) -> ArgKind {
        self.clone()
    }
}

impl MemArg for ArgKind {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        go(MemArgKind::NoMem(self))
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        MemArgKind::NoMem(self.kind())
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        MemArgKind::NoMem(Arg::display(self, opts))
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        Arg::format(self, f, opts)
    }
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        Arg::regs(self)
    }
}

impl Arg for u64 {
    fn kind(&self) -> ArgKind {
        ArgKind::Lit(*self)
    }
    fn display(&self, _opts: crate::DisplayOpts) -> ArgKindDisplay {
        ArgKindDisplay::Lit(*self)
    }
    fn format(&self, f: &mut Formatter<'_>, _opts: crate::DisplayOpts) -> core::fmt::Result {
        write!(f, "#{self}")  // AArch64 uses # for immediates
    }
}

impl MemArg for u64 {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        go(MemArgKind::NoMem(self))
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        MemArgKind::NoMem(self.kind())
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        MemArgKind::NoMem(Arg::display(self, opts))
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: crate::DisplayOpts) -> core::fmt::Result {
        Arg::format(self, f, opts)
    }
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        Arg::regs(self)
    }
}

impl<A: Arg> MemArg for MemArgKind<A> {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        go(match self {
            MemArgKind::NoMem(a) => MemArgKind::NoMem(a),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
                reg_class,
            } => MemArgKind::Mem {
                base,
                offset: match offset.as_ref() {
                    None => None,
                    Some((a, b)) => Some((a, *b)),
                },
                disp: *disp,
                size: *size,
                reg_class: *reg_class,
            },
        })
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        self.as_ref()
            .map(&mut |a| Ok::<_, Infallible>(a.kind()))
            .unwrap()
    }
    fn mem_display(&self, opts: crate::DisplayOpts) -> MemArgKind<ArgKindDisplay> {
        self.display(opts)
    }
}
