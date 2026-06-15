# Cross-platform Determinism

The active format contract is verified on the platforms covered by the CI matrix: Linux
(`ubuntu-latest`) and macOS (`macos-latest`).

The CI job runs `cargo test -p dootdoot-core --test golden_wav_hashes` on both
platforms. That test renders the fixed golden corpus, serializes the canonical WAV bytes,
and compares each SHA-256 digest with `golden_wav_hashes.tsv`.

Passing CI means Linux and macOS produce identical hashes for every golden corpus entry.
Any divergence is a sample-affecting determinism failure and must be investigated in the
math, synthesis, quantization, or WAV serialization path before the contract can be
considered green again.
