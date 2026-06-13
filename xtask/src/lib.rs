//! Build-time support library for dootdoot asset generation.

mod format_artifact;
mod pca_projection;
mod run;
mod source_files;
mod source_manifest;
mod source_manifest_error;
mod source_model;
mod squash_stats;

pub use format_artifact::{dequantize_i16, quantize_symmetric_i16, serialize_format_artifact};
pub use pca_projection::{PcaProjection, compute_pca_projection};
pub use run::run;
pub use source_files::SourceFiles;
pub use source_manifest::SourceManifest;
pub use source_manifest_error::{Result, SourceManifestError};
pub use source_model::{SourceModel, load_source_model};
pub use squash_stats::{AxisSquashStats, SquashFunction, SquashStats, compute_squash_stats};
