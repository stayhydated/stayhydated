use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, bail};
use bon::Builder;
use walkdir::WalkDir;

pub struct CopyPath {
    pub source: PathBuf,
    pub destination: PathBuf,
}

#[derive(Builder)]
#[builder(
    builder_type = WebBuildConfigBonBuilder,
    on(PathBuf, into),
    on(String, into)
)]
pub struct WebBuildConfig {
    pub command_current_dir: PathBuf,
    pub dioxus_args: Vec<String>,
    pub dx_public_dir: PathBuf,
    pub dist_dir: PathBuf,
    pub copy_dirs: Vec<CopyPath>,
    pub copy_files: Vec<CopyPath>,
    pub write_404_from_index: bool,
    pub sitemap_xml: Option<String>,
}

pub struct WebBuildConfigBuilder {
    workspace_root: PathBuf,
    command_current_dir: Option<PathBuf>,
    package: Option<String>,
    extra_copy_dirs: Vec<CopyPath>,
    extra_copy_files: Vec<CopyPath>,
    public_assets_dir: Option<PathBuf>,
    sitemap_xml: Option<String>,
}

impl WebBuildConfig {
    pub fn github_pages(workspace_root: impl Into<PathBuf>) -> WebBuildConfigBuilder {
        WebBuildConfigBuilder {
            workspace_root: workspace_root.into(),
            command_current_dir: None,
            package: None,
            extra_copy_dirs: Vec::new(),
            extra_copy_files: Vec::new(),
            public_assets_dir: Some(PathBuf::from("web/public/assets")),
            sitemap_xml: None,
        }
    }
}

impl WebBuildConfigBuilder {
    pub fn command_current_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.command_current_dir = Some(path.into());
        self
    }

    pub fn package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    pub fn no_public_assets_dir(mut self) -> Self {
        self.public_assets_dir = None;
        self
    }

    pub fn public_assets_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.public_assets_dir = Some(path.into());
        self
    }

    pub fn extra_dir(
        mut self,
        source: impl Into<PathBuf>,
        destination: impl Into<PathBuf>,
    ) -> Self {
        self.extra_copy_dirs.push(CopyPath {
            source: self.workspace_root.join(source.into()),
            destination: self.dist_dir().join(destination.into()),
        });
        self
    }

    pub fn extra_file(
        mut self,
        source: impl Into<PathBuf>,
        destination: impl Into<PathBuf>,
    ) -> Self {
        self.extra_copy_files.push(CopyPath {
            source: self.workspace_root.join(source.into()),
            destination: self.dist_dir().join(destination.into()),
        });
        self
    }

    pub fn sitemap_xml(mut self, sitemap_xml: impl Into<String>) -> Self {
        self.sitemap_xml = Some(sitemap_xml.into());
        self
    }

    pub fn build(self) -> WebBuildConfig {
        let dist_dir = self.dist_dir();
        let mut dioxus_args = vec!["build".to_owned()];
        if let Some(package) = self.package {
            dioxus_args.push("--package".to_owned());
            dioxus_args.push(package);
        }
        dioxus_args.extend(
            [
                "--platform",
                "web",
                "--ssg",
                "--release",
                "--debug-symbols",
                "false",
                "--force-sequential",
                "true",
            ]
            .into_iter()
            .map(str::to_owned),
        );

        let mut copy_dirs = Vec::new();
        if let Some(public_assets_dir) = self.public_assets_dir {
            copy_dirs.push(CopyPath {
                source: self.workspace_root.join(public_assets_dir),
                destination: dist_dir.join("assets"),
            });
        }
        copy_dirs.extend([
            CopyPath {
                source: self.workspace_root.join("web/public/book"),
                destination: dist_dir.join("book"),
            },
            CopyPath {
                source: self.workspace_root.join("web/public/llms"),
                destination: dist_dir.join("llms"),
            },
        ]);
        copy_dirs.extend(self.extra_copy_dirs);

        let mut copy_files = vec![
            CopyPath {
                source: self.workspace_root.join("web/public/.nojekyll"),
                destination: dist_dir.join(".nojekyll"),
            },
            CopyPath {
                source: self.workspace_root.join("web/public/llms.txt"),
                destination: dist_dir.join("llms.txt"),
            },
            CopyPath {
                source: self.workspace_root.join("web/public/llms-full.txt"),
                destination: dist_dir.join("llms-full.txt"),
            },
        ];
        copy_files.extend(self.extra_copy_files);

        WebBuildConfig::builder()
            .command_current_dir(
                self.command_current_dir
                    .unwrap_or_else(|| self.workspace_root.clone()),
            )
            .dioxus_args(dioxus_args)
            .dx_public_dir(self.workspace_root.join("target/dx/web/release/web/public"))
            .dist_dir(dist_dir)
            .copy_dirs(copy_dirs)
            .copy_files(copy_files)
            .write_404_from_index(true)
            .maybe_sitemap_xml(self.sitemap_xml)
            .build()
    }

    fn dist_dir(&self) -> PathBuf {
        self.workspace_root.join("web/dist")
    }
}

pub fn build(config: WebBuildConfig) -> anyhow::Result<()> {
    if config.dx_public_dir.exists() {
        fs::remove_dir_all(&config.dx_public_dir).with_context(|| {
            format!(
                "failed to clear generated Dioxus output at {}",
                config.dx_public_dir.display()
            )
        })?;
    }

    let status = Command::new("dx")
        .current_dir(&config.command_current_dir)
        .args(&config.dioxus_args)
        .status()
        .with_context(|| {
            format!(
                "failed to run `dx {}` for the docs site",
                config.dioxus_args.join(" ")
            )
        })?;

    if !status.success() {
        bail!(
            "`dx {}` failed with status {status}",
            config.dioxus_args.join(" ")
        );
    }

    if !config.dx_public_dir.is_dir() {
        bail!(
            "expected Dioxus static output at {}",
            config.dx_public_dir.display()
        );
    }

    if config.dist_dir.exists() {
        fs::remove_dir_all(&config.dist_dir)
            .with_context(|| format!("failed to remove {}", config.dist_dir.display()))?;
    }
    fs::create_dir_all(&config.dist_dir)
        .with_context(|| format!("failed to create {}", config.dist_dir.display()))?;

    copy_directory(&config.dx_public_dir, &config.dist_dir)?;

    for copy in &config.copy_dirs {
        copy_directory(&copy.source, &copy.destination)?;
    }
    for copy in &config.copy_files {
        copy_file_if_present(&copy.source, &copy.destination)?;
    }

    if config.write_404_from_index {
        fs::copy(
            config.dist_dir.join("index.html"),
            config.dist_dir.join("404.html"),
        )
        .with_context(|| {
            format!(
                "failed to write {}",
                config.dist_dir.join("404.html").display()
            )
        })?;
    }

    if let Some(sitemap_xml) = config.sitemap_xml {
        fs::write(config.dist_dir.join("sitemap.xml"), sitemap_xml).with_context(|| {
            format!(
                "failed to write {}",
                config.dist_dir.join("sitemap.xml").display()
            )
        })?;
    }

    Ok(())
}

fn copy_file_if_present(
    source: &std::path::Path,
    destination: &std::path::Path,
) -> anyhow::Result<()> {
    if !source.is_file() {
        return Ok(());
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            destination.display()
        )
    })?;

    Ok(())
}

fn copy_directory(source: &std::path::Path, destination: &std::path::Path) -> anyhow::Result<()> {
    if !source.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(source) {
        let entry = entry.with_context(|| format!("failed to walk {}", source.display()))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .with_context(|| format!("failed to strip prefix {}", source.display()))?;

        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create {}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }

    Ok(())
}
