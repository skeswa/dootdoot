//! Xtask command runner.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Result, SourceFiles, SourceManifest, SourceManifestError, compute_pca_projection,
    compute_squash_stats, load_source_model, serialize_format_artifact,
};

/// Runs the current xtask source validation step.
///
/// # Errors
///
/// Returns an error when the source manifest cannot be read, the source cache
/// is missing, or any cached source file does not match the manifest.
pub fn run() -> Result<()> {
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
    let artifact = serialize_format_artifact(&source_model, &projection, &squash_stats, &manifest)?;
    let generated_dir = workspace.join("target/generated");
    fs::create_dir_all(&generated_dir).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to create {}: {error}",
            generated_dir.display(),
        ))
    })?;
    let generated_path = generated_dir.join("format_v1.bin");
    fs::write(&generated_path, artifact).map_err(|error| {
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
