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
