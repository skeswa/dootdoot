//! Build-time support library for dootdoot asset generation.

mod bb8_metrics;
mod doot_asset;
mod pca_projection;
mod pos_class_table;
mod pos_source_manifest;
mod run;
mod source_files;
mod source_manifest;
mod source_manifest_error;
mod source_model;
mod squash_stats;

pub use doot_asset::{dequantize_i16, quantize_symmetric_i16, serialize_doot_asset};
pub use pca_projection::{PcaProjection, compute_pca_projection};
pub use pos_class_table::{
    PosClassEntry, PosPolicyConfig, PosSnapshot, PosTableClass, derive_pos_class_table,
    parse_tagged_counts,
};
pub use pos_source_manifest::PosSourceManifest;
pub use run::{run, run_with_args};
pub use source_files::SourceFiles;
pub use source_manifest::SourceManifest;
pub use source_manifest_error::{Result, SourceManifestError};
pub use source_model::{SourceModel, load_source_model};
pub use squash_stats::{AxisSquashStats, SquashFunction, SquashStats, compute_squash_stats};
