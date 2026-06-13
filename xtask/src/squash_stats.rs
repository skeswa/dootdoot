//! Squash statistics for projected PCA values.

use crate::{PcaProjection, Result, SourceManifestError, SourceModel};

/// Identifies the runtime squash function selected for `FORMAT_V1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquashFunction {
    /// Applies tanh to a z-scored projected axis value.
    TanhZScore,
}

/// Holds frozen statistics for one projected axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisSquashStats {
    mean: f64,
    standard_deviation: f64,
}

impl AxisSquashStats {
    /// Returns the projected axis mean.
    #[must_use]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Returns the projected axis population standard deviation.
    #[must_use]
    pub fn standard_deviation(&self) -> f64 {
        self.standard_deviation
    }
}

/// Holds the selected squash function and per-axis statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct SquashStats {
    function: SquashFunction,
    axes: Vec<AxisSquashStats>,
}

impl SquashStats {
    /// Returns the selected squash function.
    #[must_use]
    pub fn function(&self) -> SquashFunction {
        self.function
    }

    /// Returns one statistics record per projected axis.
    #[must_use]
    pub fn axes(&self) -> &[AxisSquashStats] {
        &self.axes
    }

    /// Returns the number of projected axes.
    #[must_use]
    pub fn axis_count(&self) -> usize {
        self.axes.len()
    }
}

/// Computes per-axis squash statistics over projected source embeddings.
///
/// # Errors
///
/// Returns an error when the source/projection dimensions are inconsistent or
/// when the source model contains no embeddings.
pub fn compute_squash_stats(
    source_model: &SourceModel,
    projection: &PcaProjection,
) -> Result<SquashStats> {
    if source_model.embedding_width() != projection.source_width() {
        return Err(SourceManifestError::new(format!(
            "squash stats width mismatch: source {}, projection {}",
            source_model.embedding_width(),
            projection.source_width(),
        )));
    }

    if source_model.token_count() == 0 {
        return Err(SourceManifestError::new(
            "squash stats require at least one token embedding",
        ));
    }

    let axes = (0..projection.axis_count())
        .map(|axis| axis_squash_stats(source_model, projection, axis))
        .collect::<Result<Vec<_>>>()?;

    Ok(SquashStats {
        function: SquashFunction::TanhZScore,
        axes,
    })
}

fn axis_squash_stats(
    source_model: &SourceModel,
    projection: &PcaProjection,
    axis: usize,
) -> Result<AxisSquashStats> {
    let values = projected_axis_values(source_model, projection, axis);
    let count = usize_to_f64(values.len())?;
    let mean = values.iter().sum::<f64>() / count;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;

            delta * delta
        })
        .sum::<f64>()
        / count;
    let standard_deviation = variance.sqrt();
    let standard_deviation = if standard_deviation <= 0.0 {
        1.0
    } else {
        standard_deviation
    };

    Ok(AxisSquashStats {
        mean,
        standard_deviation,
    })
}

fn projected_axis_values(
    source_model: &SourceModel,
    projection: &PcaProjection,
    axis: usize,
) -> Vec<f64> {
    let source_width = source_model.embedding_width();
    let component_start = axis * source_width;
    let component_end = component_start + source_width;
    let component = &projection.components()[component_start..component_end];
    let mut values = Vec::with_capacity(source_model.token_count());

    for row in source_model.embeddings().chunks_exact(source_width) {
        let value = row
            .iter()
            .zip(projection.means())
            .zip(component)
            .map(|((source, mean), loading)| (f64::from(*source) - mean) * loading)
            .sum();
        values.push(value);
    }

    values
}

fn usize_to_f64(value: usize) -> Result<f64> {
    let value = u32::try_from(value)
        .map_err(|error| SourceManifestError::new(format!("value does not fit u32: {error}")))?;

    Ok(f64::from(value))
}
