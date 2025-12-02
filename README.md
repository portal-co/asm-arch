# asm-arch

Assembly target types and data structures for code generation and manipulation.

## Overview

This workspace provides a collection of crates for working with assembly-level representations and code generation, primarily focused on x86-64 architecture support with extensible semantics and register allocation utilities.

## Crates

### portal-solutions-asm-x86-64

Core x86-64 assembly types and output generation.

- **Architecture configuration** (`X64Arch`): Configure x86-64 features like APX (Advanced Performance Extensions)
- **Register handling** (`reg`): Register formatting and display with support for different sizes (8/16/32/64-bit)
- **Condition codes** (`ConditionCode`): x86-64 condition codes for conditional instructions
- **Instruction output** (`out`): Traits and implementations for generating assembly output
  - `WriterCore`: Core trait for emitting individual instructions
  - `Writer`: Extended trait with label support
  - Argument types (`arg`): Memory and register operand representations

### portal-solutions-asm-semantics

Semantic representation of assembly operations.

- **Argument kinds** (`ArgKind`, `MemArgKind`): Represent fixed registers, slots, previous values, and literals
- **Semantic trees** (`Semantic`, `PredicateTree`): Represent conditional and computed values
- **Value operations** (`Val`): Binary operations, jumps, dereferences, and control flow

### portal-solutions-asm-regalloc

Register allocation utilities for stack-based virtual machines.

- **Register allocation** (`RegAlloc`): Manage register assignments with stack spilling
- **Frame tracking** (`RegAllocFrame`): Track register states (reserved, empty, stack, local)
- **Commands** (`Cmd`): Push, pop, and local variable operations

## Features

All crates support `no_std` environments. Enable the `alloc` feature for heap allocation support:

```toml
[dependencies]
portal-solutions-asm-x86-64 = { version = "0.1.0", features = ["alloc"] }
```

## License

MPL-2.0
