//! Principal component projection for source embeddings.

use nalgebra::{DMatrix, SymmetricEigen};

use crate::{Result, SourceManifestError, SourceModel};

/// Holds a PCA projection matrix and associated centering statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct PcaProjection {
    components: Vec<f64>,
    eigenvalues: Vec<f64>,
    means: Vec<f64>,
    source_width: usize,
    axis_count: usize,
}

impl PcaProjection {
    /// Creates a PCA projection from raw parts.
    ///
    /// # Errors
    ///
    /// Returns an error when component, eigenvalue, or mean lengths do not
    /// match the provided shape.
    pub fn from_parts(
        components: Vec<f64>,
        eigenvalues: Vec<f64>,
        means: Vec<f64>,
        source_width: usize,
        axis_count: usize,
    ) -> Result<Self> {
        let expected_components = source_width
            .checked_mul(axis_count)
            .ok_or_else(|| SourceManifestError::new("PCA projection shape overflowed"))?;

        if components.len() != expected_components {
            return Err(SourceManifestError::new(format!(
                "PCA component length mismatch: expected {expected_components}, got {}",
                components.len(),
            )));
        }

        if eigenvalues.len() != axis_count {
            return Err(SourceManifestError::new(format!(
                "PCA eigenvalue length mismatch: expected {axis_count}, got {}",
                eigenvalues.len(),
            )));
        }

        if means.len() != source_width {
            return Err(SourceManifestError::new(format!(
                "PCA mean length mismatch: expected {source_width}, got {}",
                means.len(),
            )));
        }

        Ok(Self {
            components,
            eigenvalues,
            means,
            source_width,
            axis_count,
        })
    }

    /// Canonicalizes each component sign by making its largest loading
    /// positive.
    pub fn canonicalize_component_signs(&mut self) {
        for axis in 0..self.axis_count {
            let start = axis * self.source_width;
            let end = start + self.source_width;
            let component = &mut self.components[start..end];
            let mut pivot_index = 0_usize;
            let mut pivot_magnitude = 0.0_f64;

            for (index, value) in component.iter().enumerate() {
                let magnitude = value.abs();

                if magnitude > pivot_magnitude {
                    pivot_index = index;
                    pivot_magnitude = magnitude;
                }
            }

            if pivot_magnitude > 0.0 && component[pivot_index].is_sign_negative() {
                for value in component {
                    *value = -*value;
                }
            }
        }
    }

    /// Returns PCA components in axis-major order.
    #[must_use]
    pub fn components(&self) -> &[f64] {
        &self.components
    }

    /// Returns retained eigenvalues in descending order.
    #[must_use]
    pub fn eigenvalues(&self) -> &[f64] {
        &self.eigenvalues
    }

    /// Returns one centering mean per source dimension.
    #[must_use]
    pub fn means(&self) -> &[f64] {
        &self.means
    }

    /// Returns the source embedding width.
    #[must_use]
    pub fn source_width(&self) -> usize {
        self.source_width
    }

    /// Returns the number of retained principal axes.
    #[must_use]
    pub fn axis_count(&self) -> usize {
        self.axis_count
    }
}

/// Computes a top-k PCA projection from source embeddings.
///
/// # Errors
///
/// Returns an error when the requested axis count is invalid or the source
/// model does not contain enough token rows to estimate covariance.
pub fn compute_pca_projection(
    source_model: &SourceModel,
    axis_count: usize,
) -> Result<PcaProjection> {
    let token_count = source_model.token_count();
    let source_width = source_model.embedding_width();

    if axis_count == 0 {
        return Err(SourceManifestError::new(
            "PCA axis count must be greater than zero",
        ));
    }

    if axis_count > source_width {
        return Err(SourceManifestError::new(format!(
            "PCA axis count {axis_count} exceeds source width {source_width}",
        )));
    }

    if token_count < 2 {
        return Err(SourceManifestError::new(
            "PCA requires at least two token embeddings",
        ));
    }

    let means = column_means(source_model)?;
    let centered = centered_embeddings(source_model, &means);
    let matrix = DMatrix::from_row_slice(token_count, source_width, &centered);
    let covariance_denominator = usize_to_f64(token_count - 1)?;
    let covariance = matrix.transpose() * matrix / covariance_denominator;
    let eigen = SymmetricEigen::new(covariance);
    let mut pairs: Vec<(f64, Vec<f64>)> = eigen
        .eigenvalues
        .iter()
        .enumerate()
        .map(|(index, eigenvalue)| {
            (
                *eigenvalue,
                eigen.eigenvectors.column(index).iter().copied().collect(),
            )
        })
        .collect();

    pairs.sort_by(|left, right| right.0.total_cmp(&left.0));

    let mut eigenvalues = Vec::with_capacity(axis_count);
    let mut components = Vec::with_capacity(axis_count * source_width);

    for (eigenvalue, component) in pairs.into_iter().take(axis_count) {
        eigenvalues.push(eigenvalue);
        components.extend(component);
    }

    let mut projection =
        PcaProjection::from_parts(components, eigenvalues, means, source_width, axis_count)?;
    projection.canonicalize_component_signs();

    Ok(projection)
}

fn column_means(source_model: &SourceModel) -> Result<Vec<f64>> {
    let source_width = source_model.embedding_width();
    let token_count = source_model.token_count();
    let token_count = usize_to_f64(token_count)?;
    let mut means = vec![0.0; source_width];

    for row in source_model.embeddings().chunks_exact(source_width) {
        for (mean, value) in means.iter_mut().zip(row) {
            *mean += f64::from(*value);
        }
    }

    for mean in &mut means {
        *mean /= token_count;
    }

    Ok(means)
}

fn centered_embeddings(source_model: &SourceModel, means: &[f64]) -> Vec<f64> {
    let source_width = source_model.embedding_width();
    let mut centered = Vec::with_capacity(source_model.embeddings().len());

    for row in source_model.embeddings().chunks_exact(source_width) {
        centered.extend(
            row.iter()
                .zip(means)
                .map(|(value, mean)| f64::from(*value) - mean),
        );
    }

    centered
}

fn usize_to_f64(value: usize) -> Result<f64> {
    let value = u32::try_from(value)
        .map_err(|error| SourceManifestError::new(format!("value does not fit u32: {error}")))?;

    Ok(f64::from(value))
}
