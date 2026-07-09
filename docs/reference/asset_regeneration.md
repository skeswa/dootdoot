# Asset Regeneration Guide

Normal builds use the committed `.doot` asset and do not need network access.
Regeneration is a release-engineering operation for intentional mapping or asset-spec
changes.

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
cp target/generated/dootdoot_asset_v1.doot assets/dootdoot_asset_v1.doot
cargo test -p xtask
cargo test -p dootdoot-core --test golden_wav
scripts/lint
```

If the source manifest changes, review and commit it with the regenerated asset. The
tokenizer JSON is embedded inside the `.doot` payload, so there is no separate runtime
tokenizer file to copy.

## The VOICE_V12 class-table sidecar

`assets/dootdoot_pos_v1.doot` (the baked noun/verb class table, FR-114) regenerates
separately from the semantic asset and needs no model download:

```sh
# Only if the statistics snapshot itself changes (new corpus shard, new tagger):
uv run scripts/derive_pos_table.py       # rewrites assets/pos/tagged_counts.tsv
#   ...then update tagged_counts_sha256 in assets/source_manifest.toml.

cargo run -p xtask -- pos-table          # validates the [pos] manifest pin first
cp target/generated/dootdoot_pos_v1.doot assets/dootdoot_pos_v1.doot
cargo test -p xtask --test pos_snapshot_pinning
```

The `[pos]` section of `source_manifest.toml` pins the public ranking-corpus shard
(repo/revision/file/SHA-256), the tagger name and version, and the committed
tagged-counts snapshot hash; `xtask pos-table` aborts on any mismatch. A pinning test
asserts the committed sidecar reproduces byte-for-byte from the committed snapshot.
Because classifications affect rendered samples, any regeneration that changes them is
a voice bump like every other sample-affecting change.

## When to Bump the Voice

`VOICE_V1` is locked. Any change that alters one rendered sample requires a new voice
identifier, new reference documentation, and regenerated golden WAV hashes.

Examples that require a new voice identifier (`VOICE_V2`, `VOICE_V3`, etc.):

- A different source model, tokenizer, source revision, PCA projection, axis count, or
  quantization rule.
- Different squash statistics or squash function.
- Changes to knob assembly, synthesis constants, timing constants, punctuation rules,
  empty-chirp constants, owned math, float-to-i16 rounding, or WAV serialization.
- A `VOICE_V12` class-table change that alters any word's classification (new corpus
  snapshot, policy thresholds, closed-class list) or the marker/resolution constants.

Examples that do not require a voice bump:

- Documentation-only edits.
- Packaging metadata that does not affect runtime behavior.
- Regenerating assets from the same manifest and producing byte-identical output.

## Review Checklist

- `assets/dootdoot_asset_v1.doot` has the expected size and parses as the dootdoot asset
  spec.
- The embedded tokenizer JSON hash matches the pinned manifest hash.
- Golden WAV hashes were regenerated only after accepting the new voice's sound.
- `docs/design.md`, `docs/spec.md`, `docs/plan.md`, and the versioned reference document
  describe the new contract.
