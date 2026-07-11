//! Xtask command runner.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    PosPolicyConfig, PosSourceManifest, Result, SourceFiles, SourceManifest, SourceManifestError,
    bb8_metrics, compute_pca_projection, compute_squash_stats, derive_pos_class_table,
    load_source_model, parse_tagged_counts, serialize_doot_asset, serialize_pos_table,
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
        "pos-table" => generate_pos_table(),
        unknown => Err(SourceManifestError::new(format!(
            "unknown xtask command: {unknown}",
        ))),
    }
}

/// Bakes the `VOICE_V12` sidecar class-table asset (T-121).
///
/// Validates the committed tagged-counts snapshot against its `[pos]`
/// manifest pin (FR-114's validate-before-work), derives the class table
/// under the locked policy, serializes the sidecar payload, self-checks it
/// through the core parser, and writes `target/generated/dootdoot_pos_v1.doot`
/// for manual copy into `assets/`.
fn generate_pos_table() -> Result<String> {
    let workspace = workspace_root()?;
    let manifest_path = workspace.join("assets/source_manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to read {}: {error}",
            manifest_path.display(),
        ))
    })?;
    let manifest = PosSourceManifest::parse(&manifest_text)?;
    let snapshot_path = workspace.join("assets/pos/tagged_counts.tsv");
    let snapshot_bytes = fs::read(&snapshot_path).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to read {}: {error}",
            snapshot_path.display(),
        ))
    })?;

    manifest.validate_tagged_counts(&snapshot_bytes)?;

    let snapshot_text = String::from_utf8(snapshot_bytes).map_err(|error| {
        SourceManifestError::new(format!("tagged_counts.tsv is not utf-8: {error}"))
    })?;
    let snapshot = parse_tagged_counts(&snapshot_text)?;
    let entries = derive_pos_class_table(&snapshot, &PosPolicyConfig::default());
    let payload = serialize_pos_table(&entries, &manifest)?;

    dootdoot_core::PosTable::parse(&payload).map_err(|error| {
        SourceManifestError::new(format!(
            "generated pos table failed its own parse check: {error}",
        ))
    })?;

    let generated_dir = workspace.join("target/generated");

    fs::create_dir_all(&generated_dir).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to create {}: {error}",
            generated_dir.display(),
        ))
    })?;

    let generated_path = generated_dir.join(dootdoot_core::POS_TABLE_FILE_V1);

    fs::write(&generated_path, &payload).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to write {}: {error}",
            generated_path.display(),
        ))
    })?;

    Ok(format!(
        "wrote {} ({} entries, {} bytes)",
        generated_path.display(),
        entries.len(),
        payload.len(),
    ))
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
