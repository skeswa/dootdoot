//! Source model loading and extraction.

use model2vec_rs::model::StaticModel;
use safetensors::{
    SafeTensors,
    tensor::{Dtype, TensorView},
};

use crate::{Result, SourceManifestError};

/// Holds extracted source embeddings and pooling weights.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceModel {
    embeddings: Vec<f32>,
    weights: Vec<f32>,
    token_count: usize,
    embedding_width: usize,
}

impl SourceModel {
    /// Creates a source model from row-major embeddings and one weight per
    /// token.
    ///
    /// # Errors
    ///
    /// Returns an error when the row-major embedding length or weight count
    /// does not match the provided shape.
    pub fn from_parts(
        embeddings: Vec<f32>,
        weights: Vec<f32>,
        token_count: usize,
        embedding_width: usize,
    ) -> Result<Self> {
        let expected_embedding_len = token_count
            .checked_mul(embedding_width)
            .ok_or_else(|| SourceManifestError::new("source model shape overflowed"))?;

        if embeddings.len() != expected_embedding_len {
            return Err(SourceManifestError::new(format!(
                "source embedding length mismatch: expected {expected_embedding_len}, got {}",
                embeddings.len(),
            )));
        }

        if weights.len() != token_count {
            return Err(SourceManifestError::new(format!(
                "source weight length mismatch: expected {token_count}, got {}",
                weights.len(),
            )));
        }

        Ok(Self {
            embeddings,
            weights,
            token_count,
            embedding_width,
        })
    }

    /// Returns all token embeddings in row-major order.
    #[must_use]
    pub fn embeddings(&self) -> &[f32] {
        &self.embeddings
    }

    /// Returns one pooling weight per token.
    #[must_use]
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    /// Returns the number of token rows in the source model.
    #[must_use]
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Returns the source embedding width.
    #[must_use]
    pub fn embedding_width(&self) -> usize {
        self.embedding_width
    }
}

/// Loads and extracts source model embeddings and weights.
///
/// # Errors
///
/// Returns an error when `model2vec-rs` cannot load the source model, when
/// safetensors parsing fails, or when the expected embedding/weight tensors are
/// absent or malformed.
pub fn load_source_model(
    tokenizer_bytes: &[u8],
    model_bytes: &[u8],
    config_bytes: &[u8],
) -> Result<SourceModel> {
    let _model = StaticModel::from_bytes(tokenizer_bytes, model_bytes, config_bytes, Some(false))
        .map_err(|error| {
        SourceManifestError::new(format!("model2vec-rs failed to load source: {error}"))
    })?;
    let tensors = SafeTensors::deserialize(model_bytes).map_err(|error| {
        SourceManifestError::new(format!("failed to parse safetensors: {error}"))
    })?;
    let embeddings = tensors
        .tensor("embeddings")
        .or_else(|_| tensors.tensor("0"))
        .or_else(|_| tensors.tensor("embedding.weight"))
        .map_err(|error| {
            SourceManifestError::new(format!("embeddings tensor not found: {error}"))
        })?;
    let [token_count, embedding_width] = tensor_shape_2d(&embeddings, "embeddings")?;
    let embeddings = decode_f32_tensor(&embeddings, "embeddings")?;
    let expected_embedding_len = token_count
        .checked_mul(embedding_width)
        .ok_or_else(|| SourceManifestError::new("embedding tensor shape overflowed"))?;

    if embeddings.len() != expected_embedding_len {
        return Err(SourceManifestError::new(format!(
            "embedding tensor length mismatch: expected {expected_embedding_len}, got {}",
            embeddings.len(),
        )));
    }

    let weights = match tensors.tensor("weights") {
        Ok(weights) => decode_weights(&weights, token_count)?,
        Err(_) => vec![1.0; token_count],
    };

    SourceModel::from_parts(embeddings, weights, token_count, embedding_width)
}

fn tensor_shape_2d(tensor: &TensorView<'_>, name: &str) -> Result<[usize; 2]> {
    <[usize; 2]>::try_from(tensor.shape()).map_err(|error| {
        SourceManifestError::new(format!("{name} tensor should be 2-D: {error:?}"))
    })
}

fn tensor_shape_1d(tensor: &TensorView<'_>, name: &str) -> Result<usize> {
    let [len] = <[usize; 1]>::try_from(tensor.shape()).map_err(|error| {
        SourceManifestError::new(format!("{name} tensor should be 1-D: {error:?}"))
    })?;

    Ok(len)
}

fn decode_weights(tensor: &TensorView<'_>, token_count: usize) -> Result<Vec<f32>> {
    let len = tensor_shape_1d(tensor, "weights")?;

    if len != token_count {
        return Err(SourceManifestError::new(format!(
            "weights length mismatch: expected {token_count}, got {len}",
        )));
    }

    decode_f32_tensor(tensor, "weights")
}

fn decode_f32_tensor(tensor: &TensorView<'_>, name: &str) -> Result<Vec<f32>> {
    if tensor.dtype() != Dtype::F32 {
        return Err(SourceManifestError::new(format!(
            "{name} tensor should be F32, got {:?}",
            tensor.dtype(),
        )));
    }

    tensor
        .data()
        .chunks_exact(4)
        .map(|chunk| {
            let bytes = <[u8; 4]>::try_from(chunk).map_err(|error| {
                SourceManifestError::new(format!("{name} tensor has invalid f32 chunk: {error}"))
            })?;

            Ok(f32::from_le_bytes(bytes))
        })
        .collect()
}
