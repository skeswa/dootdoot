# Packaging

`dootdoot` v1 supports source builds and local Cargo installs from the workspace. The
project license is MIT, matching [`LICENSE`](../../LICENSE) and the Cargo package
metadata.

## Supported Install Path

```sh
cargo install --path dootdoot --locked
```

The install uses committed runtime assets through `dootdoot-core`; it does not run
`xtask` and does not need network access beyond Cargo dependency resolution.

For a clean local smoke test without writing into the user's normal Cargo bin directory:

```sh
scripts/package-smoke
target/cargo-install-smoke/bin/dootdoot --version
```

## Crates.io

The workspace is ready for package listing checks, but crates.io publication should be a
separate release decision because the binary crate depends on the core crate. Publish
`dootdoot-core` first, then publish `dootdoot` with a versioned dependency on the matching
core release.

## Homebrew and Prebuilt Binaries

No Homebrew formula or prebuilt binary workflow is committed for v1. The first packaging
surface is Cargo because it keeps the binary, committed assets, and Rust toolchain story
simple. Add Homebrew or release archives only when tag-based release automation exists,
and keep those artifacts downstream of the same locked format/golden-hash checks.
