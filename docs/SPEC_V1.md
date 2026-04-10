# bfc Language and Toolchain Specification (V1 Draft)

Status: Draft (learning-oriented compiler profile)

## 1. Scope and Non-Goals

This document specifies the first stable profile of `bfc`.

Goals:
- Define one deterministic language/runtime behavior profile.
- Support multi-file input by front-end merge into one compilation unit.
- Produce either executable or library artifacts from merged input.

Non-goals for V1:
- Full industrial linker features.
- User-authored raw addresses in source.
- Function-call syntax and calling convention in the language core.
- Dynamic library output implementation (kept as TODO interface).

## 2. Input Model

`bfc` accepts one or more source files.

Each source file may contain:
- A `plt` section (symbol declarations/imports).
- A `text` section (function definitions).

Source-level addresses are forbidden. Symbols are identified by names; final addresses are assigned by object format/linker.

## 3. Merge Model (Front-End Link)

All input files are merged into one global compilation unit before code generation.

### 3.1 Symbol Rules

- Duplicate function definitions with same name: compile error.
- Duplicate declarations with incompatible signatures: compile error.
- Duplicate declarations with same signatures: allowed (deduplicated).
- Reference to undefined symbol: compile error.

### 3.2 Diagnostics

Diagnostics MUST include file and position when available.

## 4. Entry Point and Artifact Selection

Entry point is identified by annotation `@entry` on a function definition.

Rules after merge:
- `0` entry points: compile as library.
- `1` entry point: compile as executable.
- `>1` entry points: compile error.

No hardcoded function name (such as `main`) is required.

## 5. Artifact Policy

V1 output policy:
- Library case (`0` entry): output a static library (`.a`) only.
- Dynamic library output is not part of V1. If requested, the compiler MUST emit `E_ARTIFACT_KIND_UNSUPPORTED`.
- Executable case (`1` entry): output an executable binary.

## 6. Core Language Semantics

## 6.1 Cell Type and Arithmetic

V1 fixed cell semantics:
- Cell type: signed 64-bit integer (`i64`).
- Arithmetic: two's-complement wrapping semantics (modulo `2^64` bit-pattern behavior).
- Arithmetic overflow is NOT undefined behavior and does NOT raise runtime error in this profile.

Implication: `Overflow` diagnostics are extension points for future checked profiles, not active runtime behavior in V1.

## 6.2 Tape and Pointer

V1 tape model is fixed:
- Tape length is exactly `30_000` cells.
- Pointer starts at cell index `0`.
- Pointer movement outside `[0, 29_999]` MUST trap at runtime with `E_PTR_OUT_OF_BOUNDS` and terminate execution with non-zero status.

## 6.3 I/O (`','` and `'.'`)

`','` and `'.'` are stream operations, not function parameter/return mechanisms.

V1 I/O behavior is fixed to byte streams:
- Input stream: `stdin`.
- Output stream: `stdout`.
- `,`: read one byte from `stdin`.
	- If read succeeds, store that byte value (`0..255`) into current cell as `i64`.
	- On EOF, store `0` into current cell.
- `.`: write the low 8 bits of current cell (`cell as u8`) to `stdout`.

No UTF-8 scalar-output mode is defined in V1.

## 7. Function Calls

V1 does not support source-level function calls.

- No `call` keyword or equivalent instruction is defined in V1.
- Any call-like syntax MUST produce compile error `E_CALL_UNSUPPORTED`.

## 8. Suggested CLI Contract

Example:

```bash
bfc a.bf b.bf -o out
```

Behavior:
- Parse and merge all files.
- Validate symbols and entry-point constraints.
- Choose artifact kind by entry-point count.
- Generate one object pipeline output from merged unit.

## 9. Extension Points (Beyond V1)

- Additional cell profiles (e.g. `u8` wrapping, checked arithmetic).
- Alternative tape strategies (e.g. auto-growing tape).
- Explicit I/O mode flags (including UTF-8 scalar-output mode).
- Function-call syntax and ABI.
- Dynamic library output.

## 10. Trade-off Rationale

These V1 choices are intentionally biased toward implementation simplicity while matching mainstream low-level compiler practice:

- Fixed tape (`30_000`) and explicit bounds trap: simpler runtime model and deterministic behavior.
- Byte-stream I/O (`stdin`/`stdout`, low 8-bit output): straightforward backend mapping to common C/OS primitives.
- i64 wrapping arithmetic: unambiguous machine-like semantics, no UB, easy code generation.
- Front-end merge model: avoids building a full linker in V1 while still teaching symbol/entry diagnostics.
