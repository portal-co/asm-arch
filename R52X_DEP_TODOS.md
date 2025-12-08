# asm-arch Changes Needed for r52x Migration

This document lists changes needed in [portal-co/asm-arch](https://github.com/portal-co/asm-arch) to complete the r52x RISC-V to x86-64 translator migration.

## Required Changes

### 1. Add `sar` (Arithmetic Shift Right) instruction

**Location:** `crates/asm-x86-64/src/out.rs` and `crates/asm-x86-64/src/out/asm.rs`

**Problem:** RISC-V `SRA`/`SRAI`/`SRAW`/`SRAIW` instructions require arithmetic (sign-preserving) right shift. Currently r52x uses `shr` as a workaround, which is incorrect for signed values.

**Implementation:**

Add to `WriterCore` trait in `out.rs`:
```rust
/// Emits a SAR (arithmetic shift right) instruction.
///
/// Shifts `a` right by `b` bits, preserving the sign bit.
#[track_caller]
fn sar(
    &mut self,
    _cfg: crate::X64Arch,
    _a: &(dyn MemArg + '_),
    _b: &(dyn MemArg + '_),
) -> Result<(), Self::Error> {
    todo!("sar instruction not implemented")
}
```

Add to `writers!` macro in `out/asm.rs`:
```rust
fn sar(&mut self, cfg: $crate::X64Arch, a: &(dyn $crate::out::arg::MemArg + '_), b: &(dyn $crate::out::arg::MemArg + '_)) -> $crate::__::core::result::Result<(), Self::Error>{
    let a = a.mem_display(cfg.into());
    let b = b.mem_display(cfg.into());
    $crate::__::core::write!(self,"sar {a},{b}\n")
}
```

Add to `writer_dispatch!` macro delegation.

---

### 2. Add `.db` / data embedding directive

**Location:** `crates/asm-x86-64/src/out.rs` and `crates/asm-x86-64/src/out/asm.rs`

**Problem:** r52x needs to embed RISC-V bytecode as data within the generated assembly. Currently the data label exists but no bytes are emitted.

**Implementation:**

Add to `WriterCore` trait:
```rust
/// Emits raw bytes as data.
///
/// Generates a `.byte` directive (or equivalent) for the given bytes.
#[track_caller]
fn db(&mut self, _cfg: crate::X64Arch, _bytes: &[u8]) -> Result<(), Self::Error> {
    todo!("db directive not implemented")
}
```

Add to `writers!` macro:
```rust
fn db(&mut self, _cfg: $crate::X64Arch, bytes: &[u8]) -> $crate::__::core::result::Result<(), Self::Error>{
    $crate::__::core::write!(self, ".byte ")?;
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            $crate::__::core::write!(self, ", ")?;
        }
        $crate::__::core::write!(self, "0x{:02x}", b)?;
    }
    $crate::__::core::write!(self, "\n")
}
```

---

### 3. Add `jmp_label` and `jcc_label` convenience methods (Optional)

**Location:** `crates/asm-x86-64/src/out.rs`

**Problem:** Currently jumping to a label requires loading it into a register first:
```rust
w.lea_label(cfg, &RDI, label)?;
w.jmp(cfg, &RDI)?;
```

This wastes a register and adds instructions. Direct label jumps would be cleaner.

**Implementation:**

Add to `Writer` trait:
```rust
/// Emits an unconditional jump to a label.
fn jmp_label(&mut self, cfg: crate::X64Arch, label: L) -> Result<(), Self::Error>;

/// Emits a conditional jump to a label.
fn jcc_label(&mut self, cfg: crate::X64Arch, cc: crate::ConditionCode, label: L) -> Result<(), Self::Error>;
```

Add to `writers!` macro:
```rust
fn jmp_label(&mut self, _cfg: $crate::X64Arch, label: L) -> $crate::__::core::result::Result<(), Self::Error> {
    $crate::__::core::write!(self, "jmp {label}\n")
}

fn jcc_label(&mut self, _cfg: $crate::X64Arch, cc: $crate::ConditionCode, label: L) -> $crate::__::core::result::Result<(), Self::Error> {
    $crate::__::core::write!(self, "j{cc} {label}\n")
}
```

---

## Priority

1. **High:** `sar` - Required for correct RISC-V signed shift emulation
2. **Medium:** `db` - Required for self-contained code generation
3. **Low:** `jmp_label`/`jcc_label` - Quality of life, current workaround works

## Testing

After implementing these changes, r52x can be updated to:

1. Replace `shr` with `sar` for `Sra*` instructions
2. Use `db` to embed RISC-V bytecode after the data label
3. Optionally simplify label jumps with direct methods
