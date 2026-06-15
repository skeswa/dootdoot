# Source Assets

T-11 chooses `download-to-build-cache` for the upstream `potion-base-8M` source model.
The large build-time inputs are not committed. They are downloaded into:

```text
target/source-cache/minishlab/potion-base-8M/<revision>/
```

The committed [`source_manifest.toml`](source_manifest.toml) pins the exact Hugging Face
revision, source file hashes, and structural expectations. `xtask` validates the cached
`model.safetensors`, `tokenizer.json`, and `config.json` against that manifest before any
generation work runs.

The runtime assets are committed:

- `assets/format_v1.bin` is the generated `FORMAT_V1` mapping artifact. It should be
  roughly 300 KB; the current file is 295,488 bytes.
- `assets/tokenizer.json` is copied byte-for-byte from the pinned source cache.

To regenerate the mapping after an intentional format or mapping change:

1. Ensure the pinned source files exist under `target/source-cache/`.
2. Run `cargo run -p xtask`.
3. Copy `target/generated/format_v1.bin` to `assets/format_v1.bin`.
4. Copy the pinned source `tokenizer.json` to `assets/tokenizer.json`.

See the full [`docs/reference/asset_regeneration.md`](../docs/reference/asset_regeneration.md)
guide before committing regenerated assets. `FORMAT_V1` is locked, so any regeneration
that changes rendered samples must happen under a new format identifier.
