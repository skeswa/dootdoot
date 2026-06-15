//! Semantic mapping from tokenizer IDs to perceptual axes.

use thiserror::Error;

use crate::{
    FORMAT_AXIS_COUNT, FORMAT_TOKEN_RECORD_BYTES, FormatArtifact, FormatSquashFunction,
    embedded_format_v1, tanh,
};

/// Maps tokenizer IDs to baked semantic vectors and pooling weights.
#[derive(Debug, Clone)]
pub struct Mapping<'a> {
    format: FormatArtifact<'a>,
}

/// Gives one dequantized token vector and its pooling weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenVector {
    axes: [f64; FORMAT_AXIS_COUNT],
    weight: f64,
}

/// Gives a pooled sequence baseline vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PooledVector {
    axes: [f64; FORMAT_AXIS_COUNT],
}

/// Gives a bounded squashed semantic vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquashedVector {
    axes: [f64; FORMAT_AXIS_COUNT],
}

/// Gives the frozen per-axis modulation depths in pitch/vowel/contour/warble
/// order.
pub const KNOB_MODULATION_DEPTHS: [f64; FORMAT_AXIS_COUNT] = [0.85, 0.90, 1.10, 1.20];

/// Gives the frozen per-axis knob bounds in pitch/vowel/contour/warble order.
pub const KNOB_BOUNDS: [KnobBounds; FORMAT_AXIS_COUNT] = [
    KnobBounds::new(-1.0, 1.0),
    KnobBounds::new(-1.0, 1.0),
    KnobBounds::new(-1.0, 1.0),
    KnobBounds::new(-1.0, 1.0),
];

/// Gives one frozen knob range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobBounds {
    lower: f64,
    upper: f64,
}

/// Gives one syllable's semantic knob row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobSet {
    axes: [f64; FORMAT_AXIS_COUNT],
}

/// Reports why a token ID could not be mapped.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct MappingError {
    message: String,
}

impl<'a> Mapping<'a> {
    /// Builds a mapping from a parsed format artifact.
    pub fn from_format(format: FormatArtifact<'a>) -> Self {
        Self { format }
    }

    /// Looks up and dequantizes one tokenizer ID.
    ///
    /// # Errors
    ///
    /// Returns an error when `token_id` is outside the baked record table or
    /// when record offsets overflow platform pointer sizes.
    pub fn lookup(&self, token_id: u32) -> Result<TokenVector, MappingError> {
        let token_index = usize::try_from(token_id)
            .map_err(|error| MappingError::new(format!("token ID does not fit usize: {error}")))?;

        if token_index >= self.format.token_count() {
            return Err(MappingError::new(format!(
                "token ID {token_id} is outside mapping table with {} records",
                self.format.token_count(),
            )));
        }

        let start = token_index
            .checked_mul(FORMAT_TOKEN_RECORD_BYTES)
            .ok_or_else(|| MappingError::new("mapping record offset overflow"))?;
        let end = start
            .checked_add(FORMAT_TOKEN_RECORD_BYTES)
            .ok_or_else(|| MappingError::new("mapping record end overflow"))?;
        let record = self
            .format
            .record_bytes()
            .get(start..end)
            .ok_or_else(|| MappingError::new("mapping record is missing from artifact"))?;
        let axes = [
            dequantize(read_i16(record, 0)?, self.format.axis_scales()[0]),
            dequantize(read_i16(record, 2)?, self.format.axis_scales()[1]),
            dequantize(read_i16(record, 4)?, self.format.axis_scales()[2]),
            dequantize(read_i16(record, 6)?, self.format.axis_scales()[3]),
        ];
        let weight = dequantize(read_i16(record, 8)?, self.format.weight_scale());

        Ok(TokenVector { axes, weight })
    }

    /// Returns the number of token records in the mapping table.
    pub fn token_count(&self) -> usize {
        self.format.token_count()
    }

    /// Applies the frozen axis squash to one token vector.
    pub fn squash_token(&self, token: TokenVector) -> SquashedVector {
        self.squash_axes(token.axes())
    }

    /// Applies the frozen axis squash to a pooled sequence baseline.
    pub fn squash_pooled(&self, pooled: PooledVector) -> SquashedVector {
        self.squash_axes(pooled.axes())
    }

    fn squash_axes(&self, input_axes: [f64; FORMAT_AXIS_COUNT]) -> SquashedVector {
        match self.format.squash_function() {
            FormatSquashFunction::TanhZScore => {
                let mut axes = [0.0_f64; FORMAT_AXIS_COUNT];

                for (squashed_axis, (input_axis, stats)) in axes
                    .iter_mut()
                    .zip(input_axes.into_iter().zip(self.format.squash_stats()))
                {
                    *squashed_axis = tanh((input_axis - stats.mean()) / stats.standard_deviation());
                }

                SquashedVector { axes }
            }
        }
    }
}

impl TokenVector {
    /// Builds a token vector from dequantized axis and weight values.
    pub fn new(axes: [f64; FORMAT_AXIS_COUNT], weight: f64) -> Self {
        Self { axes, weight }
    }

    /// Returns dequantized semantic axes in fixed `VOICE_V1` order.
    pub fn axes(&self) -> [f64; FORMAT_AXIS_COUNT] {
        self.axes
    }

    /// Returns the dequantized pooling weight.
    pub fn weight(&self) -> f64 {
        self.weight
    }
}

impl PooledVector {
    /// Returns the pooled semantic axes in fixed `VOICE_V1` order.
    pub fn axes(&self) -> [f64; FORMAT_AXIS_COUNT] {
        self.axes
    }
}

impl SquashedVector {
    /// Builds a squashed vector from bounded axis values.
    pub fn new(axes: [f64; FORMAT_AXIS_COUNT]) -> Self {
        Self { axes }
    }

    /// Returns the squashed axes in fixed `VOICE_V1` order.
    pub fn axes(&self) -> [f64; FORMAT_AXIS_COUNT] {
        self.axes
    }
}

impl KnobBounds {
    /// Builds a knob bound pair.
    pub const fn new(lower: f64, upper: f64) -> Self {
        Self { lower, upper }
    }

    /// Returns the lower bound.
    pub fn lower(&self) -> f64 {
        self.lower
    }

    /// Returns the upper bound.
    pub fn upper(&self) -> f64 {
        self.upper
    }
}

impl KnobSet {
    /// Builds a knob row from already-bounded axes.
    pub(crate) fn from_axes(axes: [f64; FORMAT_AXIS_COUNT]) -> Self {
        let mut bounded_axes = [0.0_f64; FORMAT_AXIS_COUNT];

        for (index, axis) in bounded_axes.iter_mut().enumerate() {
            let bounds = KNOB_BOUNDS[index];
            *axis = axes[index].clamp(bounds.lower(), bounds.upper());
        }

        Self { axes: bounded_axes }
    }

    /// Returns semantic knobs in pitch/vowel/contour/warble order.
    pub fn axes(&self) -> [f64; FORMAT_AXIS_COUNT] {
        self.axes
    }

    /// Returns the pitch-center knob.
    pub fn pitch_center(&self) -> f64 {
        self.axes[0]
    }

    /// Returns the vowel/formant-position knob.
    pub fn vowel_position(&self) -> f64 {
        self.axes[1]
    }

    /// Returns the contour/glide-shape knob.
    pub fn contour(&self) -> f64 {
        self.axes[2]
    }

    /// Returns the warble-depth knob.
    pub fn warble_depth(&self) -> f64 {
        self.axes[3]
    }
}

impl MappingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Loads the embedded `VOICE_V1` mapping table.
///
/// # Errors
///
/// Returns an error if the committed format artifact cannot be parsed.
pub fn embedded_mapping() -> Result<Mapping<'static>, MappingError> {
    embedded_format_v1()
        .map(Mapping::from_format)
        .map_err(|error| {
            MappingError::new(format!("failed to load embedded mapping format: {error}"))
        })
}

/// Pools dequantized token vectors into a sequence baseline.
///
/// The formula is `(1 / token_count) * sum(weight_i * axes_i)`. This
/// intentionally does not apply the `model2vec.encode()` L2 normalization step.
///
/// # Errors
///
/// Returns an error when the sequence is empty or its length cannot be
/// represented for deterministic floating-point division.
pub fn pool_sequence(tokens: &[TokenVector]) -> Result<PooledVector, MappingError> {
    if tokens.is_empty() {
        return Err(MappingError::new("cannot pool an empty token sequence"));
    }

    let denominator = f64::from(u32::try_from(tokens.len()).map_err(|error| {
        MappingError::new(format!("token sequence length does not fit u32: {error}"))
    })?);
    let mut axes = [0.0_f64; FORMAT_AXIS_COUNT];

    for token in tokens {
        for (pooled_axis, token_axis) in axes.iter_mut().zip(token.axes()) {
            *pooled_axis += token_axis * token.weight();
        }
    }

    for axis in &mut axes {
        *axis /= denominator;
    }

    Ok(PooledVector { axes })
}

/// Assembles one per-syllable knob row from squashed baseline and token values.
pub fn assemble_knobs(baseline: SquashedVector, token: SquashedVector) -> KnobSet {
    let baseline_axes = baseline.axes();
    let token_axes = token.axes();
    let mut axes = [0.0_f64; FORMAT_AXIS_COUNT];

    for (index, axis) in axes.iter_mut().enumerate() {
        let bounds = KNOB_BOUNDS[index];
        *axis = (baseline_axes[index]
            + (KNOB_MODULATION_DEPTHS[index] * (token_axes[index] - baseline_axes[index])))
            .clamp(bounds.lower(), bounds.upper());
    }

    KnobSet { axes }
}

/// Assembles one knob row per voiced syllable token.
pub fn assemble_knob_sequence(baseline: SquashedVector, tokens: &[SquashedVector]) -> Vec<KnobSet> {
    tokens
        .iter()
        .copied()
        .map(|token| assemble_knobs(baseline, token))
        .collect()
}

fn dequantize(code: i16, scale: f32) -> f64 {
    f64::from(code) * f64::from(scale)
}

fn read_i16(bytes: &[u8], offset: usize) -> Result<i16, MappingError> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| MappingError::new("mapping record field offset overflow"))?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| MappingError::new("mapping record field is missing"))?;
    let mut raw = [0_u8; 2];
    raw.copy_from_slice(field);

    Ok(i16::from_le_bytes(raw))
}
