# LFI Specification (Intel syntax)

Light-weight Fine-grained Isolation (LFI) is a sandboxing scheme for native machine
code. A static verifier checks that generated code satisfies the constraints below;
if it passes, the code cannot escape its sandbox at runtime regardless of what it
executes.

This document is a faithful translation of the authoritative Git source at
<https://github.com/lfi-project/lfi-specification> into Intel syntax. Where the
upstream spec uses GNU AT&T syntax, all examples here are re-expressed in Intel
(NASM-style) syntax. Numbering follows the upstream LaTeX document.

---

## System Components

1. **Verifier** — reads a block of machine code, returns accept or reject.
2. **Assembler** — produces machine code from LFI assembly source.
3. **Runtime** — loads and executes programs while maintaining memory isolation.

---

## x86-64 (LFI-x64)

### Bundle model

Code is divided into **32-byte bundles**. Every bundle starts at a 32-byte-aligned
address. An instruction must not span a bundle boundary. A *macroinstruction* is a
fixed sequence of consecutive instructions; the entire sequence must fit inside one
bundle.

Functions must begin at a bundle-aligned address (`.align 32` / `.p2align 5`).

### Reserved registers

| Register | Role | Constraint |
|----------|------|------------|
| `r14` | `rbase` — sandbox base address; also the GS segment base | **Never written** by sandboxed code |
| `r15` | Context register (optional; enabled with `--ctxreg`) | **Never written**; only certain `mov` reads allowed |
| `rsp` | Stack pointer | Only written via explicit *modsp* macroinstructions (see below) |

### Permitted memory operand forms

For all instructions **except** `lea`, every memory operand must be one of:

```asm
gs:[eX + N]              ; GS-segment with 32-bit base (eX = any 32-bit GPR exc. esp)
gs:[eX + eY*M + N]       ; GS-segment with 32-bit base and scaled index
gs:[eY*M + N]            ; GS-segment with index only (no base)
[rsp + N]                ; RSP-relative (64-bit base, no index)
[rip + N]                ; RIP-relative (64-bit base, no index)
[r14 + N]                ; r14-relative (64-bit base, no index)
```

The address-size in `gs:` forms must be 32-bit (enforced by using 32-bit register
names such as `eax`, `ecx`, `r11d`). Writing a 32-bit register automatically
zero-extends to 64-bit, which sandboxes the address within the 4 GiB GS-segment window.

`lea` is exempt and may use any memory operand form.

`fs` segment is **forbidden**.

### RSP modification macroinstructions (*modsp*)

`rsp` may only be modified by one of the following two-instruction pairs, which
together restore the sandbox-relative invariant:

```asm
; Pattern 1 – move into esp
mov esp, eX
add rsp, r14       ; or: lea rsp, [rsp + r14]

; Pattern 2 – add immediate to esp
add esp, N
add rsp, r14

; Pattern 3 – subtract immediate from esp
sub esp, N
add rsp, r14

; Pattern 4 – AND esp (e.g. stack alignment)
and esp, N
add rsp, r14

; Pattern 5 – LEA into esp
lea esp, [eX + N]
add rsp, r14
```

`or rsp, r14` is also accepted instead of `add rsp, r14` for the second instruction.

### Indirect-jump macroinstruction

Three consecutive instructions; must fit in one 32-byte bundle:

```asm
and eX, 0xffffffe0   ; mask register to bundle boundary (32-bit dest = zero-extend)
add rX, r14          ; add sandbox base  (or: or rX, r14)
jmp rX               ; indirect jump
```

where `eX` is the 32-bit alias of `rX`, and neither is `r14` or `rsp`.

### Indirect-call macroinstruction

The `call rX` instruction must land at a bundle boundary. NOPs are inserted between
the masking pair and the call to achieve this alignment:

```asm
and eX, 0xffffffe0
add rX, r14          ; or: or rX, r14
[nop ...]            ; zero or more NOP bytes to reach the next bundle boundary
call rX              ; must be at a bundle-aligned address
```

### Runtime-call macroinstruction (*rtcall*, replaces `ret`)

`ret` is **forbidden**. Functions return (and make runtime calls) via:

```asm
lea r11, [rip + 8]          ; r11 = address of the instruction after jmp
jmp qword ptr [r14 + N]     ; dispatch through the runtime table
                            ; N must be one of: 0, 8, 16, 24, -8, -16, -24, -32
```

The return address in `r11` must be the next instruction (displacement = size of the
`jmp` instruction) **or** a bundle-aligned address.

### Load macroinstruction (alternative sandboxed load)

Loads may also use this two-instruction form instead of the `gs:` form:

```asm
mov eX, eX                  ; zero-extend 32-bit address → 64-bit (sandbox)
mov rDST, [r14 + rX + N]    ; load with r14 as base, rX as index, scale=1
```

`rDST` may be any non-reserved GPR; `rX` must be the same register as the `mov eX, eX`.

### String instructions

`rep stosq`, `rep movsq`, and `rep cmpsq` are only permitted as part of the
following macroinstructions (the `lea` zeroes the upper 32 bits):

```asm
; rep stosq:
mov edi, edi                ; zero-extend rdi
lea rdi, [r14 + rdi]        ; add base
rep stosq

; rep movsq / rep cmpsq:
mov edi, edi
lea rdi, [r14 + rdi]
mov esi, esi
lea rsi, [r14 + rsi]
rep movsq                   ; (or rep cmpsq)
```

### Context register reads (optional; `--ctxreg` mode)

When `--ctxreg` is enabled, `r15` is the context register and may only be read via:

```asm
mov rX, qword ptr [r15 + 16]    ; only offset 16 (CTXREG_TP_OFFSET)
```

Writes to `r15` remain forbidden.

### Permitted ISA extensions

Base x86-64, CMOV, CX8, FPU, SSE, SSE2, CMPXCHG16B, POPCNT, SSE3, SSE4.1,
SSE4.2, BMI1, BMI2, LZCNT.

The `fs` segment, and all other segments not listed above, are forbidden.

### Assembler pseudo-ISA (LFI-x64 → x64 translations)

The LFI assembler accepts a restricted pseudo-ISA and expands each instruction to
its sandboxed form:

| LFI-x64 source | Expanded x64 output |
|----------------|---------------------|
| `jmp rX` (indirect) | `and eX, 0xffffffe0` / `add rX, r14` / `jmp rX` |
| `jmp [mem]` (indirect through memory) | `mov rT, [mem]` / `and eT, 0xffffffe0` / `add rT, r14` / `jmp rT` |
| `call rX` (indirect) | `and eX, 0xffffffe0` / `add rX, r14` / *(NOP padding)* / `call rX` |
| `call [mem]` (indirect through memory) | `mov rT, [mem]` / `and eT, 0xffffffe0` / `add rT, r14` / *(NOP padding)* / `call rT` |
| `mov [N(rX)], ...` (non-rsp, non-r14 base) | `mov gs:[N(eX)], ...` |
| `ret` | `lea r11, [rip+8]` / `jmp qword ptr [r14+0]` |

---

## AArch64 (LFI-Arm64)

### Reserved registers

| Register | Role | Constraint |
|----------|------|------------|
| `x27` | `rbase` — sandbox base address | **Never written** |
| `x28` | Address register | Only written via `add x28, x27, wN, uxtw` |
| `x30` | Return address | Only written via `add x30, x27, wN, uxtw` or `ldr x30, [x27, #i]` |
| `sp` | Stack pointer | Only written via `add sp, x27, wN, uxtw` (plus pre/post-index stores which implicitly update sp) |

`i` in `ldr x30, [x27, #i]` must satisfy `i % 8 == 0` and `|i| ≤ 32`.

### Permitted memory operand forms

```asm
[x28, #i]           ; address-register + immediate offset
[x28]               ; address-register only
[sp, #i]            ; stack-relative (immediate offset)
[sp, #i]!           ; pre-increment
[sp], #i            ; post-increment
[x27, wN, uxtw]     ; rbase + zero_extend(wN)
[x27, #i]           ; rbase + immediate (i % 8 == 0, |i| ≤ 32)
[x27, #i] (ldur)    ; rbase + negative immediate (|i| ≤ 32, unscaled)
```

### Control flow

| Instruction | Constraint |
|-------------|-----------|
| `br` | Must target `x28` only |
| `blr` | Must target `x28` or `x30` only |
| `ret` | Must target `x30` only |
| `b label` / `bl label` | Direct (immediate); target must be bundle-aligned |
| `cbz`, `cbnz`, `tbz`, `tbnz` | Conditional; target must be bundle-aligned |
| `b.cond` | Conditional; target must be bundle-aligned |

Forbidden: `msr`, `mrs` (system register access), `svc` (unless translated to rtcall).

### Assembler pseudo-ISA (LFI-Arm64 → Arm64 translations)

| LFI-Arm64 source | Expanded Arm64 output |
|------------------|-----------------------|
| `br xN` | `add x28, x27, wN, uxtw` / `br x28` |
| `blr xN` | `add x28, x27, wN, uxtw` / `blr x28` |
| `ret` | `ret` (x30 already contains sandboxed return address) |
| `ret xN` | `add x28, x27, wN, uxtw` / `ret x28` |
| `ldr x30, [...]` | `ldr xT, [...]` / `add x30, x27, wT, uxtw` |
| `mov sp, xN` | `add sp, x27, wN, uxtw` |
| `add sp, sp, #i` | `add xT, sp, #i` / `add sp, x27, wT, uxtw` |
| `sub sp, sp, #i` | `sub xT, sp, #i` / `add sp, x27, wT, uxtw` |
| `ldr ..., [xM]` | `ldr ..., [x27, wM, uxtw]` |
| `ldr ..., [xM, #i]` | `add xT, xM, #i` / `ldr ..., [x27, wT, uxtw]` |
| `ldr ..., [xM, #i]!` | `add xM, xM, #i` / `ldr ..., [x27, wM, uxtw]` |
| `ldr ..., [xM], #i` | `ldr ..., [x27, wM, uxtw]` / `add xM, xM, #i` |
| `ldX ..., [xM, #i]` | `add x28, x27, wM, uxtw` / `ldX ..., [x28, #i]` |
| `stX ..., [xM, #i]` | `add x28, x27, wM, uxtw` / `stX ..., [x28, #i]` |
| `mrs x0, tpidr_el0` | rtcall 1 |
| `msr tpidr_el0, x0` | rtcall 2 |
| `svc #0` | rtcall 0 |

---

## RISC-V 64 (LFI-RISC-V64)

The RISC-V 64 specification is in progress upstream. A verifier implementation
exists in `lfi-verifier` but the rule set is not yet stable. See
<https://github.com/lfi-project/lfi-verifier/tree/main/src/riscv64> for the
current implementation.

---

## Verifier API

```c
// From lfi-verifier/src/include/lfiv.h

enum LFIBoxType {
    LFI_BOX_FULL,    // all memory accesses sandboxed
    LFI_BOX_STORES,  // only store instructions sandboxed (loads unrestricted)
};

struct LFIVOptions {
    enum LFIBoxType box;
    bool no_bdd;     // disable BDD pre-filter for x86-64
    bool ctxreg;     // enable context register (r15 on x64, x25 on arm64)
    void (*err)(char *msg, size_t size);
};

bool lfiv_verify_arm64(char *code, size_t size, uintptr_t addr, struct LFIVOptions *opts);
bool lfiv_verify_x64  (char *code, size_t size, uintptr_t addr, struct LFIVOptions *opts);
bool lfiv_verify_riscv64(char *code, size_t size, uintptr_t addr, struct LFIVOptions *opts);
```

**CLI:**

```
lfi-verify [OPTIONS] <ELF-file>...

  -a, --arch=ARCH        x64 | arm64
  -s, --sandbox=TYPE     full (default) | stores
      --no-bdd           disable BDD filter (x64 only)
      --ctxreg           enable context register
```

The verifier reads all `PT_LOAD` segments with the execute flag set from a 64-bit
ELF binary and checks every 32-byte bundle.
