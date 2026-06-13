//! Build-time support library for dootdoot asset generation.

mod run;
mod source_files;
mod source_manifest;
mod source_manifest_error;

pub use run::run;
pub use source_files::SourceFiles;
pub use source_manifest::SourceManifest;
pub use source_manifest_error::{Result, SourceManifestError};
