# Dootdoot Asset Spec

The primary committed runtime asset is `assets/dootdoot_asset_v1.doot`. The `.doot`
file is a Protocol Buffers message that carries every runtime byte needed for
tokenization and the semantic lookup table; normal builds embed it with
`include_bytes!`. Since `VOICE_V12` a second, independent sidecar asset —
`assets/dootdoot_pos_v1.doot`, the baked noun/verb class table — is committed and
embedded alongside it (its layout is specified at the bottom of this document); the
semantic asset itself remains at spec v1.

## Protobuf Message

```proto
syntax = "proto3";

package dootdoot.asset.v1;

message DootAsset {
  uint32 spec_version = 1;
  uint32 token_count = 2;
  uint32 axis_count = 3;
  repeated float axis_scales = 4;
  float weight_scale = 5;
  SquashFunction squash_function = 6;
  repeated SquashAxisStats squash_stats = 7;
  bytes model_sha256 = 8;
  bytes tokenizer_sha256 = 9;
  bytes pca_sha256 = 10;
  bytes tokenizer_json = 11;
  bytes token_records = 12;
}

message SquashAxisStats {
  double mean = 1;
  double standard_deviation = 2;
}

enum SquashFunction {
  SQUASH_FUNCTION_UNSPECIFIED = 0;
  SQUASH_FUNCTION_TANH_Z_SCORE = 1;
}
```

## Required Values

- `spec_version` is `1`.
- `axis_count` is `4`.
- `axis_scales` has four positive finite `f32` values.
- `weight_scale` is positive and finite.
- `squash_stats` has four entries, each with a finite mean and positive finite standard
  deviation.
- `model_sha256`, `tokenizer_sha256`, and `pca_sha256` are 32 bytes each.
- `tokenizer_json` is the complete HuggingFace tokenizer configuration used at runtime.
- `token_records` has exactly `token_count * 10` bytes.

## Per-token record

`token_records` is a compact fixed-width byte field inside the protobuf message. Each
token record is 10 bytes, little-endian:

| Offset | Size | Field              | Type      |
| -----: | ---: | ------------------ | --------- |
|      0 |    8 | projected PCA axes | `4 * i16` |
|      8 |    2 | pooling weight     | `i16`     |

The compact record encoding keeps the semantic table small while protobuf provides the
forward/backward-compatible envelope for the runtime asset.

## Freeze status

`VOICE_V1` is locked. The Phase 7 voice tuning accepted the final sample-affecting
synthesis constants, T-52 squash finalized the tanh z-score mapping without regenerating
the asset, and T-53 learnability spread validated the final semantic/audio cluster
separation.

Any future change that alters one or more rendered output samples requires a new voice
identifier plus regenerated golden fixtures. `VOICE_V2` was the first post-V1
sample-affecting contract bump.

## VOICE_V12 class-table sidecar (`dootdoot_pos_v1.doot`)

The noun/verb class table (FR-114) is a separate, deliberately simple little-endian
binary — not protobuf — parsed by `dootdoot_core::PosTable` and serialized by
`xtask::serialize_pos_table` from the same layout constants, so writer and reader
cannot drift.

Header (78 bytes):

| Offset | Size | Field                  | Value / type      |
| -----: | ---: | ---------------------- | ----------------- |
|      0 |    8 | magic                  | `DOOTPOS1`        |
|      8 |    2 | spec version           | `u16` = 1         |
|     10 |   32 | ranking-corpus SHA-256 | pinned in `[pos]` |
|     42 |   32 | tagged-counts SHA-256  | pinned in `[pos]` |
|     74 |    4 | entry count            | `u32`             |

Entries follow, strictly sorted by surface bytes (lookups binary-search):

| Offset | Size | Field          | Type                     |
| -----: | ---: | -------------- | ------------------------ |
|      0 |    1 | class          | `u8`: 0 = noun, 1 = verb |
|      1 |    2 | surface length | `u16`                    |
|      3 |    n | surface        | UTF-8 bytes              |

Parsing validates the magic, spec version, class bytes, UTF-8, strict sort order, and
exact total length. Absent words classify `Other`. Regeneration is documented in
[`asset_regeneration.md`](asset_regeneration.md); the committed sidecar must reproduce
byte-for-byte from the committed `assets/pos/tagged_counts.tsv` snapshot.
