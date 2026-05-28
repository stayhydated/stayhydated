use std::path::{Path, PathBuf};

use anyhow::Context as _;
use cargo_metadata::MetadataCommand;

pub mod book;
pub mod llms;
pub mod release;
pub mod web;

pub fn workspace_root_from_xtask_manifest() -> anyhow::Result<PathBuf> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to read cargo metadata from the current workspace")?;

    Ok(metadata.workspace_root.into_std_path_buf())
}

pub fn workspace_root_from_xtask_manifest_dir(manifest_dir: &Path) -> anyhow::Result<PathBuf> {
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("failed to resolve workspace root from xtask manifest directory")
}
