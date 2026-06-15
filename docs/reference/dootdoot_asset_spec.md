# Dootdoot Asset Spec

The committed runtime asset is `assets/dootdoot_asset_v1.doot`. The `.doot` file is a
Protocol Buffers message that carries every runtime byte needed for tokenization and the
semantic lookup table. Normal builds embed this single file with `include_bytes!`.

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
