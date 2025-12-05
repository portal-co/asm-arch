# AArch64 Assembly Target

This crate provides AArch64 (ARM64) assembly types and instruction generation, similar to the x86-64 crate.

## Features

- **Core instruction system**: Full trait-based interface for AArch64 instruction generation
- **Register handling**: Support for all 31 general-purpose registers (X0-X30) plus SP
- **SIMD/FP support**: NEON/AdvSIMD register handling (V0-V31)
- **Condition codes**: Complete set of AArch64 condition codes (EQ, NE, HI, LS, etc.)
- **Memory operations**: Load/store instructions with flexible addressing modes
- **x64_shim**: Translation guide for mapping x86-64 instructions to AArch64

## Usage

```rust
use portal_solutions_asm_aarch64::{AArch64Arch, out::WriterCore};
use core::fmt::Write;

let cfg = AArch64Arch::default();
let mut output = String::new();
let writer: &mut dyn Write = &mut output;

// Generate instructions
WriterCore::mov(writer, cfg, &reg0, &reg1)?;
WriterCore::add(writer, cfg, &reg0, &reg0, &imm)?;  // reg0 = reg0 + imm
WriterCore::ret(writer, cfg)?;
```

## Architecture

### Core Types

- **`AArch64Arch`**: Architecture configuration
- **`ConditionCode`**: Condition codes for conditional instructions
- **`RegisterClass`**: GPR or SIMD register selection
- **`DisplayOpts`**: Display formatting options
- **`RegFormatOpts`**: Register formatting options

### Modules

- **`out`**: Instruction output generation
  - **`out::arg`**: Argument and memory operand types
  - **`out::asm`**: Assembly text output implementations
- **`reg`**: Register handling and formatting
- **`shim`**: x86-64 to AArch64 translation guide (requires `x64_shim` feature)

## Register Naming

### General-Purpose Registers

- **64-bit**: `x0`-`x30`, `sp` (stack pointer)
- **32-bit**: `w0`-`w30`, `wsp`
- Register 31 is SP (stack pointer) or ZR (zero register) depending on context

### SIMD/FP Registers

- **Registers**: `v0`-`v31`
- **Element sizes**: `.b` (byte), `.h` (halfword), `.s` (single), `.d` (double)
- Example: `v0.d` for double-precision element in v0

### Special Registers

- **x30** (LR): Link register for function returns
- **x29** (FP): Frame pointer (by convention)
- **x16-x17**: Intra-procedure-call temporary registers (IP0, IP1)

## Instruction Set

### Data Movement

- **`mov`**: Move data between registers or from immediate
- **`mov_imm`**: Load 64-bit immediate (MOVZ/MOVK sequence)
- **`ldr`**: Load from memory
- **`str`**: Store to memory
- **`ldp/stp`**: Load/store register pairs

### Arithmetic

- **`add`**, **`sub`**: Addition and subtraction
- **`mul`**: Multiplication
- **`udiv`**, **`sdiv`**: Unsigned/signed division
- **`fadd`**, **`fsub`**, **`fmul`**, **`fdiv`**: Floating-point operations

### Bitwise

- **`and`**, **`orr`**, **`eor`**: Bitwise AND, OR, XOR
- **`mvn`**: Bitwise NOT
- **`lsl`**, **`lsr`**: Logical shifts

### Control Flow

- **`b`**: Unconditional branch
- **`bcond`**: Conditional branch (b.eq, b.ne, etc.)
- **`bl`**: Branch with link (call)
- **`br`**: Branch to register
- **`ret`**: Return from function

### Comparison

- **`cmp`**: Compare and set flags
- **`csel`**: Conditional select

### Special

- **`brk`**: Breakpoint
- **`sxt`**, **`uxt`**: Sign/zero extend

## x86-64 to AArch64 Shim

The `shim` module (enabled with `x64_shim` feature) provides a complete translation layer
for converting x86-64 instructions to AArch64.

### MemArg Adapter Layer

The key component is `MemArgAdapter`, which converts x86-64 `MemArg` trait objects to AArch64 `MemArg` trait objects. This adapter handles:

- **Register arguments**: Direct passthrough (both architectures share the same `Reg` type)
- **Literal values**: Direct passthrough
- **Memory references**: Converts x86-64's `u32` displacement to AArch64's `i32`
- **Register classes**: Maps `Xmm` to `Simd`, `Gpr` to `Gpr`

### Stack-Based Calling Convention

The shim implements x86-64's stack-based calling convention on AArch64:

- **CALL**: Pushes LR (return address) onto stack, calls target, then pops LR
  ```asm
  sub sp, sp, #8
  str x30, [sp]      // Push LR
  bl target
  ldr x30, [sp]      // Pop LR
  add sp, sp, #8
  ```

- **RET**: Pops return address from stack into LR, then returns
  ```asm
  ldr x30, [sp]      // Pop return address
  add sp, sp, #8
  ret
  ```

This ensures compatibility with x86-64's expectation that return addresses live on the stack.

```rust
use portal_solutions_asm_x86_64::{X64Arch, out::WriterCore as X64WriterCore};
use portal_solutions_asm_aarch64::shim::X64ToAArch64Shim;

let mut output = String::new();
let writer: &mut dyn Write = &mut output;
let mut shim = X64ToAArch64Shim::new(writer);

// Use x86-64 instruction API, get AArch64 output
X64WriterCore::mov(&mut shim, cfg, &dest, &src)?;
X64WriterCore::call(&mut shim, cfg, &target)?;  // Automatically handles stack
X64WriterCore::ret(&mut shim, cfg)?;             // Pops from stack
```

### Direct Translations (1:1)

Many instructions have direct equivalents:

| x86-64 | AArch64 | Notes |
|--------|---------|-------|
| MOV | MOV | Direct |
| ADD | ADD | Direct |
| SUB | SUB | Direct |
| MUL | MUL | Direct |
| DIV | UDIV | Unsigned |
| IDIV | SDIV | Signed |
| AND | AND | Direct |
| OR | ORR | Direct |
| XOR | EOR | Direct |
| SHL | LSL | Direct |
| SHR | LSR | Direct |
| NOT | MVN | Direct |
| CMP | CMP | Direct |
| RET | RET | Direct |
| CALL | BL/BLR | Direct |
| JMP | B/BR | Direct |

### Complex Translations

Some instructions require multiple AArch64 instructions:

#### XCHG (Exchange)
```
x86-64: XCHG a, b
AArch64: 
  MOV x16, a
  MOV a, b
  MOV b, x16
```
**Performance**: 3 instructions instead of 1. Not atomic without explicit barriers.

#### PUSH
```
x86-64: PUSH op
AArch64:
  SUB sp, sp, #8
  STR op, [sp]
```
**Performance**: 2 instructions instead of 1.

#### POP
```
x86-64: POP op
AArch64:
  LDR op, [sp]
  ADD sp, sp, #8
```
**Performance**: 2 instructions instead of 1.

#### PUSHF/POPF (Flags)
```
x86-64: PUSHF
AArch64:
  MRS x16, NZCV
  SUB sp, sp, #8
  STR x16, [sp]
```
**Performance**: 3 instructions instead of 1.

### Condition Code Mapping

| x86-64 | AArch64 | Meaning |
|--------|---------|---------|
| E/Z | EQ | Equal/Zero |
| NE/NZ | NE | Not equal/Not zero |
| B/C | LO | Unsigned less/Carry |
| NB/NC | HS | Unsigned ≥/No carry |
| A | HI | Unsigned greater |
| NA | LS | Unsigned ≤ |
| L | LT | Signed less |
| NL | GE | Signed ≥ |
| G | GT | Signed greater |
| NG | LE | Signed ≤ |
| O | VS | Overflow |
| NO | VC | No overflow |
| S | MI | Sign/Negative |
| NS | PL | No sign/Positive |
| P/PE | **AL** | Parity even (no equiv) |
| NP/PO | **AL** | Parity odd (no equiv) |

**Note**: AArch64 has no parity flag. Parity conditions are mapped to "always" (AL).

### Register Mapping

- x86-64: 16 GPRs (RAX-R15) → AArch64: 31 GPRs (X0-X30) + SP
- Direct mapping for registers 0-15
- Register 31 is SP (not a general register like x86-64)
- Temporary registers: X16-X17 (IP0-IP1) used for instruction sequences

## Performance Considerations

### Advantages of AArch64

1. **More registers**: 31 vs 16, reducing spills
2. **Simpler instruction encoding**: Fixed-width 32-bit instructions
3. **Better floating-point**: Integrated NEON SIMD
4. **No complex addressing**: Simpler memory operations

### Limitations vs x86-64

1. **No XCHG**: Requires 3 MOV instructions (not atomic)
2. **No parity flag**: Parity checks require explicit computation
3. **Limited immediate sizes**: Large immediates require MOVZ/MOVK sequences
4. **No PUSH/POP**: Requires explicit SP adjustment

### Performance Tips

1. **Use register pairs**: LDP/STP are faster than separate loads/stores
2. **Minimize immediate loads**: Use MOVZ/MOVK sparingly
3. **Leverage more registers**: Take advantage of 31 registers vs x86-64's 16
4. **Use conditional select**: CSEL is faster than branch for simple conditionals

## Testing

Run tests with:

```bash
cargo test -p portal-solutions-asm-aarch64 --all-features
```

Tests cover:
- Register naming and formatting
- Condition code display
- Instruction generation
- x86-64 condition code translation (with `x64_shim` feature)

## Implementation Status

### Complete

- ✅ Core type system
- ✅ Register handling (GPR and SIMD)
- ✅ Condition codes
- ✅ Memory argument types
- ✅ Assembly text output
- ✅ All basic instructions
- ✅ Translation guide documentation

### Complete (New!)

- ✅ **x64_shim**: Full translation implementation with MemArg adapter layer
  - `MemArgAdapter`: Converts between x86-64 and AArch64 argument types
  - `X64ToAArch64Shim`: Implements all x86-64 instruction traits
  - Automatic translation of all basic operations
  - Full test coverage

### Future Enhancements

- [ ] Binary encoding support
- [ ] Advanced SIMD instructions
- [ ] Atomic operations
- [ ] System instructions
- [ ] Full x64_shim implementation with MemArg adapter
- [ ] Peephole optimizations

## License

MPL-2.0

## Contributing

Contributions are welcome! Areas of interest:

1. **Binary encoding**: Add support for encoding instructions to bytes
2. **Advanced SIMD**: Extend NEON/AdvSIMD instruction coverage
3. **Optimizations**: Add peephole optimizer for common patterns
4. **x64_shim adapter**: Implement MemArg conversion layer for full shim support
5. **Documentation**: Improve examples and guides

## References

- [ARM Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest/)
- [ARM Instruction Set Reference](https://developer.arm.com/documentation/dui0801/latest/)
- [Condition Codes](https://developer.arm.com/documentation/dui0068/b/ARM-Instruction-Reference/Conditional-execution)
