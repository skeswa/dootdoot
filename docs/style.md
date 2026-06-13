# dootdoot — Rust Style Guide

> A highly opinionated guide combining canonical Rust best practices with this
> project's preferences. It is **tailored to dootdoot** (it references the
> `dootdoot-core` / `dootdoot` / `xtask` workspace and the determinism contract from
> [`design.md`](./design.md)) but is broadly reusable for other Rust projects.
>
> **Posture: normative and mechanically enforced.** Every rule a tool can check is
> backed by committed config (`rustfmt.toml`, `[workspace.lints]`, `clippy.toml`,
> `deny.toml`) and CI. Prose carries only the judgment-based rules. The guide and the
> toolchain must never drift: if you change a rule here, change the config too.
>
> Keywords **MUST**, **SHOULD**, **MAY** are used in the usual sense.

---

## 1. Philosophy & goals

These principles justify every concrete rule that follows. When a rule seems
inconvenient, re-read this section — the rule is almost always serving one of these.

1. **High cohesion, low coupling.** Each file/module does *one* cohesive thing; modules
   depend on each other as little as possible, and only through deliberate, narrow
   seams. Cohesion drives the file rules (§2–§3); decoupling drives the API rules (§6).
2. **Testability is a structural property, not an afterthought.** The architecture
   MUST make ~99% of code deterministically testable by construction (§9). If something
   is hard to test, that is a design smell to fix, not a test to skip.
3. **Determinism.** Same input → same output, everywhere, forever (the project's
   headline property). This shapes the bans on nondeterminism (§9), the cast rules
   (§7), and the concurrency rules (§10).
4. **Clarity over cleverness.** Code is read far more than written. Prefer the obvious
   construct, the descriptive name, and the documented intent over the terse trick.
5. **Make the compiler do the work.** Push invariants into the type system (§7) and
   correctness rules into lints (§11) so that violations fail at build time, not review
   time or runtime.

---

## 2. Project & module structure

- **Workspace.** The repository is a Cargo workspace of three crates:
  - **`dootdoot-core`** — the pure, deterministic engine (functional core). No I/O, no
    audio device. Fully unit-testable in isolation.
  - **`dootdoot`** — the thin CLI binary (imperative shell): argument parsing, stdin,
    playback, error/exit-code mapping. Holds essentially all side effects.
  - **`xtask`** — build-time-only tooling (e.g. the asset generator). Never shipped;
    carries heavy build-time dependencies so the shipped binary does not.
- **Small files, one cohesive thing each.** Most files SHOULD be under ~200 lines.
  Length is a *smell*, not a law: split a file when it starts doing two jobs, not when
  it crosses a line count. There is **no hard per-file line gate** (it would fight
  cohesion when one type legitimately needs a long `impl`). Function length *is*
  mechanically bounded via `clippy::too_many_lines`.
- **Files are named after the construct they define.** `Voice` lives in `voice.rs`,
  `TokenId` in `token_id.rs`. Module names match their namesake (singular: `voice`,
  not `voices`); only genuinely collection-like modules go plural.
- **One *primary* construct per file ("primary-plus-satellites").** A file is named
  after exactly one primary construct. It MAY also contain only that construct's own
  `impl`s, its private helpers, and tightly-coupled *satellite* types — its error,
  builder, iterator (`VoiceError`, `VoiceBuilder`, `VoiceIter`). A second *independent*
  public type belongs in its own file.
- **Modern module path style, no `mod.rs`.** Use `foo.rs` + `foo/` directories.
  `mod.rs` files are not used.

---

## 3. File internal organization

Files are organized **top-to-bottom in order of relevance to the reader.** Public,
high-level API is most relevant and comes first; implementation detail and tests come
last. A reader scanning from the top meets the contract before the mechanism.

**Canonical item order, top to bottom:**

1. **Module doc** (`//!`) — what this file/module is and why it exists.
2. **Imports** (`use`) — grouped/ordered mechanically by rustfmt (§11).
3. **The namesake construct** — the primary type/trait/fn the file is named after.
   Always at the very top of the code. Public consts/types that define its contract sit
   with it.
4. **Its inherent `impl`(s)** — methods ordered *by relevance*: constructors
   (`new`/`from_*`/`with_*`) first, then primary operations (most-used first), then
   secondary/niche operations, then trivial accessors last.
5. **Trait impls for the namesake** — std/derive-style first, then project traits.
6. **Tightly-coupled satellite types** and their impls.
7. **Private helpers** — free functions, private consts/statics used only here.
8. **Tests** — always last (see below).

**Cross-cutting rule:** at every level, **public before private**, with relevance as
the tiebreak. Private helper consts live near their use or in the helpers block, never
at the top.

**Tests placement (three tiers):**

- **Simple unit tests → inline** at the bottom in `#[cfg(test)] mod tests { … }`.
- **Complex white-box tests (need private access) → a sibling file.** The parent
  declares `#[cfg(test)] mod tests;` at its very bottom; with the no-`mod.rs` style the
  tests live in `<namesake>/tests.rs` (e.g. `voice.rs` → `voice/tests.rs`) and reach
  privates via `use super::*;`.
- **Black-box tests (public API only) → the top-level `tests/` directory**, one file
  per area (e.g. the golden-WAV suite).
- **Threshold for "complex"** (judgment): split a test module out to a sibling file
  when it would (a) rival or exceed the length of the code under test, (b) need its own
  fixtures/helpers/sub-modules, or (c) exceed ~100 lines. Below that, keep it inline.
- **Shared fixtures** go in a `#[cfg(test)]`-gated `testutil` module or `tests/common/`.

---

## 4. Naming conventions

Casing is the standard, clippy-enforced set: `UpperCamelCase` types/traits/enums,
`snake_case` fns/vars/modules, `SCREAMING_SNAKE_CASE` consts/statics, short lowercase
lifetimes (`'a`). Beyond that:

- **Acronyms are words:** `HttpClient`, `TokenId`, `WavWriter` — never `HTTPClient`,
  `ID`, `WAVWriter`. (`Id`, not `ID`.)
- **No `get_` prefix on getters:** `fn name()` / `fn name_mut()`.
- **Conversion prefixes carry cost semantics:** `as_*` = cheap borrow/view, `to_*` =
  expensive/clone, `into_*` = owning/consuming.
- **Constructors:** `new`, `with_*`, `from_*`. **Predicates:** `is_*`, `has_*`,
  `should_*`.
- **Satellites:** `<Name>Error`, `<Name>Iter`, `<Name>Builder`.
- **Abbreviations — whole words, with a small allowlist.** Prefer full words. The only
  permitted abbreviations are a documented canonical set — `ctx`, `cfg`, `buf`, `idx`,
  `id`, `len`, `tmp` — plus **established domain terms** where the abbreviation *is* the
  real word: `pcm`, `lfo`, `fft`, `wav`, `hz`, `pca`. Everything else is spelled out.

---

## 5. Documentation

Documentation is a hard, partly-enforced contract.

**The doc-comment contract (applies to every public member):**

- A **single leading sentence** that briefly says what the item does. It ends with a
  period (rustdoc uses it as the listing blurb), is written in **third-person present**
  ("Renders…", "Returns…", "Holds…"), and **MUST NOT merely restate the name.**
- If more is warranted, a **blank line**, then one or more paragraphs giving context
  (why it exists) and/or how it works. Keep it brief.

**Scope:**

- **All public members MUST be documented** — enforced by `#![deny(missing_docs)]` at
  each crate root. Missing public docs fail CI.
- **Non-intuitive or complicated private members MUST also be documented** to the same
  contract. This is a prose/review rule, *not* lint-enforced, because the lint is
  all-or-nothing and trivial private helpers are intentionally left undocumented.
- Every file/module starts with a `//!` module doc; `lib.rs` carries a crate-level
  `//!` doc.

**Mechanics & required sections (clippy-enforced):**

- `///` on items, `//!` on modules/crate.
- `# Errors` on every `pub fn` returning `Result` (`clippy::missing_errors_doc`).
- `# Panics` on every public fn that can panic (`clippy::missing_panics_doc`); reachable
  panics MUST be documented here.
- `# Safety` on every `unsafe fn` (`clippy::missing_safety_doc`).
- `// SAFETY:` comments are **mandatory inline** at every `unsafe { }` block (distinct
  from the `# Safety` doc section). (Note: `unsafe` is forbidden by default — see §11.)

**Examples / doctests:**

- **Required for non-trivial public API**, written as `# Examples` runnable doctests.
  This is deliberate synergy with §9: doctests are deterministic tests and keep docs
  honest. Trivial getters/obvious constructors are exempt.

---

## 6. Visibility & API design

The goal is **low coupling**: small files (§2) MUST NOT leak into consumers' import
paths, and internal reorganization MUST NOT break users.

- **Private-by-default with a facade at the crate root.** Modules are declared
  `mod foo;` (private). `lib.rs` curates the public surface via explicit
  `pub use foo::Bar;` re-exports. Users import `dootdoot_core::Voice`, never
  `dootdoot_core::synth::voice::Voice`. Split files freely without breaking anyone.
- **Scope as tightly as possible.** Prefer `pub(crate)`/`pub(super)` over `pub`. `pub`
  means "deliberate, documented, supported."
- **No public fields** on public structs — expose accessors (no `get_`, §4). Plain data
  DTOs are an explicit, documented exception.
- **`#[non_exhaustive]`** on public enums and structs likely to grow.
- **Sealed traits** for any public trait not meant to be implemented downstream.
- **Do not leak dependency types across the public boundary.** External crate types
  (`hound::*`, `tokenizers::*`, `rodio::*`) stay behind our own types/newtypes, so a
  dependency swap is a one-module change. Internally, depend on **traits, not
  concretes**, at module seams — the same seams that enable testing (§9).

---

## 7. Types & data modeling

Let the type system carry correctness.

- **Make illegal states unrepresentable.** Reach for types before runtime checks:
  enums instead of boolean/`Option` soup; validated/non-empty types over "validate
  later." A function should be hard to call wrong.
- **Newtypes over primitive obsession.** Domain values get newtypes: `Hz(f64)`,
  `Seconds(f64)`, `Sample(i16)`, `TokenId(u32)`, `AxisValue` — not bare `f64`/`u32`.
  Prevents unit mix-ups and documents intent at the signature.
- **Enums over `bool` parameters.** `fn render(mode: Playback)`, not
  `fn render(play: bool)`. No mystery `true` at call sites.
- **Builders only when earned** (several optional fields); otherwise constructors +
  `Default` + struct-update syntax.
- **Borrow in, own out.** Params take `&str`/`&[T]`/`impl AsRef<…>`; return owned
  values. Don't force allocations on callers.
- **Derive discipline.** `#[derive(Debug)]` on essentially everything — enforced by
  `#![deny(missing_debug_implementations)]` (Debug is non-negotiable for test
  diagnostics, §9). `Clone`/`Copy`/`PartialEq`/`Eq`/`Hash` added deliberately; `Copy`
  only for genuinely small value types.
- **Cast discipline (serves determinism).** **`as` is forbidden** for lossy/numeric
  conversions (the `cast_*` clippy family is denied). Use `From`/`TryFrom` or explicit,
  documented rounding helpers. The single sanctioned float→int conversion (the spec's
  fixed quantization rule) lives in one named function, never a bare `as`.

---

## 8. Error handling

Split by crate role.

- **Libraries (`dootdoot-core`): typed errors via `thiserror`.** One
  `#[non_exhaustive]` error enum per module boundary; meaningful variants; source
  chains preserved with `#[source]`/`#[from]`; messages **lowercase, no trailing
  period**. **No `anyhow` in any library's public API** — callers must be able to match
  on errors. A `pub type Result<T> = …` alias per crate is encouraged.
- **Binary (`dootdoot`): `anyhow`** at the top level for aggregation + `.context(…)`,
  mapped to friendly stderr messages and exit codes. `anyhow` lives only in the binary.
- **`Option` vs `Result`:** `Option` for legitimate absence, `Result` for failure.
  Never use errors for normal control flow.
- **`unwrap`/`expect`:** `unwrap` is denied outside tests (`clippy::unwrap_used`).
  `expect` is permitted **only** for statically-guaranteed invariants and its message
  MUST state why it cannot fail. Both are freely allowed in `#[cfg(test)]`.
- **Panics:** `panic!`/`unreachable!` only for genuine programmer-error invariants —
  **never** for input-driven failures (bad text, bad path → typed/`anyhow` error). All
  reachable panics documented under `# Panics` (§5).
- **`todo!`/`unimplemented!`/`dbg!`** are denied on `main` (`clippy::todo`,
  `unimplemented`, `dbg_macro`). Fine locally, never merged.
- **Flow:** prefer `?`; don't `match`-and-rewrap when `?` + `#[from]` suffices.

---

## 9. Testability & testing

Testability is guaranteed by **architecture**, then verified by tests. The "99%
deterministically testable" goal is met by construction.

**Architecture:**

- **Functional core, imperative shell.** All logic lives in **pure** functions/types:
  no I/O, no clock, no randomness, no global state — deterministic in → deterministic
  out. `dootdoot-core` is that core. Side effects (stdin, WAV write, the `rodio`
  device, time) are confined to the thin shell in the `dootdoot` binary.
- **Inject unavoidable effects behind traits.** Clock, filesystem, audio sink, any
  entropy → a trait with a real impl in the shell and an **in-memory fake** in tests.
  The genuinely-untestable residue shrinks to a few syscalls.
- **Ban nondeterminism in the core** (also serves the determinism contract): no
  `SystemTime::now`, no un-seeded randomness, and **no reliance on `HashMap`/`HashSet`
  iteration order** (use `BTreeMap`/`Vec`, or sort before output). No global mutable
  state or singletons.
- **"The 1%"** is exactly the shell's irreducible syscalls (real playback, real file
  write/stdin read). Everything above the trait seam is in the 99%.

**Test types & tooling:**

- **Value/unit tests** for pure logic (placement per §3).
- **Golden/snapshot tests** via `insta` for structured output (e.g. the `--explain`
  table); **hash-based golden files** for binary output (the WAV determinism contract).
- **Property-based tests** via `proptest` for invariants (e.g. "same input twice →
  identical bytes", "output length is a deterministic function of token count").
- **Doctests** as deterministic examples (§5).

**Coverage:** measured with `cargo-llvm-cov` in CI. Threshold **≥95% on
`dootdoot-core`**, a lower bar on the shell. The number is a backstop; the architecture
is the real guarantee. Any coverage exclusion MUST be explicitly annotated, never
silent.

**Hygiene:** no sleeps, no timing-dependent assertions, no network in tests.
Descriptive test names; arrange-act-assert; assert behavior at module seams, not
private implementation detail.

---

## 10. Concurrency & async

Synchronous by default; concurrency is added only when it pays for itself, and never at
the cost of determinism.

- **Default to synchronous, blocking code.** Do **not** introduce an async runtime
  (`tokio`) for CLI/CPU-bound tools. `async` is justified **only** by genuine
  concurrent I/O (network services, many simultaneous sockets), never "it might be
  faster." dootdoot is synchronous end to end.
- **When async is warranted: async shell, sync core.** Keep `async` at the I/O edges;
  keep the functional core (§9) plain synchronous and pure, so it tests without an
  executor. One runtime per binary; don't mix runtimes.
- **CPU parallelism: prefer `rayon`** data-parallelism over hand-rolled threads; use
  **scoped threads** when manual threading must borrow stack data.
- **Parallelism MUST be bit-identical to the serial path** (determinism contract).
  Parallelize only independent work and **collect results in deterministic order** (by
  index), never relying on completion order. Beware float reduction-order changes; fix
  the association if a reduction must be parallel. Golden-file tests (§9) guard this.
- **Shared state:** prefer ownership/message-passing (channels) over shared mutable
  state; use `Arc<Mutex<_>>`/`RwLock` only deliberately; **never hold a lock across an
  `.await`**; avoid lock-ordering hazards.
- **No hand-rolled `unsafe impl Send/Sync`** unless unavoidable and documented with a
  `// SAFETY:` rationale. (`unsafe` is forbidden by default — §11.)

---

## 11. Tooling & enforcement

The mechanical backbone. Config is committed; CI gates are blocking on `main`.

**Toolchain & edition:**

- **Pinned stable toolchain** via `rust-toolchain.toml` (with `rustfmt`, `clippy`,
  `llvm-tools`). Stable only for building — never nightly. Pinning serves reproducible,
  deterministic builds.
- **Edition 2024.** **MSRV = the pinned toolchain** — these are applications, not
  widely-consumed libraries, so there is no back-compat burden; track latest stable and
  bump deliberately.

**Formatting (`rustfmt.toml`):**

- Includes the §3 import policy: `group_imports = "StdExternalCrate"` (three blocks:
  std/core/alloc, external crates, then `crate`/`super`/`self`) and
  `imports_granularity = "Crate"`, plus `format_code_in_doc_comments` and
  `wrap_comments`.
- These are nightly-only rustfmt options, so **CI formats with a pinned nightly used
  *only* for formatting**: `cargo +nightly fmt --check`. Building and testing stay on
  pinned stable.
- No glob imports except a documented crate `prelude` and `use super::*;` inside
  `mod tests`.

**Lints (`[workspace.lints]` in the workspace `Cargo.toml`, inherited via
`[lints] workspace = true`):**

- Posture: `clippy::all` + `clippy::pedantic` warned, with a **curated allow-list** for
  the genuinely noisy pedantic lints; `clippy::cargo` on for metadata hygiene;
  `clippy::nursery` **opt-in per-lint only** (unstable/false-positive-prone).
- **Targeted denies** (centralizing the rules above): `missing_docs`,
  `missing_debug_implementations`, `clippy::unwrap_used` (outside tests), the
  `clippy::cast_*` family, `clippy::missing_errors_doc`/`missing_panics_doc`/
  `missing_safety_doc`, `clippy::todo`/`unimplemented`/`dbg_macro`,
  `clippy::too_many_lines`.

**CI pipeline (all blocking on `main`):**

1. `cargo +nightly fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test` (including doctests)
4. `cargo llvm-cov` (thresholds per §9)
5. **Cross-platform determinism** — macOS + Linux, assert identical golden hashes
6. `cargo deny` and `cargo machete` (§12)

---

## 12. Dependencies & `unsafe`

**Dependencies:**

- **Minimal and justified.** Prefer `std`. Each new dependency MUST earn its place
  (maintenance, transitive weight, license, its own `unsafe` footprint). Disable
  `default-features`; enable only what's needed.
- **`cargo deny`** (committed `deny.toml`, CI-gated): **RustSec advisories** denied
  (vulnerable/unmaintained); **license allowlist** permissive only —
  MIT / Apache-2.0 / BSD / ISC / Zlib / Unicode (anything else fails and needs explicit
  review); **bans** — no duplicate major versions, no wildcard deps; **sources** —
  crates.io only unless whitelisted.
- **Commit `Cargo.lock`** — applications/binaries, so the lockfile is part of
  reproducible, deterministic builds.
- **`cargo-machete`** in CI to catch unused deps.
- External types stay wrapped behind our own (§6) so a dependency swap is local.

**`unsafe`:**

- **`unsafe_code = "forbid"`** in `[workspace.lints]` for all three crates. dootdoot is
  pure safe Rust (`unsafe` lives inside vetted upstream crates), so forbidding it
  outright is a strong correctness/determinism signal.
- **Escape hatch (default remains forbid):** if a crate genuinely needs `unsafe`,
  downgrade *that crate* to `unsafe_code = "deny"` with written justification, isolate
  all `unsafe` in a dedicated module, set `unsafe_op_in_unsafe_fn = "deny"`, add a
  `// SAFETY:` comment on every block and a `# Safety` doc on every `unsafe fn` (§5),
  and keep blocks minimal.

---

## 13. Quick reference (the rules at a glance)

- One cohesive thing per file; named after its primary construct; namesake at the top;
  no `mod.rs`; files small by cohesion, not a line gate.
- Inside a file: module doc → imports → namesake → impls (constructors → common →
  niche → accessors) → trait impls → satellites → private helpers → tests last.
- Simple tests inline; complex white-box tests in `<namesake>/tests.rs`; black-box
  tests in top-level `tests/`.
- Public API: private modules + curated `pub use` facade; tight visibility; no public
  fields; `#[non_exhaustive]`; sealed traits; never leak dependency types.
- Docs: every public item documented (`deny(missing_docs)`); one-sentence summary +
  optional context paragraph; `# Errors`/`# Panics`/`# Safety`; doctests on non-trivial
  public API; document non-obvious private items too.
- Types: newtypes for domain values; illegal states unrepresentable; enums over
  `bool`; `Debug` everywhere; no `as` casts.
- Errors: `thiserror` in libs, `anyhow` in the binary; no `unwrap` outside tests;
  panics only for invariants, never input; no `todo!`/`dbg!` on main.
- Testability: functional core / imperative shell; effects behind injected traits; zero
  nondeterminism in the core; `insta` + `proptest` + golden files; ≥95% core coverage.
- Concurrency: sync by default; async only for real I/O and only at the edges; `rayon`
  for CPU work; parallel output MUST be bit-identical to serial.
- Tooling: pinned stable + edition 2024; nightly rustfmt for formatting only;
  `[workspace.lints]` pedantic + targeted denies; blocking CI.
- Deps: minimal + `cargo deny` + `cargo machete` + committed `Cargo.lock`;
  `forbid(unsafe_code)` workspace-wide.
