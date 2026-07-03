# Assets

T-11 chooses `download-to-build-cache` for the upstream `potion-base-8M` source model.
The large build-time inputs are not committed. They are downloaded into:

```text
target/source-cache/minishlab/potion-base-8M/<revision>/
```

The committed [`source_manifest.toml`](source_manifest.toml) pins the exact Hugging Face
revision, source file hashes, and structural expectations. `xtask` validates the cached
`model.safetensors`, `tokenizer.json`, and `config.json` against that manifest before any
generation work runs.

The runtime asset is committed as a single Protocol Buffers payload:

- `assets/dootdoot_asset_v1.doot` contains the tokenizer JSON, the projected semantic
  mapping records, quantization scales, squash statistics, and provenance hashes. It is
  roughly 1 MB because it accumulates the old split runtime payloads into one file.

To regenerate the asset after an intentional asset-spec or mapping change:

1. Ensure the pinned source files exist under `target/source-cache/`.
2. Run `cargo run -p xtask`.
3. Copy `target/generated/dootdoot_asset_v1.doot` to `assets/dootdoot_asset_v1.doot`.

See the full [`docs/reference/asset_regeneration.md`](../docs/reference/asset_regeneration.md)
guide before committing regenerated assets. `VOICE_V1` is locked, so any regeneration
that changes rendered samples must happen under a new voice identifier.

## VOICE_V12 sidecar class table

- `assets/dootdoot_pos_v1.doot` is the baked noun/verb class table (FR-114): a small
  deterministic binary keyed by surface word, derived from
  `assets/pos/tagged_counts.tsv` — per-word statistics tagged once from a pinned public
  CommitChronicle shard (see the `[pos]` manifest section; no message text is
  committed).

To regenerate after an intentional policy or snapshot change:

1. If the snapshot itself changes: `uv run scripts/derive_pos_table.py`, then update
   `tagged_counts_sha256` in `source_manifest.toml`.
2. Run `cargo run -p xtask -- pos-table` (validates the snapshot pin first).
3. Copy `target/generated/dootdoot_pos_v1.doot` to `assets/dootdoot_pos_v1.doot`.

`xtask/tests/pos_snapshot_pinning.rs` asserts the committed sidecar reproduces
byte-for-byte from the committed snapshot. Once `VOICE_V12` freezes, the class table is
part of the voice contract: regeneration that changes classifications changes rendered
samples and needs a version bump.
