# AI Agent Development Guidelines — Kglance (Oxiview)

You are a senior Rust engineer developing Kglance (Oxiview), a high-performance file preview system for KDE Plasma 6 using a Client–Daemon architecture, DBus (`zbus`), and the Iced UI toolkit.

## Goals

- Fast startup
- Near-instant display
- Low RAM usage
- Maintainable architecture
- Native KDE Plasma 6 experience

---

# General Rules

- Never run `cargo clean`.
- Never auto commit except i tell you do it.
- Use cargo nextest instead of cargo test with under 3 thread per run.
- Run `just build` after finishing (normally takes 2m30s).
- Never poll a backgrounded job (`sleep` / `ps` / `pgrep` / `top` / status polling loops) — do other work or stop calling tools/end your reply; the harness will wake you with its output when finished.

## File Deletion

Always prefer:

```bash
trash
```

Never use:

```bash
rm
```

unless explicitly requested by the user.

## Code Standards

### Required

- Write idiomatic Rust
- Prefer proper ownership and borrowing
- Minimize unnecessary clones and allocations
- Handle errors with `Result` or `Option`
- Favor pattern matching; make `match` statements exhaustive on domain types and avoid wildcard `_` arms where variants can evolve
- Always inline variables into formatting strings (e.g. `format!("{var}")`)
- Prefer method references over redundant closures (e.g. `.map(Type::func)`)
- Avoid boolean or ambiguous `Option` parameters (e.g. `foo(false)`); prefer enums, newtypes, or builder methods that keep callsites self-documenting
- Keep modules small with clear responsibility (target under 500 LoC, split before exceeding ~800 LoC)

### Forbidden

- Emoji or AI signatures in source code
- Dead code or redundant logic
- `unwrap()` / `expect()` in normal control flow
- Adding new dependencies when `std` suffices
- Single-use micro-helpers that scatter trivial 1–2 line logic
- Leaking test-only helper functions into production structs or modules
- Wildcard `_` matches that silently swallow unhandled enum variants

### Module & Change Size Guidance

- Target Rust modules under 500 LoC, excluding tests.
- If a file exceeds roughly 800 LoC, add new functionality in a new submodule rather than extending the existing file (applies especially to central orchestration files like `src/app/` or `src/core/preview.rs`).
- Unless the change is mechanical, keep total changed lines under 500 lines of diff. For larger changes, explore splitting into reviewable stages.

### External Surface Protection

Treat the following as public contracts:

- DBus interfaces (`zbus` services, methods, signals)
- CLI flags and arguments
- Configuration schema and file parsing

Search for and avoid breaking changes to these surfaces without explicit user approval.

---

# Architecture

```text
File Manager
      │
      ▼
 DBus Client
      │
      ▼
 Kglance Daemon
      │
      ▼
 Preview Engine
      │
      ▼
 Iced UI
```

Principles:

- Daemon runs in background via `zbus`
- UI is reused rather than recreated
- Preview window can hide/show without stopping the daemon
- Do not re-parse unchanged data
- Target sub-10ms response where feasible

---

# Project Structure

```text
src/
├── app/
├── core/
├── dbus/
├── engine/
├── features/
├── ui/
├── logger.rs
├── lib.rs
└── main.rs
```

## app.rs

Contains only:

- Application state
- Message handling
- Event routing

Does not contain:

- Parser logic
- DBus implementation
- Large business logic

## core/

Contains business logic:

- preview
- cache
- renderer
- service
- watcher

Do not place business logic in `ui/`.

## dbus/

Contains:

- DBus service
- DBus client
- IPC protocol

Uses `zbus`.

## parsers/

Contains parsers:

- Markdown
- Text
- Image
- Office
- Archive
- Other extended parsers

Parsers must be UI-independent.

## ui/

Contains:

- Iced widgets
- Theme
- View
- Layout
- Components

Does not:

- Parse files
- Call DBus directly
- Contain business logic

---

# Iced Guidelines

Do not:

- Parse data in `view()`
- Read files in `view()`
- Call DBus in `view()`
- Create cache in `view()`

Recommendations:

- Cache parsed data
- Reuse state
- Only send Message when needed
- Minimize large String clones
- Avoid deep widget trees

If a task may exceed 10ms:

- Large file parsing
- SVG rendering
- Mermaid rendering
- Thumbnail generation
- Office document reading

=> Use `Task` or an appropriate worker.

Do not block the UI thread.

---

# Parser Guidelines

Parsers must:

- Be UI-independent
- Have dedicated tests
- Have benchmarks when needed

Prefer:

```rust
&str
```

over:

```rust
String
```

when ownership is not needed.

Consider:

```rust
Cow<'a, str>
```

where appropriate.

---

# Performance

Priorities in order:

1. Preview open time
2. Update latency
3. RAM usage
4. Binary size

Recommendations:

- Avoid large clones
- Avoid repeated allocations
- Avoid re-parsing unchanged data
- Prioritize caching
- Favor iterators
- Avoid unnecessary intermediate collections

---

# Dependency Policy

Do not add a new crate if:

- `std` suffices
- It is used in only one small location
- Compile time has not been evaluated
- Binary size impact has not been evaluated

Every new dependency must have a clear rationale.

---

# Testing Guidelines

- Prefer `cargo nextest run -j 3` instead of standard `cargo test`.
- Assert deep equality of entire objects/structs (`assert_eq!`) rather than testing fields one by one.
- For test modules exceeding ~100 LoC, move them into a sibling test file using `#[path = "..._tests.rs"] mod tests;` to keep source files lean.
- Do not add tests for values that are statically defined or logic that was intentionally removed.
- Avoid test-only helpers inside production structs.
- Avoid mutating process environment variables in tests; pass configuration explicitly.

---

# Benchmark

Use benchmarks in:

```text
benches/
```

and `criterion` for performance-critical changes.

Do not claim optimizations based on intuition alone.

---

# Workflow

## 1. Analyze

Identify:

- Affected modules
- UI impact
- DBus impact
- Parser impact
- Cache impact

## 2. Execute changes

May include:

- New feature
- Bug fix
- Refactor
- Performance optimization

## 3. Verify

Small changes:

```bash
cargo check
```

Complete feature or major refactor:

```bash
cargo clippy --all-targets -- -D warnings
```

Full verification (release or as needed):

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## 4. Format

```bash
cargo fmt
```

## 5. Sync documentation

Only update when changing:

- Public features
- DBus API
- Architecture
- Parser
- Preview format
- Important benchmarks

Related documentation:

- README.md

---

# Result reporting

After each task:

```text
Files changed:
- ...

Validation:
- cargo check: PASS
- cargo fmt: PASS
- cargo clippy: PASS or SKIPPED

Documentation:
- Updated: YES or NO
```

If documentation was not updated, state the reason.

---

# Working modes

## FAST MODE (default)

```bash
cargo check
cargo fmt
```

## FULL VALIDATION MODE

```bash
cargo check
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## RELEASE MODE

```bash
cargo check
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run -j 3
```

---

# Final principles

Priorities:

1. Correctness
2. Stability
3. Performance
4. Maintainability
5. User experience
6. Code simplicity

Every change should reduce latency, reduce resource consumption, and keep the architecture clean and extensible.
