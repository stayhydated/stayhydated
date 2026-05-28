use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
    path::Path,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, bail};
use bon::Builder;
use cargo_metadata::{DependencyKind, MetadataCommand, Package, PackageId};

#[derive(Clone, Debug)]
struct ReleasePackage {
    id: PackageId,
    name: String,
    version: String,
}

#[derive(Builder, Clone, Debug)]
#[builder(on(String, into))]
pub struct PublishOptions {
    pub execute: bool,
    pub from: Option<String>,
    pub registry: Option<String>,
    #[builder(default)]
    pub allow_dirty: bool,
    #[builder(default)]
    pub no_verify: bool,
    #[builder(default)]
    pub include_dev_deps: bool,
    #[builder(default)]
    pub skip_existing: bool,
    #[builder(default = 3)]
    pub retries: u32,
    #[builder(default = 20)]
    pub retry_delay_seconds: u64,
}

impl PublishOptions {
    pub fn new(execute: bool) -> Self {
        Self::builder().execute(execute).build()
    }

    pub fn resume_from(mut self, from: Option<String>) -> Self {
        self.from = from;
        self
    }

    pub fn registry(mut self, registry: Option<String>) -> Self {
        self.registry = registry;
        self
    }

    pub fn allow_dirty(mut self, allow_dirty: bool) -> Self {
        self.allow_dirty = allow_dirty;
        self
    }

    pub fn no_verify(mut self, no_verify: bool) -> Self {
        self.no_verify = no_verify;
        self
    }

    pub fn include_dev_deps(mut self, include_dev_deps: bool) -> Self {
        self.include_dev_deps = include_dev_deps;
        self
    }

    pub fn skip_existing(mut self, skip_existing: bool) -> Self {
        self.skip_existing = skip_existing;
        self
    }

    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    pub fn retry_delay_seconds(mut self, retry_delay_seconds: u64) -> Self {
        self.retry_delay_seconds = retry_delay_seconds;
        self
    }
}

pub fn plan(workspace_root: &Path) -> anyhow::Result<()> {
    let packages = release_order(workspace_root)?;
    print_order(&packages);
    Ok(())
}

pub fn publish(workspace_root: &Path, options: &PublishOptions) -> anyhow::Result<()> {
    let packages = release_order(workspace_root)?;
    let packages = packages_from(&packages, options.from.as_deref())?;

    print_order(packages);

    if !options.execute {
        println!();
        println!("No packages were uploaded. Add --execute to run:");
        for package in packages {
            println!("  {}", cargo_publish_command(package, options).join(" "));
        }
        return Ok(());
    }

    if !options.include_dev_deps {
        ensure_cargo_hack()?;
    }

    for package in packages {
        publish_package(workspace_root, package, options)?;
    }

    Ok(())
}

fn release_order(workspace_root: &Path) -> anyhow::Result<Vec<ReleasePackage>> {
    let metadata = MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .exec()
        .context("failed to read cargo metadata")?;

    let package_by_id = metadata
        .packages
        .iter()
        .map(|package| (package.id.clone(), package))
        .collect::<HashMap<_, _>>();

    let publishable = metadata
        .workspace_members
        .iter()
        .filter_map(|id| package_by_id.get(id).copied())
        .filter(|package| is_publishable(package))
        .collect::<Vec<_>>();

    let publishable_ids = publishable
        .iter()
        .map(|package| package.id.clone())
        .collect::<HashSet<_>>();
    let package_name_to_id = publishable
        .iter()
        .map(|package| (package.name.to_string(), package.id.clone()))
        .collect::<HashMap<_, _>>();
    let workspace_index = publishable
        .iter()
        .enumerate()
        .map(|(index, package)| (package.id.clone(), index))
        .collect::<HashMap<_, _>>();

    let mut remaining_deps = publishable
        .iter()
        .map(|package| {
            let deps = package
                .dependencies
                .iter()
                .filter(|dependency| !matches!(dependency.kind, DependencyKind::Development))
                .filter_map(|dependency| package_name_to_id.get(&dependency.name.to_string()))
                .filter(|dependency_id| publishable_ids.contains(*dependency_id))
                .cloned()
                .collect::<HashSet<_>>();
            (package.id.clone(), deps)
        })
        .collect::<HashMap<_, _>>();

    let mut dependents = HashMap::<PackageId, Vec<PackageId>>::new();
    for (package_id, deps) in &remaining_deps {
        for dep_id in deps {
            dependents
                .entry(dep_id.clone())
                .or_default()
                .push(package_id.clone());
        }
    }

    let mut ready = remaining_deps
        .iter()
        .filter_map(|(package_id, deps)| deps.is_empty().then_some(package_id.clone()))
        .collect::<Vec<_>>();
    sort_by_workspace_index(&mut ready, &workspace_index);

    let mut ordered = Vec::new();
    while let Some(package_id) = ready.first().cloned() {
        ready.remove(0);

        let package = package_by_id
            .get(&package_id)
            .with_context(|| format!("metadata missing package {package_id}"))?;
        ordered.push(ReleasePackage {
            id: package.id.clone(),
            name: package.name.to_string(),
            version: package.version.to_string(),
        });

        for dependent_id in dependents.get(&package_id).into_iter().flatten() {
            let deps = remaining_deps
                .get_mut(dependent_id)
                .with_context(|| format!("metadata missing dependent package {dependent_id}"))?;
            deps.remove(&package_id);
            if deps.is_empty() && !ordered.iter().any(|package| package.id == *dependent_id) {
                ready.push(dependent_id.clone());
            }
        }
        sort_by_workspace_index(&mut ready, &workspace_index);
    }

    if ordered.len() != publishable.len() {
        let ordered_ids = ordered
            .iter()
            .map(|package| package.id.clone())
            .collect::<HashSet<_>>();
        let blocked = publishable
            .iter()
            .filter(|package| !ordered_ids.contains(&package.id))
            .map(|package| {
                let deps = remaining_deps
                    .get(&package.id)
                    .into_iter()
                    .flat_map(|deps| deps.iter())
                    .filter_map(|dep_id| package_by_id.get(dep_id))
                    .map(|dep| dep.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} waits on {}", package.name, deps)
            })
            .collect::<Vec<_>>();
        bail!(
            "workspace publish dependencies contain a cycle: {}",
            blocked.join("; ")
        );
    }

    Ok(ordered)
}

fn packages_from<'a>(
    packages: &'a [ReleasePackage],
    from: Option<&str>,
) -> anyhow::Result<&'a [ReleasePackage]> {
    let Some(from) = from else {
        return Ok(packages);
    };

    let index = packages
        .iter()
        .position(|package| package.name == from)
        .with_context(|| {
            let names = packages
                .iter()
                .map(|package| package.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("unknown release package `{from}`; expected one of: {names}")
        })?;

    Ok(&packages[index..])
}

fn publish_package(
    workspace_root: &Path,
    package: &ReleasePackage,
    options: &PublishOptions,
) -> anyhow::Result<()> {
    let command = cargo_publish_command(package, options);
    for attempt in 0..=options.retries {
        if requires_clean_worktree_guard(options) {
            ensure_tracked_worktree_clean(workspace_root)?;
        }

        println!();
        println!("Running {}", command.join(" "));

        let output = Command::new(&command[0])
            .current_dir(workspace_root)
            .args(&command[1..])
            .output()
            .with_context(|| format!("failed to run {}", command.join(" ")))?;

        print_output(&output)?;

        if output.status.success() {
            return Ok(());
        }

        if options.skip_existing && output_mentions_existing_upload(&output) {
            println!(
                "{} {} is already uploaded; continuing because --skip-existing was set",
                package.name, package.version
            );
            return Ok(());
        }

        if attempt == options.retries {
            bail!(
                "{} failed after {} attempt(s) with status {}",
                command.join(" "),
                attempt + 1,
                output.status
            );
        }

        println!(
            "Publish failed; retrying in {}s for crates.io index propagation",
            options.retry_delay_seconds
        );
        thread::sleep(Duration::from_secs(options.retry_delay_seconds));
    }

    Ok(())
}

fn ensure_cargo_hack() -> anyhow::Result<()> {
    let output = Command::new("cargo")
        .args(["hack", "--version"])
        .output()
        .context("failed to run `cargo hack --version`")?;

    if output.status.success() {
        return Ok(());
    }

    print_output(&output)?;
    bail!(
        "release publish requires cargo-hack; install it with `cargo install cargo-hack` or pass --include-dev-deps"
    );
}

fn ensure_tracked_worktree_clean(workspace_root: &Path) -> anyhow::Result<()> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .context("failed to inspect git working tree")?;

    if !output.status.success() {
        print_output(&output)?;
        bail!("failed to inspect git working tree before publishing");
    }

    if !output.stdout.is_empty() {
        let changes = String::from_utf8_lossy(&output.stdout);
        bail!(
            "release publish uses cargo-hack manifest rewrites and passes --allow-dirty through to cargo publish; commit or stash tracked changes first, or pass xtask's --allow-dirty to publish them anyway:\n{}",
            changes.trim_end()
        );
    }

    Ok(())
}

fn cargo_publish_command(package: &ReleasePackage, options: &PublishOptions) -> Vec<String> {
    let mut command = if options.include_dev_deps {
        vec![
            "cargo".to_owned(),
            "publish".to_owned(),
            "-p".to_owned(),
            package.name.clone(),
        ]
    } else {
        vec![
            "cargo".to_owned(),
            "hack".to_owned(),
            "--no-dev-deps".to_owned(),
            "publish".to_owned(),
            "-p".to_owned(),
            package.name.clone(),
        ]
    };

    if let Some(registry) = &options.registry {
        command.push("--registry".to_owned());
        command.push(registry.clone());
    }
    if cargo_publish_needs_allow_dirty(options) {
        command.push("--allow-dirty".to_owned());
    }
    if options.no_verify {
        command.push("--no-verify".to_owned());
    }

    command
}

fn cargo_publish_needs_allow_dirty(options: &PublishOptions) -> bool {
    options.allow_dirty || !options.include_dev_deps
}

fn requires_clean_worktree_guard(options: &PublishOptions) -> bool {
    !options.allow_dirty && !options.include_dev_deps
}

fn print_order(packages: &[ReleasePackage]) {
    println!("Release publish order:");
    for (index, package) in packages.iter().enumerate() {
        println!("{:>2}. {} {}", index + 1, package.name, package.version);
    }
    println!("Order is computed from non-dev workspace dependencies.");
}

fn print_output(output: &Output) -> anyhow::Result<()> {
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;
    Ok(())
}

fn output_mentions_existing_upload(output: &Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    stderr.contains("already uploaded") || stderr.contains("already exists")
}

fn sort_by_workspace_index(
    package_ids: &mut [PackageId],
    workspace_index: &HashMap<PackageId, usize>,
) {
    package_ids.sort_by_key(|package_id| workspace_index.get(package_id).copied());
}

fn is_publishable(package: &Package) -> bool {
    package
        .publish
        .as_ref()
        .is_none_or(|registries| !registries.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str) -> ReleasePackage {
        ReleasePackage {
            id: PackageId {
                repr: format!("path+file:///workspace/{name}#0.1.0"),
            },
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
        }
    }

    fn options() -> PublishOptions {
        PublishOptions::builder().execute(false).build()
    }

    #[test]
    fn cargo_hack_publish_allows_its_temporary_manifest_edits() {
        let command = cargo_publish_command(&package("public-crate"), &options());

        assert_eq!(
            command,
            [
                "cargo",
                "hack",
                "--no-dev-deps",
                "publish",
                "-p",
                "public-crate",
                "--allow-dirty",
            ]
        );
    }

    #[test]
    fn cargo_hack_publish_guards_preexisting_dirty_changes_by_default() {
        assert!(requires_clean_worktree_guard(&options()));
    }

    #[test]
    fn explicit_dirty_publish_disables_the_clean_worktree_guard() {
        let mut options = options();
        options.allow_dirty = true;

        assert!(!requires_clean_worktree_guard(&options));
    }

    #[test]
    fn plain_cargo_publish_does_not_allow_dirty_by_default() {
        let mut options = options();
        options.include_dev_deps = true;

        let command = cargo_publish_command(&package("public-crate"), &options);

        assert_eq!(command, ["cargo", "publish", "-p", "public-crate"]);
    }
}
