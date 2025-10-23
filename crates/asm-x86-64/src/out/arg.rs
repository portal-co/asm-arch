use portal_pc_asm_common::types::{mem::MemorySized, reg::Reg};

use super::*;
use crate::reg::{RegDisplay, X64Reg};
use core::{
    convert::Infallible,
    fmt::{Display, Formatter},
    mem::transmute,
};
// use portal_solutions_blitz_common::MemorySized;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum ArgKind {
    Reg { reg: Reg, size: MemorySize },
    Lit(u64),
}
impl ArgKind {
    pub fn display(&self, opts: X64Arch) -> ArgKindDisplay {
        match self {
            ArgKind::Reg { reg, size } => ArgKindDisplay::Reg(X64Reg::display(
                reg,
                RegFormatOpts::default_with_arch_and_size(opts, *size),
            )),
            ArgKind::Lit(i) => ArgKindDisplay::Lit(*i),
        }
    }
}
#[non_exhaustive]
pub enum ArgKindDisplay {
    Reg(RegDisplay),
    Lit(u64),
}
impl Display for ArgKindDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ArgKindDisplay::Reg(reg_display) => write!(f, "{reg_display}"),
            ArgKindDisplay::Lit(i) => write!(f, "{i}"),
        }
    }
}
impl Display for ArgKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.display(Default::default()))
    }
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum MemArgKind<A = ArgKind> {
    NoMem(A),
    Mem {
        base: A,
        offset: Option<(A, u32)>,
        disp: u32,
        size: MemorySize,
    },
}
impl<A: Arg> MemArgKind<A> {
    pub fn display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        return self
            .as_ref()
            .map(&mut |a| Ok::<_, Infallible>(a.display(opts)))
            .unwrap();
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
                size,
            } => {
                let ptr = match size {
                    MemorySize::_8 => "byte",
                    MemorySize::_16 => "word",
                    MemorySize::_32 => "dword",
                    MemorySize::_64 => "qword",
                };
                let c;
                let d;
                // let a = ;
                write!(
                    f,
                    "{ptr} ptr [{base}{}+{disp}]",
                    match offset.as_ref() {
                        None => format_args!(""),
                        Some((a, b)) => {
                            c = a;
                            d = b;
                            format_args!("+{c}*{d}")
                        }
                    }
                )
            }
        }
    }
}
impl<A> MemArgKind<A> {
    pub fn as_ref<'a>(&'a self) -> MemArgKind<&'a A> {
        match self {
            MemArgKind::NoMem(a) => MemArgKind::NoMem(a),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
            } => MemArgKind::Mem {
                base,
                offset: offset.as_ref().map(|(a, b)| (a, *b)),
                disp: *disp,
                size: *size,
            },
        }
    }
    pub fn as_mut<'a>(&'a mut self) -> MemArgKind<&'a mut A> {
        match self {
            MemArgKind::NoMem(a) => MemArgKind::NoMem(a),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
            } => MemArgKind::Mem {
                base,
                offset: offset.as_mut().map(|(a, b)| (a, *b)),
                disp: *disp,
                size: *size,
            },
        }
    }
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
            } => MemArgKind::Mem {
                base: go(base)?,
                offset: match offset {
                    None => None,
                    Some((a, b)) => Some((go(a)?, b)),
                },
                disp,
                size,
            },
        })
    }
}
pub trait MemArg {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_));
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        let mut m = None;
        self.mem_kind(&mut |a| {
            m = Some(a.map(&mut |a| Ok::<_, Infallible>(a.kind())));
            // Ok::<_,Infallible>(())
        });
        m.unwrap().unwrap()
    }
    fn mem_display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        let mut m = None;
        self.mem_kind(&mut |a| {
            m = Some(a.display(opts));
            // Ok::<_,Infallible>(())
        });
        m.unwrap()
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        write!(f, "{}", self.mem_display(opts))
    }
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        let mut m = None;
        self.mem_kind(&mut |a| {
            let regs = match a {
                MemArgKind::NoMem(a) => a.regs().collect::<::alloc::vec::Vec<_>>(),
                MemArgKind::Mem {
                    base,
                    offset,
                    disp,
                    size,
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
    fn mem_display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        (&**self).mem_display(opts)
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        (&**self).mem_format(f, opts)
    }
    #[cfg(feature = "alloc")]
    fn mem_regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        (&**self).mem_regs()
    }
}
pub trait Arg: MemArg {
    fn kind(&self) -> ArgKind;
    fn format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        write!(f, "{}", self.display(opts))
    }
    fn display(&self, opts: X64Arch) -> ArgKindDisplay {
        return self.kind().display(opts);
    }
    #[cfg(feature = "alloc")]
    fn regs<'a>(&'a self) -> ::alloc::boxed::Box<dyn Iterator<Item = Reg> + 'a> {
        use core::iter::empty;

        match self.kind() {
            ArgKind::Reg { reg, size } => ::alloc::boxed::Box::new([reg].into_iter()),
            ArgKind::Lit(_) => ::alloc::boxed::Box::new(empty()),
        }
    }
}
impl<T: Arg + ?Sized> Arg for &'_ T {
    fn format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        (&**self).format(f, opts)
    }

    fn display(&self, opts: X64Arch) -> ArgKindDisplay {
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
    fn display(&self, opts: X64Arch) -> ArgKindDisplay {
        ArgKindDisplay::Reg(X64Reg::display(
            self,
            RegFormatOpts::default_with_arch(opts),
        ))
    }
    fn format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        X64Reg::format(self, f, &RegFormatOpts::default_with_arch(opts))
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
    fn mem_display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        MemArgKind::NoMem(Arg::display(self, opts))
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
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
    fn display(&self, opts: X64Arch) -> ArgKindDisplay {
        let MemorySized { value, size } = self;
        if typeid::of::<T>() == typeid::of::<Reg>() {
            ArgKindDisplay::Reg(X64Reg::display(
                unsafe { transmute::<&T, &Reg>(value) },
                RegFormatOpts::default_with_arch_and_size(opts, *size),
            ))
        } else {
            self.kind().display(opts)
        }
    }
    fn format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        let MemorySized { value, size } = self;
        if typeid::of::<T>() == typeid::of::<Reg>() {
            X64Reg::format(
                unsafe { transmute::<&T, &Reg>(value) },
                f,
                &RegFormatOpts::default_with_arch_and_size(opts, *size),
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
                ArgKind::Reg { reg, size } => ArgKind::Reg {
                    reg,
                    size: self.size,
                },
                ArgKind::Lit(l) => ArgKind::Lit(l),
            }),
            MemArgKind::Mem {
                base,
                offset,
                disp,
                size,
            } => MemArgKind::Mem {
                base,
                offset,
                disp,
                size: self.size,
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
                size,
            } => go(MemArgKind::Mem {
                base,
                offset,
                disp,
                size: self.size,
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
    fn mem_display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        MemArgKind::NoMem(Arg::display(self, opts))
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
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
    fn display(&self, opts: X64Arch) -> ArgKindDisplay {
        ArgKindDisplay::Lit(*self)
    }
    fn format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
        write!(f, "{self}")
    }
}
impl MemArg for u64 {
    fn mem_kind(&self, go: &mut (dyn FnMut(MemArgKind<&'_ (dyn Arg + '_)>) + '_)) {
        go(MemArgKind::NoMem(self))
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        MemArgKind::NoMem(self.kind())
    }
    fn mem_display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        MemArgKind::NoMem(Arg::display(self, opts))
    }
    fn mem_format(&self, f: &mut Formatter<'_>, opts: X64Arch) -> core::fmt::Result {
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
            } => MemArgKind::Mem {
                base,
                offset: match offset.as_ref() {
                    None => None,
                    Some((a, b)) => Some((a, *b)),
                },
                disp: *disp,
                size: *size,
            },
        })
    }
    fn concrete_mem_kind(&self) -> MemArgKind<ArgKind> {
        self.as_ref()
            .map(&mut |a| Ok::<_, Infallible>(a.kind()))
            .unwrap()
    }
    fn mem_display(&self, opts: X64Arch) -> MemArgKind<ArgKindDisplay> {
        self.display(opts)
    }
}
