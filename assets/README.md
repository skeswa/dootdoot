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
