#![no_std]
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub extern crate alloc;
#[doc(hidden)]
pub use core;
use core::mem::replace;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Target {
    pub reg: u8,
    pub kind: usize,
}
pub struct RegAlloc<const N: usize> {
    pub frames: [RegAllocFrame; N],
    pub tos: Option<u8>,
}
pub enum RegAllocFrame {
    Reserved,
    Empty,
    Stack { elem: StackElement },
    Local(u32),
}
pub enum StackElement {
    Above(u8),
    Native,
}
pub enum Cmd {
    Push(u8),
    Pop(u8),
    GetLocal { dest: u8, local: u32 },
    SetLocal { src: u8, local: u32 },
}
impl<const N: usize> RegAlloc<N> {
    fn evict(&mut self) -> (u8, impl Iterator<Item = Cmd> + use<N>) {
        let mut i = 0;
        let mut c = None;
        loop {
            let f = &mut self.frames[(i & ((N - 1) & 0xff))];

            match f {
                RegAllocFrame::Reserved => {
                    i += 1;
                    continue;
                }
                RegAllocFrame::Empty => {
                    return (i as u8, c.into_iter());
                }
                RegAllocFrame::Stack { elem } => {
                    let StackElement::Native = elem else {
                        i += 1;
                        continue;
                    };
                    if let Some(t) = self.tos {
                        if t == i as u8 {
                            i += 1;
                            continue;
                        }
                    }
                    c = Some(Cmd::Push(i as u8));
                    *f = RegAllocFrame::Empty;
                    for f in self.frames.iter_mut() {
                        if let RegAllocFrame::Stack { elem } = f {
                            if let StackElement::Above(v) = elem {
                                if *v == i as u8 {
                                    *elem = StackElement::Native;
                                }
                            }
                        }
                    }
                    return (i as u8, c.into_iter());
                }
                RegAllocFrame::Local(l) => {
                    c = Some(Cmd::SetLocal {
                        src: i as u8,
                        local: *l,
                    });
                    *f = RegAllocFrame::Empty;
                    return (i as u8, c.into_iter());
                }
            }
        }
    }
    pub fn push(&mut self) -> (u8, impl Iterator<Item = Cmd>) {
        let mut c = None;
        let mut e = None;
        loop {
            let mut i = 0;
            while let Some(a) = self.frames.get_mut(i) {
                if let RegAllocFrame::Empty = a {
                    *a = RegAllocFrame::Stack {
                        elem: match replace(&mut self.tos, Some(i as u8)) {
                            None => StackElement::Native,
                            Some(a) => StackElement::Above(a),
                        },
                    };
                    return (i as u8, e.into_iter().flatten().chain(c.into_iter()));
                }
                i += 1;
                i = i & ((N - 1) & 0xff);
            }
            let mut i = self.tos;
            if let Some(mut i) = i {
                let mut i3 = 0u8;
                let i2 = loop {
                    let f = &self.frames[i as usize & ((N - 1) & 0xff)];
                    match f {
                        RegAllocFrame::Stack { elem } => match elem {
                            StackElement::Above(a) => {
                                i = *a;
                                i3 = i;
                            }
                            StackElement::Native => {
                                break i;
                            }
                        },
                        _ => todo!(),
                    }
                };
                c = Some(Cmd::Push(i2));
                self.frames[i2 as usize] = RegAllocFrame::Empty;
                self.frames[i3 as usize] = RegAllocFrame::Stack {
                    elem: StackElement::Native,
                };
            } else {
                let (_, v) = self.evict();
                e = Some(v);
            }
        }
    }
    pub fn push_existing(&mut self, a: u8) -> impl Iterator<Item = Cmd> {
        let mut c: Option<Cmd> = None;
        if let RegAllocFrame::Empty = &self.frames[a as usize] {
            self.frames[a as usize] = RegAllocFrame::Stack {
                elem: match replace(&mut self.tos, Some(a)) {
                    None => StackElement::Native,
                    Some(a) => StackElement::Above(a),
                },
            };
            return c.into_iter();
        }
        todo!()
    }
    pub fn pop(&mut self) -> (u8, impl Iterator<Item = Cmd>) {
        let mut c = None;
        'a: loop {
            match self.tos.take() {
                Some(i) => {
                    let a = &mut self.frames[i as usize & ((N - 1) & 0xff)];
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
                    while let Some(a) = self.frames.get_mut(i) {
                        if let RegAllocFrame::Empty = a {
                            c = Some(Cmd::Pop(i as u8));
                            self.tos = Some(i as u8);
                            continue 'a;
                        }
                        i += 1;
                        i = i & ((N - 1) & 0xff);
                    }
                }
            }
        }
    }
    pub fn pop_local(&mut self, target: u32) -> (impl Iterator<Item = Cmd>) {
        let mut c = None;
        'a: loop {
            match self.tos.take() {
                Some(i) => {
                    let a = &mut self.frames[i as usize];
                    if let RegAllocFrame::Stack { elem } = replace(a, RegAllocFrame::Local(target))
                    {
                        self.tos = match elem {
                            StackElement::Above(v) => Some(v),
                            StackElement::Native => None,
                        };
                        return (c.into_iter());
                    }
                }
                None => {
                    let mut i = 0;
                    while let Some(a) = self.frames.get_mut(i) {
                        if let RegAllocFrame::Empty = a {
                            c = Some(Cmd::Pop(i as u8));
                            self.tos = Some(i as u8);
                            continue 'a;
                        }
                        i += 1;
                        i = i & ((N - 1) & 0xff);
                    }
                }
            }
        }
    }
    pub fn push_local(&mut self, src: u32) -> impl Iterator<Item = Cmd> {
        let mut c = None;
        let mut e = None;
        'a: loop {
            let mut i = 0;
            while let Some(a) = self.frames.get_mut(i) {
                if let RegAllocFrame::Local(l) = a {
                    if *l == src {
                        *a = RegAllocFrame::Stack {
                            elem: match replace(&mut self.tos, Some(i as u8)) {
                                None => StackElement::Native,
                                Some(a) => StackElement::Above(a),
                            },
                        };
                        return e.into_iter().flatten().chain(c.into_iter());
                    }
                }
                i += 1;
                i = i & ((N - 1) & 0xff);
            }
            i = 0;
            while let Some(a) = self.frames.get_mut(i) {
                if let RegAllocFrame::Empty = a {
                    *a = RegAllocFrame::Local(src);
                    c = Some(Cmd::GetLocal {
                        dest: i as u8,
                        local: src,
                    });
                    continue 'a;
                }
                i += 1;
                i = i & ((N - 1) & 0xff);
            }
            let (_, v) = self.evict();
            e = Some(v);
        }
    }
    pub fn flush(&mut self) -> impl Iterator<Item = Cmd> {
        let mut i = 0u8;
        core::iter::from_fn(move || {
            for _ in 0u8..=(((N - 1) & 0xff) as u8) {
                let o = i;
                i = i.wrapping_add(1);
                match &self.frames[i as usize & ((N - 1) & 0xff)] {
                    RegAllocFrame::Reserved => {}
                    RegAllocFrame::Empty => {}
                    RegAllocFrame::Stack { elem } => match elem {
                        StackElement::Above(_) => {}
                        StackElement::Native => {
                            self.frames[i as usize] = RegAllocFrame::Empty;
                            for f in self.frames.iter_mut() {
                                if let RegAllocFrame::Stack { elem } = f {
                                    if let StackElement::Above(v) = elem {
                                        if *v == i {
                                            *elem = StackElement::Native;
                                        }
                                    }
                                }
                            }
                            if self.tos.is_some_and(|a| a == i) {
                                self.tos = None;
                            }
                            return Some(Cmd::Push(i));
                        }
                    },
                    RegAllocFrame::Local(l) => {
                        let l = *l;
                        self.frames[i as usize] = RegAllocFrame::Empty;
                        return Some(Cmd::SetLocal { src: i, local: l });
                    }
                }
            }
            return None;
        })
    }
}
