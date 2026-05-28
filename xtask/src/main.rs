use anyhow::Context as _;
use clap::{Parser, Subcommand};
use stayhydated_xtask::web::WebBuildConfig;

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Build { target } => match target {
            BuildTarget::Web => build_web(),
        },
    }
}

#[derive(Debug, Parser)]
#[command(bin_name = "cargo xtask", about = "Repository automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build generated artifacts.
    Build {
        #[command(subcommand)]
        target: BuildTarget,
    },
}

#[derive(Debug, Subcommand)]
enum BuildTarget {
    /// Build the GitHub Pages web output.
    Web,
}

fn build_web() -> anyhow::Result<()> {
    let workspace_root = stayhydated_xtask::workspace_root_from_xtask_manifest()
        .context("failed to resolve workspace root")?;

    stayhydated_xtask::web::build(
        WebBuildConfig::github_pages(&workspace_root)
            .command_current_dir(workspace_root.join("web"))
            .sitemap_xml(web::sitemap_xml())
            .build(),
    )
}
