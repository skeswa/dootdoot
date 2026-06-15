//! Dootdoot asset spec serialization.

use dootdoot_core::{
    DOOT_ASSET_AXIS_COUNT, DOOT_ASSET_HASH_BYTES, DOOT_ASSET_TOKEN_RECORD_BYTES, DootAsset,
    DootAssetHashes, DootAssetParts, DootAssetScales, DootAssetSquashAxisStats,
    DootAssetSquashFunction,
};
use num_traits::ToPrimitive;
use sha2::{Digest, Sha256};

use crate::{
    PcaProjection, Result, SourceManifest, SourceManifestError, SourceModel, SquashFunction,
    SquashStats,
};

/// Serializes a complete `.doot` asset to protobuf bytes.
///
/// # Errors
///
/// Returns an error when projection dimensions are inconsistent, hashes are
/// malformed, numeric fields exceed the asset spec range, or quantization
/// cannot be represented.
pub fn serialize_doot_asset(
    source_model: &SourceModel,
    projection: &PcaProjection,
    squash_stats: &SquashStats,
    manifest: &SourceManifest,
    tokenizer_json: &[u8],
) -> Result<Vec<u8>> {
    validate_artifact_inputs(source_model, projection, squash_stats)?;

    let projected = projected_values(source_model, projection);
    let axis_scales = axis_scales(&projected)?;
    let weight_scale = scale_from_max(
        source_model
            .weights()
            .iter()
            .map(|weight| f64::from(*weight).abs())
            .fold(0.0, f64::max),
    )?;
    let mut record_bytes =
        Vec::with_capacity(source_model.token_count() * DOOT_ASSET_TOKEN_RECORD_BYTES);

    write_records(
        &mut record_bytes,
        source_model,
        &projected,
        &axis_scales,
        weight_scale,
    )?;

    let parts = DootAssetParts::new(
        usize_to_u32(source_model.token_count())?,
        DootAssetScales::new(axis_scales, weight_scale),
        doot_squash_function(squash_stats.function()),
        doot_squash_stats(squash_stats),
        DootAssetHashes::new(
            decode_hash(manifest.model_sha256(), "model")?,
            decode_hash(manifest.tokenizer_sha256(), "tokenizer")?,
            pca_hash(projection),
        ),
        tokenizer_json.to_vec(),
        record_bytes,
    );

    DootAsset::from_parts(parts)
        .and_then(|asset| asset.to_protobuf_bytes())
        .map_err(|error| SourceManifestError::new(format!("failed to encode .doot asset: {error}")))
}

/// Quantizes a value with symmetric signed int16 half-even rounding.
///
/// # Errors
///
/// Returns an error when scale is not positive/finite or the rounded value
/// cannot be represented after clamping.
pub fn quantize_symmetric_i16(value: f64, scale: f64) -> Result<i16> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(SourceManifestError::new(format!(
            "quantization scale must be positive and finite, got {scale}",
        )));
    }

    let scaled = value / scale;
    let rounded = round_half_to_even(scaled)?;
    let clamped = rounded.clamp(-32767.0, 32767.0);

    clamped.to_i16().ok_or_else(|| {
        SourceManifestError::new(format!("quantized value is outside i16 range: {clamped}"))
    })
}

/// Dequantizes a signed int16 code with the provided scale.
#[must_use]
pub fn dequantize_i16(code: i16, scale: f64) -> f64 {
    f64::from(code) * scale
}

fn validate_artifact_inputs(
    source_model: &SourceModel,
    projection: &PcaProjection,
    squash_stats: &SquashStats,
) -> Result<()> {
    if projection.axis_count() != DOOT_ASSET_AXIS_COUNT {
        return Err(SourceManifestError::new(format!(
            "dootdoot asset spec requires {DOOT_ASSET_AXIS_COUNT} PCA axes, got {}",
            projection.axis_count(),
        )));
    }

    if squash_stats.axis_count() != DOOT_ASSET_AXIS_COUNT {
        return Err(SourceManifestError::new(format!(
            "dootdoot asset spec requires {DOOT_ASSET_AXIS_COUNT} squash axes, got {}",
            squash_stats.axis_count(),
        )));
    }

    if source_model.embedding_width() != projection.source_width() {
        return Err(SourceManifestError::new(format!(
            "dootdoot asset source/projection width mismatch: source {}, projection {}",
            source_model.embedding_width(),
            projection.source_width(),
        )));
    }

    Ok(())
}

fn projected_values(source_model: &SourceModel, projection: &PcaProjection) -> Vec<f64> {
    let source_width = source_model.embedding_width();
    let mut projected = Vec::with_capacity(source_model.token_count() * DOOT_ASSET_AXIS_COUNT);

    for row in source_model.embeddings().chunks_exact(source_width) {
        for axis in 0..DOOT_ASSET_AXIS_COUNT {
            let component_start = axis * source_width;
            let component_end = component_start + source_width;
            let component = &projection.components()[component_start..component_end];
            let value = row
                .iter()
                .zip(projection.means())
                .zip(component)
                .map(|((source, mean), loading)| (f64::from(*source) - mean) * loading)
                .sum();
            projected.push(value);
        }
    }

    projected
}

fn axis_scales(projected: &[f64]) -> Result<[f32; DOOT_ASSET_AXIS_COUNT]> {
    let mut maxima = [0.0_f64; DOOT_ASSET_AXIS_COUNT];

    for token_axes in projected.chunks_exact(DOOT_ASSET_AXIS_COUNT) {
        for (maximum, value) in maxima.iter_mut().zip(token_axes) {
            *maximum = maximum.max(value.abs());
        }
    }

    let mut scales = [0.0_f32; DOOT_ASSET_AXIS_COUNT];

    for (scale, maximum) in scales.iter_mut().zip(maxima) {
        *scale = scale_from_max(maximum)?;
    }

    Ok(scales)
}

fn scale_from_max(maximum: f64) -> Result<f32> {
    let scale = if maximum <= 0.0 {
        1.0
    } else {
        maximum / 32767.0
    };

    scale.to_f32().ok_or_else(|| {
        SourceManifestError::new(format!("scale cannot be represented as f32: {scale}"))
    })
}

fn write_records(
    bytes: &mut Vec<u8>,
    source_model: &SourceModel,
    projected: &[f64],
    axis_scales: &[f32; DOOT_ASSET_AXIS_COUNT],
    weight_scale: f32,
) -> Result<()> {
    for (token_index, token_axes) in projected.chunks_exact(DOOT_ASSET_AXIS_COUNT).enumerate() {
        for (value, scale) in token_axes.iter().zip(axis_scales) {
            push_i16(bytes, quantize_symmetric_i16(*value, f64::from(*scale))?);
        }

        push_i16(
            bytes,
            quantize_symmetric_i16(
                f64::from(source_model.weights()[token_index]),
                f64::from(weight_scale),
            )?,
        );
    }

    Ok(())
}

fn doot_squash_function(function: SquashFunction) -> DootAssetSquashFunction {
    match function {
        SquashFunction::TanhZScore => DootAssetSquashFunction::TanhZScore,
    }
}

fn doot_squash_stats(
    squash_stats: &SquashStats,
) -> [DootAssetSquashAxisStats; DOOT_ASSET_AXIS_COUNT] {
    let axes = squash_stats.axes();

    [
        DootAssetSquashAxisStats::new(axes[0].mean(), axes[0].standard_deviation()),
        DootAssetSquashAxisStats::new(axes[1].mean(), axes[1].standard_deviation()),
        DootAssetSquashAxisStats::new(axes[2].mean(), axes[2].standard_deviation()),
        DootAssetSquashAxisStats::new(axes[3].mean(), axes[3].standard_deviation()),
    ]
}

fn decode_hash(hash: &str, name: &str) -> Result<[u8; DOOT_ASSET_HASH_BYTES]> {
    let mut bytes = [0_u8; DOOT_ASSET_HASH_BYTES];

    hex::decode_to_slice(hash, &mut bytes)
        .map_err(|error| SourceManifestError::new(format!("invalid {name} hash: {error}")))?;

    Ok(bytes)
}

fn pca_hash(projection: &PcaProjection) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for value in projection.components() {
        hasher.update(value.to_le_bytes());
    }

    let digest = hasher.finalize();
    let mut hash = [0_u8; 32];
    hash.copy_from_slice(&digest);
    hash
}

fn round_half_to_even(value: f64) -> Result<f64> {
    let truncated = value.trunc();
    let fraction = (value - truncated).abs();

    if fraction < 0.5 {
        return Ok(truncated);
    }

    let direction = if value.is_sign_negative() { -1.0 } else { 1.0 };

    if fraction > 0.5 {
        return Ok(truncated + direction);
    }

    let truncated_integer = truncated
        .to_i64()
        .ok_or_else(|| SourceManifestError::new(format!("value cannot be rounded: {value}")))?;

    if truncated_integer % 2 == 0 {
        Ok(truncated)
    } else {
        Ok(truncated + direction)
    }
}

fn push_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn usize_to_u32(value: usize) -> Result<u32> {
    u32::try_from(value)
        .map_err(|error| SourceManifestError::new(format!("value does not fit u32: {error}")))
}
