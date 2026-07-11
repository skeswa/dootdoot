//! Dootdoot asset spec parsing and embedded asset access.

use prost::Message;
use thiserror::Error;

/// Identifies the first frozen sample-affecting output contract.
pub const VOICE_V1: &str = "VOICE_V1";

/// Identifies the second frozen sample-affecting output contract.
pub const VOICE_V2: &str = "VOICE_V2";

/// Identifies the phrase-continuous output contract.
pub const VOICE_V3: &str = "VOICE_V3";

/// Identifies the repeated-onset-smoothed output contract.
pub const VOICE_V4: &str = "VOICE_V4";

/// Identifies the word-attack-smoothed output contract.
pub const VOICE_V5: &str = "VOICE_V5";

/// Identifies the repeated-phrase-smoothed output contract.
pub const VOICE_V6: &str = "VOICE_V6";

/// Identifies the contextual-performance output contract.
pub const VOICE_V7: &str = "VOICE_V7";

/// Identifies the semantic-engagement and bursty-texture output contract.
pub const VOICE_V8: &str = "VOICE_V8";

/// Identifies the audible-punctuation output contract: a question, exclamation,
/// period, dash, and ellipsis each read as a distinct prosodic gesture.
pub const VOICE_V9: &str = "VOICE_V9";

/// Identifies the bidirectional-whistle vocabulary output contract: the whistle
/// can descend as well as climb (the exclamation flourish falls), accents
/// engage it harder/earlier and swoop wider, neutral text paces shorter, and
/// agitated accents can burst into the noisy band.
pub const VOICE_V10: &str = "VOICE_V10";

/// Identifies the natural-voice output contract: syllable onsets bloom in
/// rather than clicking (longer envelope attack, gentler word-onset transient),
/// per-syllable pacing breathes across a phrase (positional lilt + agogic and
/// phrase-final lengthening), a dash localizes its breathy hesitation to the
/// pre-dash word instead of the whole clause, and aspiration breath is
/// pitch-synchronously modulated over a whiter, additive source so it fuses
/// into the voice instead of reading as a separate hiss.
pub const VOICE_V11: &str = "VOICE_V11";

/// Identifies the noun/verb-recognizability output contract: content words
/// carry a systematic two-pillar class signature — a layered co-onset marker
/// (noun = broadband click/pop splash, verb = up-swept dual-sine chirp, both
/// blooming with the tonal body) and a compound `stem → class-resolution`
/// silhouette (noun settles, verb pushes; single-token words gain a derived
/// resolution syllable, multi-token words shape their last subword) at a
/// shortened compound pace. Classes come from a baked, pinned, per-surface
/// class table under the conservative ambiguity policy; unclassified words
/// render exactly as `VOICE_V11`.
pub const VOICE_V12: &str = "VOICE_V12";

/// Identifies the active sample-affecting output contract.
pub const ACTIVE_VOICE: &str = VOICE_V12;

/// Identifies the first committed dootdoot asset spec file.
pub const DOOT_ASSET_FILE_V1: &str = "dootdoot_asset_v1.doot";

/// Gives the active dootdoot asset spec version.
pub const DOOT_ASSET_SPEC_VERSION: u32 = 1;

/// Gives the number of PCA/perceptual axes stored per token.
pub const DOOT_ASSET_AXIS_COUNT: usize = 4;

/// Gives the number of SHA-256 bytes stored for each provenance hash.
pub const DOOT_ASSET_HASH_BYTES: usize = 32;

/// Gives the number of dequantization scales in the asset.
pub const DOOT_ASSET_SCALE_COUNT: usize = DOOT_ASSET_AXIS_COUNT + 1;

/// Gives the number of squash statistics stored per axis.
pub const DOOT_ASSET_SQUASH_STATS_PER_AXIS: usize = 2;

/// Gives the byte length of one compact token record.
pub const DOOT_ASSET_TOKEN_RECORD_BYTES: usize = (DOOT_ASSET_AXIS_COUNT * 2) + 2;

const EMBEDDED_DOOT_ASSET: &[u8] = include_bytes!("../../assets/dootdoot_asset_v1.doot");

/// Marks the dootdoot asset spec module in the public facade.
#[derive(Debug)]
pub struct DootAssetSpec;

/// Gives a parsed dootdoot asset spec payload.
#[derive(Debug, Clone)]
pub struct DootAsset {
    header: DootAssetHeader,
    tokenizer_json: Vec<u8>,
    record_bytes: Vec<u8>,
}

/// Gives validated scalar scales for a dootdoot asset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DootAssetScales {
    axis_scales: [f32; DOOT_ASSET_AXIS_COUNT],
    weight_scale: f32,
}

/// Gives provenance hashes for a dootdoot asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DootAssetHashes {
    model: [u8; DOOT_ASSET_HASH_BYTES],
    tokenizer: [u8; DOOT_ASSET_HASH_BYTES],
    pca: [u8; DOOT_ASSET_HASH_BYTES],
}

/// Gives the pieces needed to encode a dootdoot asset.
#[derive(Debug, Clone, PartialEq)]
pub struct DootAssetParts {
    token_count: u32,
    scales: DootAssetScales,
    squash_function: DootAssetSquashFunction,
    squash_stats: [DootAssetSquashAxisStats; DOOT_ASSET_AXIS_COUNT],
    hashes: DootAssetHashes,
    tokenizer_json: Vec<u8>,
    record_bytes: Vec<u8>,
}

/// Describes the squash function frozen into the dootdoot asset spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DootAssetSquashFunction {
    /// Applies tanh to z-scored projected axis values.
    TanhZScore,
}

/// Gives the per-axis squash statistics frozen into the dootdoot asset spec.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DootAssetSquashAxisStats {
    mean: f64,
    standard_deviation: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct DootAssetHeader {
    file_name: &'static str,
    spec_version: u32,
    token_count: usize,
    axis_scales: [f32; DOOT_ASSET_AXIS_COUNT],
    weight_scale: f32,
    squash_function: DootAssetSquashFunction,
    squash_stats: [DootAssetSquashAxisStats; DOOT_ASSET_AXIS_COUNT],
    model_hash: [u8; DOOT_ASSET_HASH_BYTES],
    tokenizer_hash: [u8; DOOT_ASSET_HASH_BYTES],
    pca_hash: [u8; DOOT_ASSET_HASH_BYTES],
}

/// Reports why a dootdoot asset could not be parsed.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct DootAssetError {
    message: String,
}

impl DootAsset {
    /// Parses a dootdoot asset spec protobuf payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is not valid protobuf, declares an
    /// unsupported asset spec version, or contains internally inconsistent
    /// runtime data.
    pub fn parse(bytes: &[u8]) -> Result<Self, DootAssetError> {
        let proto = DootAssetProto::decode(bytes).map_err(|error| {
            DootAssetError::new(format!("failed to decode dootdoot asset protobuf: {error}"))
        })?;

        parse_proto(proto)
    }

    /// Builds and validates a dootdoot asset from generation output.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied parts do not satisfy the dootdoot
    /// asset spec.
    pub fn from_parts(parts: DootAssetParts) -> Result<Self, DootAssetError> {
        parse_proto(parts.into_proto())
    }

    /// Encodes this asset as a dootdoot asset spec protobuf payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the asset cannot be represented by the protobuf
    /// schema.
    pub fn to_protobuf_bytes(&self) -> Result<Vec<u8>, DootAssetError> {
        Ok(self.to_proto()?.encode_to_vec())
    }

    /// Returns the committed file name for this asset.
    pub fn file_name(&self) -> &'static str {
        self.header.file_name
    }

    /// Returns the dootdoot asset spec version.
    pub fn spec_version(&self) -> u32 {
        self.header.spec_version
    }

    /// Returns the number of token records in the asset.
    pub fn token_count(&self) -> usize {
        self.header.token_count
    }

    /// Returns the per-axis dequantization scales for projected values.
    pub fn axis_scales(&self) -> &[f32; DOOT_ASSET_AXIS_COUNT] {
        &self.header.axis_scales
    }

    /// Returns the dequantization scale for token pooling weights.
    pub fn weight_scale(&self) -> f32 {
        self.header.weight_scale
    }

    /// Returns the squash function frozen into the asset.
    pub fn squash_function(&self) -> DootAssetSquashFunction {
        self.header.squash_function
    }

    /// Returns the per-axis squash statistics frozen into the asset.
    pub fn squash_stats(&self) -> &[DootAssetSquashAxisStats; DOOT_ASSET_AXIS_COUNT] {
        &self.header.squash_stats
    }

    /// Returns the SHA-256 hash of the source model.
    pub fn model_hash(&self) -> [u8; DOOT_ASSET_HASH_BYTES] {
        self.header.model_hash
    }

    /// Returns the SHA-256 hash of the source tokenizer.
    pub fn tokenizer_hash(&self) -> [u8; DOOT_ASSET_HASH_BYTES] {
        self.header.tokenizer_hash
    }

    /// Returns the SHA-256 hash of the PCA matrix used to project the model.
    pub fn pca_hash(&self) -> [u8; DOOT_ASSET_HASH_BYTES] {
        self.header.pca_hash
    }

    /// Returns the embedded tokenizer JSON bytes.
    pub fn tokenizer_json(&self) -> &[u8] {
        &self.tokenizer_json
    }

    /// Returns the compact per-token record bytes.
    pub fn record_bytes(&self) -> &[u8] {
        &self.record_bytes
    }
}

impl DootAssetScales {
    /// Builds a scale set from axis and weight scales.
    pub const fn new(axis_scales: [f32; DOOT_ASSET_AXIS_COUNT], weight_scale: f32) -> Self {
        Self {
            axis_scales,
            weight_scale,
        }
    }
}

impl DootAssetHashes {
    /// Builds a provenance hash set.
    pub const fn new(
        model_hash: [u8; DOOT_ASSET_HASH_BYTES],
        tokenizer_hash: [u8; DOOT_ASSET_HASH_BYTES],
        pca_hash: [u8; DOOT_ASSET_HASH_BYTES],
    ) -> Self {
        Self {
            model: model_hash,
            tokenizer: tokenizer_hash,
            pca: pca_hash,
        }
    }
}

impl DootAssetParts {
    /// Builds asset parts from generation output.
    pub fn new(
        token_count: u32,
        scales: DootAssetScales,
        squash_function: DootAssetSquashFunction,
        squash_stats: [DootAssetSquashAxisStats; DOOT_ASSET_AXIS_COUNT],
        hashes: DootAssetHashes,
        tokenizer_json: Vec<u8>,
        record_bytes: Vec<u8>,
    ) -> Self {
        Self {
            token_count,
            scales,
            squash_function,
            squash_stats,
            hashes,
            tokenizer_json,
            record_bytes,
        }
    }

    fn into_proto(self) -> DootAssetProto {
        DootAssetProto {
            spec_version: DOOT_ASSET_SPEC_VERSION,
            token_count: self.token_count,
            axis_count: u32::try_from(DOOT_ASSET_AXIS_COUNT)
                .expect("doot asset axis count should fit u32"),
            axis_scales: self.scales.axis_scales.to_vec(),
            weight_scale: self.scales.weight_scale,
            squash_function: squash_function_id(self.squash_function),
            squash_stats: self
                .squash_stats
                .iter()
                .map(|axis| DootAssetSquashAxisStatsProto {
                    mean: axis.mean(),
                    standard_deviation: axis.standard_deviation(),
                })
                .collect(),
            model_sha256: self.hashes.model.to_vec(),
            tokenizer_sha256: self.hashes.tokenizer.to_vec(),
            pca_sha256: self.hashes.pca.to_vec(),
            tokenizer_json: self.tokenizer_json,
            token_records: self.record_bytes,
        }
    }
}

impl DootAssetSquashAxisStats {
    /// Builds squash statistics for one PCA axis.
    pub const fn new(mean: f64, standard_deviation: f64) -> Self {
        Self {
            mean,
            standard_deviation,
        }
    }

    /// Returns the projected-axis mean used by the squash function.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Returns the projected-axis standard deviation used by the squash
    /// function.
    pub fn standard_deviation(&self) -> f64 {
        self.standard_deviation
    }
}

impl DootAsset {
    fn to_proto(&self) -> Result<DootAssetProto, DootAssetError> {
        Ok(DootAssetProto {
            spec_version: self.header.spec_version,
            token_count: usize_to_u32(self.header.token_count)?,
            axis_count: u32::try_from(DOOT_ASSET_AXIS_COUNT).map_err(|error| {
                DootAssetError::new(format!(
                    "dootdoot asset axis count does not fit u32: {error}",
                ))
            })?,
            axis_scales: self.header.axis_scales.to_vec(),
            weight_scale: self.header.weight_scale,
            squash_function: squash_function_id(self.header.squash_function),
            squash_stats: self
                .header
                .squash_stats
                .iter()
                .map(|axis| DootAssetSquashAxisStatsProto {
                    mean: axis.mean(),
                    standard_deviation: axis.standard_deviation(),
                })
                .collect(),
            model_sha256: self.header.model_hash.to_vec(),
            tokenizer_sha256: self.header.tokenizer_hash.to_vec(),
            pca_sha256: self.header.pca_hash.to_vec(),
            tokenizer_json: self.tokenizer_json.clone(),
            token_records: self.record_bytes.clone(),
        })
    }
}

impl DootAssetError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parses the embedded dootdoot asset spec payload.
///
/// # Errors
///
/// Returns an error if the committed `.doot` asset is malformed or
/// inconsistent with the asset spec constants compiled into the crate.
pub fn embedded_doot_asset() -> Result<DootAsset, DootAssetError> {
    DootAsset::parse(EMBEDDED_DOOT_ASSET)
}

fn parse_proto(proto: DootAssetProto) -> Result<DootAsset, DootAssetError> {
    if proto.spec_version != DOOT_ASSET_SPEC_VERSION {
        return Err(DootAssetError::new(format!(
            "dootdoot asset spec version mismatch: expected {DOOT_ASSET_SPEC_VERSION}, got {}",
            proto.spec_version,
        )));
    }

    let axis_count = u32_to_usize(proto.axis_count)?;

    if axis_count != DOOT_ASSET_AXIS_COUNT {
        return Err(DootAssetError::new(format!(
            "dootdoot asset axis count mismatch: expected {DOOT_ASSET_AXIS_COUNT}, got {axis_count}",
        )));
    }

    let token_count = u32_to_usize(proto.token_count)?;
    let axis_scales = read_f32_array(&proto.axis_scales, "axis scales")?;
    let weight_scale = proto.weight_scale;

    validate_scale("axis 0", axis_scales[0])?;
    validate_scale("axis 1", axis_scales[1])?;
    validate_scale("axis 2", axis_scales[2])?;
    validate_scale("axis 3", axis_scales[3])?;
    validate_scale("weight", weight_scale)?;

    let squash_function = parse_squash_function(proto.squash_function)?;
    let squash_stats = read_squash_stats(&proto.squash_stats)?;
    let expected_records_len = token_count
        .checked_mul(DOOT_ASSET_TOKEN_RECORD_BYTES)
        .ok_or_else(|| DootAssetError::new("dootdoot asset record length overflow"))?;

    if proto.token_records.len() != expected_records_len {
        return Err(DootAssetError::new(format!(
            "dootdoot asset record length mismatch: expected {expected_records_len} bytes, got {}",
            proto.token_records.len(),
        )));
    }

    if proto.tokenizer_json.is_empty() {
        return Err(DootAssetError::new(
            "dootdoot asset tokenizer JSON payload is empty",
        ));
    }

    Ok(DootAsset {
        header: DootAssetHeader {
            file_name: DOOT_ASSET_FILE_V1,
            spec_version: proto.spec_version,
            token_count,
            axis_scales,
            weight_scale,
            squash_function,
            squash_stats,
            model_hash: read_hash(&proto.model_sha256, "model")?,
            tokenizer_hash: read_hash(&proto.tokenizer_sha256, "tokenizer")?,
            pca_hash: read_hash(&proto.pca_sha256, "pca")?,
        },
        tokenizer_json: proto.tokenizer_json,
        record_bytes: proto.token_records,
    })
}

fn read_f32_array(
    values: &[f32],
    name: &str,
) -> Result<[f32; DOOT_ASSET_AXIS_COUNT], DootAssetError> {
    if values.len() != DOOT_ASSET_AXIS_COUNT {
        return Err(DootAssetError::new(format!(
            "dootdoot asset {name} length mismatch: expected {DOOT_ASSET_AXIS_COUNT}, got {}",
            values.len(),
        )));
    }

    let mut result = [0.0_f32; DOOT_ASSET_AXIS_COUNT];
    result.copy_from_slice(values);

    Ok(result)
}

fn read_squash_stats(
    values: &[DootAssetSquashAxisStatsProto],
) -> Result<[DootAssetSquashAxisStats; DOOT_ASSET_AXIS_COUNT], DootAssetError> {
    if values.len() != DOOT_ASSET_AXIS_COUNT {
        return Err(DootAssetError::new(format!(
            "dootdoot asset squash stats length mismatch: expected {DOOT_ASSET_AXIS_COUNT}, got {}",
            values.len(),
        )));
    }

    Ok([
        read_squash_axis_stats(&values[0])?,
        read_squash_axis_stats(&values[1])?,
        read_squash_axis_stats(&values[2])?,
        read_squash_axis_stats(&values[3])?,
    ])
}

fn read_squash_axis_stats(
    proto: &DootAssetSquashAxisStatsProto,
) -> Result<DootAssetSquashAxisStats, DootAssetError> {
    if !proto.mean.is_finite() {
        return Err(DootAssetError::new(format!(
            "dootdoot asset squash mean is not finite: {}",
            proto.mean,
        )));
    }

    if !proto.standard_deviation.is_finite() || proto.standard_deviation <= 0.0 {
        return Err(DootAssetError::new(format!(
            "dootdoot asset squash standard deviation must be positive and finite: {}",
            proto.standard_deviation,
        )));
    }

    Ok(DootAssetSquashAxisStats {
        mean: proto.mean,
        standard_deviation: proto.standard_deviation,
    })
}

fn parse_squash_function(value: i32) -> Result<DootAssetSquashFunction, DootAssetError> {
    match DootAssetSquashFunctionId::try_from(value) {
        Ok(DootAssetSquashFunctionId::TanhZScore) => Ok(DootAssetSquashFunction::TanhZScore),
        Ok(DootAssetSquashFunctionId::Unspecified) | Err(_) => Err(DootAssetError::new(format!(
            "unknown dootdoot asset squash function id: {value}",
        ))),
    }
}

fn squash_function_id(function: DootAssetSquashFunction) -> i32 {
    match function {
        DootAssetSquashFunction::TanhZScore => DootAssetSquashFunctionId::TanhZScore as i32,
    }
}

fn read_hash(bytes: &[u8], name: &str) -> Result<[u8; DOOT_ASSET_HASH_BYTES], DootAssetError> {
    if bytes.len() != DOOT_ASSET_HASH_BYTES {
        return Err(DootAssetError::new(format!(
            "dootdoot asset {name} hash length mismatch: expected {DOOT_ASSET_HASH_BYTES}, got {}",
            bytes.len(),
        )));
    }

    let mut result = [0_u8; DOOT_ASSET_HASH_BYTES];
    result.copy_from_slice(bytes);

    Ok(result)
}

fn validate_scale(name: &str, scale: f32) -> Result<(), DootAssetError> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(DootAssetError::new(format!(
            "dootdoot asset {name} scale must be positive and finite: {scale}",
        )));
    }

    Ok(())
}

fn u32_to_usize(value: u32) -> Result<usize, DootAssetError> {
    usize::try_from(value).map_err(|error| {
        DootAssetError::new(format!(
            "dootdoot asset integer does not fit usize: {error}",
        ))
    })
}

fn usize_to_u32(value: usize) -> Result<u32, DootAssetError> {
    u32::try_from(value).map_err(|error| {
        DootAssetError::new(format!("dootdoot asset integer does not fit u32: {error}",))
    })
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DootAssetProto {
    #[prost(uint32, tag = "1")]
    pub(crate) spec_version: u32,
    #[prost(uint32, tag = "2")]
    pub(crate) token_count: u32,
    #[prost(uint32, tag = "3")]
    pub(crate) axis_count: u32,
    #[prost(float, repeated, tag = "4")]
    pub(crate) axis_scales: Vec<f32>,
    #[prost(float, tag = "5")]
    pub(crate) weight_scale: f32,
    #[prost(enumeration = "DootAssetSquashFunctionId", tag = "6")]
    pub(crate) squash_function: i32,
    #[prost(message, repeated, tag = "7")]
    pub(crate) squash_stats: Vec<DootAssetSquashAxisStatsProto>,
    #[prost(bytes = "vec", tag = "8")]
    pub(crate) model_sha256: Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    pub(crate) tokenizer_sha256: Vec<u8>,
    #[prost(bytes = "vec", tag = "10")]
    pub(crate) pca_sha256: Vec<u8>,
    #[prost(bytes = "vec", tag = "11")]
    pub(crate) tokenizer_json: Vec<u8>,
    #[prost(bytes = "vec", tag = "12")]
    pub(crate) token_records: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(crate) struct DootAssetSquashAxisStatsProto {
    #[prost(double, tag = "1")]
    pub(crate) mean: f64,
    #[prost(double, tag = "2")]
    pub(crate) standard_deviation: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub(crate) enum DootAssetSquashFunctionId {
    Unspecified = 0,
    TanhZScore = 1,
}
