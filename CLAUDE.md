# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project state: pre-implementation

There is **no Rust code yet** — only design docs in `docs/`. Build it task-by-task from
`docs/plan.md`, starting at **T-01** (workspace scaffolding). The commands below are the
agreed tooling and won't run until the workspace exists.
*(Delete this section once the workspace is scaffolded.)*

## Source-of-truth documents (read before acting; don't re-derive settled decisions)

- **`docs/design.md`** — full rationale + the end-to-end pipeline (§2) and runtime/build split (§9).
- **`docs/spec.md`** — normative requirements (`FR-*`, `NFR-*`).
- **`docs/plan.md`** — tasks `T-01…T-57`, dependencies, critical path.
- **`docs/style.md`** — the **mandatory, enforced** Rust style guide (backed by committed config + blocking CI).

Keep these in sync with any code you write.

## Architecture

A Rust CLI that **deterministically** turns text into BB-8-like droid sound, where
*semantically similar text sounds similar* (a learnable sound-language). Pipeline detail
is in `docs/design.md` §2; the workspace has three crates:

- **`dootdoot-core`** — pure, deterministic engine (functional core): tokenizer, mapping,
  synth, owned math, WAV, `FORMAT_V1` constants. No I/O, no audio device.
- **`dootdoot`** — thin CLI shell (imperative shell): `clap`, stdin, `rodio` playback,
  `--explain`, error/exit mapping. Holds essentially all side effects.
- **`xtask`** — build-time only, never shipped: generates `assets/format_v1.bin` from
  `potion-base-2M` via `model2vec-rs`.

### Load-bearing invariants (violating these breaks the core promises)

- **model2vec / `candle` are BUILD-TIME ONLY.** The shipped binary has no tensor runtime:
  the token→4-axis mapping is precomputed into `assets/format_v1.bin` (~300 KB) and
  `tokenizer.json`, both committed and `include_bytes!`-embedded. (Sound because PCA
  projection is linear: pooling baked vectors == pooling-then-projecting, *exact before
  the int16 quantization* the table uses.)
- **The sequence baseline is dootdoot's own pooling, NOT `model2vec.encode()`.** `encode()`
  L2-normalizes the pooled vector (`potion-base-2M` has `normalize: true`); that step is
  nonlinear and does not commute with the projection, so it can't be recovered from baked
  4-axis vectors. dootdoot's baseline is the token-weight-scaled mean in PCA space,
  denominator = token count, **no L2 norm** — a documented, FORMAT_V1-pinned divergence
  (design.md §4.2, FR-11). Goal: relative semantic ordering, not `encode()` equivalence.
- **Determinism is bit-exact on the CI-verified platforms (macOS + Linux);** Windows is
  intended but not yet guaranteed. No libm transcendentals in the audio path
  (`sin`/`exp`/`tanh` are our own pinned impls); synthesis in `f64`; one fixed float→i16
  rounding rule; no fast-math/FMA. Any parallelism must be byte-identical to serial.
- **`FORMAT_V1` is a versioned contract** over everything affecting an output sample
  (mapping, quantization scales, tokenizer config, synth + timing constants, punctuation
  rules, chirp, float→i16 rounding, WAV serialization, math version).
  **Any change that alters even one sample MUST bump the version** (`V1`→`V2`) and
  regenerate golden fixtures. Golden-WAV hash tests are that contract.
- **One canonical audio buffer** feeds both file output and playback, so what plays equals
  what's saved.

## Development workflow: red-green TDD (mandatory)

Implement every behavior test-first: **red** (write a failing test, confirm it fails for
the right reason) → **green** (minimum code to pass) → **refactor** (clean up, stay green).
The pure functional core exists to make this easy. Pick the cheapest test level that pins
the behavior — value test, `proptest` invariant, `insta` snapshot, or golden-WAV hash
(see `docs/style.md` §9). If a test is hard to write, fix the design, don't skip it.

## Version control

Uses **`jj` (Jujutsu)**, colocated with `.git` — use `jj`, not `git`. Segment work into
small, focused revisions, each a single coherent change with a clear description (imperative
summary + a "why" body when non-obvious). Aim for ~one `T-*` task / one red-green cycle per
revision. **Start a new revision (`jj new`) before beginning the next logical change.**

## Commands (per `docs/style.md` §11)

```bash
# Run
cargo run -p dootdoot -- "hello there"        # play live
cargo run -p dootdoot -- "hi" -o hi.wav       # write WAV (no playback)
cargo run -p dootdoot -- "hi" -o hi.wav --play

# Test (includes doctests)
cargo test
cargo test -p dootdoot-core <test_name>        # one crate / a single test by name

# Format — NIGHTLY rustfmt required (rustfmt.toml uses nightly-only options);
# build/test stay on pinned stable.
cargo +nightly fmt          # --check in CI

# Lint (warnings are errors in CI)
cargo clippy --all-targets -- -D warnings

# Coverage (≥95% on dootdoot-core), dependency hygiene
cargo llvm-cov
cargo deny check && cargo machete

# Regenerate the baked asset (ONLY when intentionally changing the mapping → FORMAT bump)
cargo run -p xtask
```

## Key conventions (full guide: `docs/style.md`)

- **Functional core / imperative shell** — pure logic in `dootdoot-core`, effects behind
  injected traits in the binary. This is what makes ~99% of code deterministically testable.
- **`forbid(unsafe_code)` workspace-wide.** No `as` for numeric casts (use `From`/`TryFrom`
  or the named quantization helper). No reliance on `HashMap` iteration order.
- **Files:** one primary construct per file, named after it, namesake at top, ordered by
  relevance (public first); no `mod.rs`. Complex tests in `<namesake>/tests.rs`.
- **Public API is a curated `pub use` facade** over private modules; don't leak dependency
  types (`hound`/`rodio`/`tokenizers`) across public boundaries.
- **Errors:** `thiserror` in libs, `anyhow` only in the binary; no `unwrap` outside tests;
  panic only for invariants, never bad input.
