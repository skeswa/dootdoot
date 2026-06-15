//! Xtask command runner.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Result, SourceFiles, SourceManifest, SourceManifestError, bb8_metrics, compute_pca_projection,
    compute_squash_stats, load_source_model, serialize_doot_asset,
};

/// Runs the selected xtask command from process arguments.
///
/// # Errors
///
/// Returns an error when the selected xtask command cannot complete.
pub fn run() -> Result<String> {
    run_with_args(std::env::args().skip(1))
}

/// Runs the selected xtask command from explicit arguments.
///
/// With no arguments, this runs the asset-generation workflow. The
/// `bb8-metrics` subcommand renders the local Phase 7 comparison report.
///
/// # Errors
///
/// Returns an error when the selected xtask command cannot complete.
pub fn run_with_args<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();

    if args.is_empty() {
        generate_doot_asset()?;

        return Ok(String::new());
    }

    let command = args.remove(0);

    match command.as_str() {
        "bb8-metrics" => bb8_metrics::run(&args),
        unknown => Err(SourceManifestError::new(format!(
            "unknown xtask command: {unknown}",
        ))),
    }
}

fn generate_doot_asset() -> Result<()> {
    let workspace = workspace_root()?;
    let manifest_path = workspace.join("assets/source_manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to read {}: {error}",
            manifest_path.display(),
        ))
    })?;
    let manifest = SourceManifest::parse(&manifest_text)?;
    let source_dir = workspace
        .join("target/source-cache")
        .join(manifest.hf_repo())
        .join(manifest.revision());
    let model = read_source_file(&source_dir, "model.safetensors")?;
    let tokenizer = read_source_file(&source_dir, "tokenizer.json")?;
    let config = read_source_file(&source_dir, "config.json")?;

    manifest.validate_sources(SourceFiles::new(
        manifest.revision(),
        &model,
        &tokenizer,
        &config,
    ))?;
    let source_model = load_source_model(&tokenizer, &model, &config)?;
    let expected_width = usize::from(manifest.hidden_dim());

    if source_model.embedding_width() != expected_width {
        return Err(SourceManifestError::new(format!(
            "source embedding width mismatch: expected {expected_width}, got {}",
            source_model.embedding_width(),
        )));
    }

    let projection = compute_pca_projection(&source_model, 4)?;
    let squash_stats = compute_squash_stats(&source_model, &projection)?;
    let asset = serialize_doot_asset(
        &source_model,
        &projection,
        &squash_stats,
        &manifest,
        &tokenizer,
    )?;
    let generated_dir = workspace.join("target/generated");
    fs::create_dir_all(&generated_dir).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to create {}: {error}",
            generated_dir.display(),
        ))
    })?;
    let generated_path = generated_dir.join("dootdoot_asset_v1.doot");
    fs::write(&generated_path, asset).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to write {}: {error}",
            generated_path.display(),
        ))
    })?;

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| SourceManifestError::new("failed to resolve workspace root"))
}

fn read_source_file(source_dir: &Path, file_name: &str) -> Result<Vec<u8>> {
    let path = source_dir.join(file_name);

    fs::read(&path).map_err(|error| {
        SourceManifestError::new(format!("failed to read {}: {error}", path.display()))
    })
}
