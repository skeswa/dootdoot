//! Build-time dependency boundary tests.

const XTASK_MANIFEST: &str = include_str!("../Cargo.toml");
const CORE_MANIFEST: &str = include_str!("../../dootdoot-core/Cargo.toml");
const CLI_MANIFEST: &str = include_str!("../../dootdoot/Cargo.toml");

#[test]
fn build_time_dependencies_stay_in_xtask() {
    for dependency in [
        "hex",
        "model2vec-rs",
        "nalgebra",
        "num-traits",
        "safetensors",
        "serde",
        "serde_json",
        "sha2",
        "toml",
    ] {
        assert!(
            XTASK_MANIFEST.contains(dependency),
            "xtask manifest should include {dependency}",
        );
        assert!(
            !CORE_MANIFEST.contains(dependency),
            "core manifest should not include {dependency}",
        );
        assert!(
            !CLI_MANIFEST.contains(dependency),
            "cli manifest should not include {dependency}",
        );
    }
}
