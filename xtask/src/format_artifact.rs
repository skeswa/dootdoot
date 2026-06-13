//! `format_v1.bin` serialization.

use dootdoot_core::{
    FORMAT_AXIS_COUNT, FORMAT_HEADER_BYTES, FORMAT_MAGIC, FORMAT_SCALE_COUNT,
    FORMAT_SQUASH_STATS_PER_AXIS, FORMAT_TOKEN_RECORD_BYTES, FORMAT_VERSION_NUMBER,
};
use num_traits::ToPrimitive;
use sha2::{Digest, Sha256};

use crate::{
    PcaProjection, Result, SourceManifest, SourceManifestError, SourceModel, SquashFunction,
    SquashStats,
};

/// Serializes a complete `format_v1.bin` artifact to bytes.
///
/// # Errors
///
/// Returns an error when projection dimensions are inconsistent, hashes are
/// malformed, numeric fields exceed the format range, or quantization cannot be
/// represented.
pub fn serialize_format_artifact(
    source_model: &SourceModel,
    projection: &PcaProjection,
    squash_stats: &SquashStats,
    manifest: &SourceManifest,
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
    let mut bytes = Vec::with_capacity(
        FORMAT_HEADER_BYTES + (source_model.token_count() * FORMAT_TOKEN_RECORD_BYTES),
    );

    write_header(
        &mut bytes,
        source_model,
        &axis_scales,
        weight_scale,
        squash_stats,
        manifest,
        &pca_hash(projection),
    )?;
    write_records(
        &mut bytes,
        source_model,
        &projected,
        &axis_scales,
        weight_scale,
    )?;

    Ok(bytes)
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
    if projection.axis_count() != FORMAT_AXIS_COUNT {
        return Err(SourceManifestError::new(format!(
            "format requires {FORMAT_AXIS_COUNT} PCA axes, got {}",
            projection.axis_count(),
        )));
    }

    if squash_stats.axis_count() != FORMAT_AXIS_COUNT {
        return Err(SourceManifestError::new(format!(
            "format requires {FORMAT_AXIS_COUNT} squash axes, got {}",
            squash_stats.axis_count(),
        )));
    }

    if source_model.embedding_width() != projection.source_width() {
        return Err(SourceManifestError::new(format!(
            "format source/projection width mismatch: source {}, projection {}",
            source_model.embedding_width(),
            projection.source_width(),
        )));
    }

    Ok(())
}

fn projected_values(source_model: &SourceModel, projection: &PcaProjection) -> Vec<f64> {
    let source_width = source_model.embedding_width();
    let mut projected = Vec::with_capacity(source_model.token_count() * FORMAT_AXIS_COUNT);

    for row in source_model.embeddings().chunks_exact(source_width) {
        for axis in 0..FORMAT_AXIS_COUNT {
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

fn axis_scales(projected: &[f64]) -> Result<[f32; FORMAT_AXIS_COUNT]> {
    let mut maxima = [0.0_f64; FORMAT_AXIS_COUNT];

    for token_axes in projected.chunks_exact(FORMAT_AXIS_COUNT) {
        for (maximum, value) in maxima.iter_mut().zip(token_axes) {
            *maximum = maximum.max(value.abs());
        }
    }

    let mut scales = [0.0_f32; FORMAT_AXIS_COUNT];

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

fn write_header(
    bytes: &mut Vec<u8>,
    source_model: &SourceModel,
    axis_scales: &[f32; FORMAT_AXIS_COUNT],
    weight_scale: f32,
    squash_stats: &SquashStats,
    manifest: &SourceManifest,
    pca_hash: &[u8; 32],
) -> Result<()> {
    bytes.extend_from_slice(&FORMAT_MAGIC);
    push_u32(bytes, usize_to_u32(FORMAT_HEADER_BYTES)?);
    push_u32(bytes, FORMAT_VERSION_NUMBER);
    push_u32(bytes, usize_to_u32(source_model.token_count())?);
    push_u32(bytes, usize_to_u32(FORMAT_AXIS_COUNT)?);

    for scale in axis_scales {
        push_f32(bytes, *scale);
    }

    push_f32(bytes, weight_scale);
    push_u32(bytes, squash_function_id(squash_stats.function()));

    for axis in squash_stats.axes() {
        push_f64(bytes, axis.mean());
        push_f64(bytes, axis.standard_deviation());
    }

    debug_assert_eq!(FORMAT_SCALE_COUNT, FORMAT_AXIS_COUNT + 1);
    debug_assert_eq!(FORMAT_SQUASH_STATS_PER_AXIS, 2);

    bytes.extend_from_slice(&decode_hash(manifest.model_sha256(), "model")?);
    bytes.extend_from_slice(&decode_hash(manifest.tokenizer_sha256(), "tokenizer")?);
    bytes.extend_from_slice(pca_hash);

    Ok(())
}

fn write_records(
    bytes: &mut Vec<u8>,
    source_model: &SourceModel,
    projected: &[f64],
    axis_scales: &[f32; FORMAT_AXIS_COUNT],
    weight_scale: f32,
) -> Result<()> {
    for (token_index, token_axes) in projected.chunks_exact(FORMAT_AXIS_COUNT).enumerate() {
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

fn squash_function_id(function: SquashFunction) -> u32 {
    match function {
        SquashFunction::TanhZScore => 1,
    }
}

fn decode_hash(hash: &str, name: &str) -> Result<[u8; 32]> {
    let mut bytes = [0_u8; 32];

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

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn usize_to_u32(value: usize) -> Result<u32> {
    u32::try_from(value)
        .map_err(|error| SourceManifestError::new(format!("value does not fit u32: {error}")))
}
