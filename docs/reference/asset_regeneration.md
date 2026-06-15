# Asset Regeneration Guide

Normal builds use committed assets and do not need network access. Regeneration is a
release-engineering operation for intentional mapping or format changes.

## Inputs

`xtask` reads the source files pinned by [`assets/source_manifest.toml`](../../assets/source_manifest.toml):

- Hugging Face repository and immutable revision.
- SHA-256 hashes for `model.safetensors` and `tokenizer.json`.
- Structural expectations: `hidden_dim = 256`, `normalize = true`, and source dtype.

The source files live in the build cache:

```text
target/source-cache/minishlab/potion-base-8M/<revision>/
```

`xtask` validates the cached files against the manifest before it writes generated
artifacts. A mismatch means the cache or manifest is wrong; do not hand-edit generated
outputs to work around it.

## Regeneration Steps

```sh
cargo run -p xtask
cp target/generated/format_v1.bin assets/format_v1.bin
cp target/source-cache/minishlab/potion-base-8M/<revision>/tokenizer.json assets/tokenizer.json
cargo test -p xtask
cargo test -p dootdoot-core --test golden_wav_hashes
scripts/lint
```

If the source manifest changes, review and commit it with the regenerated assets. If the
tokenizer changes, copy the new pinned `tokenizer.json` byte-for-byte from the validated
cache.

## When to Bump the Format

`FORMAT_V1` is locked. Any change that alters one rendered sample requires a new format
identifier, new reference documentation, and regenerated golden WAV hashes.

Examples that require `FORMAT_V2`:

- A different source model, tokenizer, source revision, PCA projection, axis count, or
  quantization rule.
- Different squash statistics or squash function.
- Changes to knob assembly, synthesis constants, timing constants, punctuation rules,
  empty-chirp constants, owned math, float-to-i16 rounding, or WAV serialization.

Examples that do not require a format bump:

- Documentation-only edits.
- Packaging metadata that does not affect runtime behavior.
- Regenerating assets from the same manifest and producing byte-identical output.

## Review Checklist

- `assets/format_v1.bin` or the new versioned asset has the expected size and layout.
- `assets/tokenizer.json` matches the pinned manifest hash.
- Golden WAV hashes were regenerated only after accepting the new format's sound.
- `docs/design.md`, `docs/spec.md`, `docs/plan.md`, and the versioned reference document
  describe the new contract.
