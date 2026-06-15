# `format_v1.bin` Layout

All integer and floating-point fields are little-endian. The runtime file stores projected
values only; it does not store the source embedding matrix or the PCA matrix itself.

## Header

| Offset | Size | Field                | Type          |
| -----: | ---: | -------------------- | ------------- |
|      0 |    8 | magic `DOOTV1\0\0`   | bytes         |
|      8 |    4 | header byte length   | `u32`         |
|     12 |    4 | format version `1`   | `u32`         |
|     16 |    4 | vocab size           | `u32`         |
|     20 |    4 | axis count `4`       | `u32`         |
|     24 |   20 | axis scales + weight | `5 * f32`     |
|     44 |    4 | squash function id   | `u32`         |
|     48 |   64 | squash stats         | `4 * 2 * f64` |
|    112 |   32 | model SHA-256        | bytes         |
|    144 |   32 | tokenizer SHA-256    | bytes         |
|    176 |   32 | PCA matrix SHA-256   | bytes         |

Header size: 208 bytes.

## Per-token record

Each token record is 10 bytes:

| Offset | Size | Field              | Type      |
| -----: | ---: | ------------------ | --------- |
|      0 |    8 | projected PCA axes | `4 * i16` |
|      8 |    2 | pooling weight     | `i16`     |

The record count must equal the header vocab size.

## Freeze status

`FORMAT_V1` is locked. The Phase 7 voice tuning accepted the final sample-affecting
synthesis constants, T-52 squash finalized the tanh z-score mapping without regenerating
the artifact, and T-53 learnability spread validated the final semantic/audio cluster
separation.

Any future change that alters one or more rendered output samples requires a new
identifier, `FORMAT_V2`, plus regenerated golden fixtures.
