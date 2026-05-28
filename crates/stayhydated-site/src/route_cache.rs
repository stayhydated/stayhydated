use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn mark_generated_route_cache(
    public_dir: &Path,
    marker_file_name: &str,
    marker_message: &str,
) -> std::io::Result<()> {
    fs::create_dir_all(public_dir)?;
    fs::write(public_dir.join(marker_file_name), marker_message)
}

pub fn cleanup_generated_route_cache<I, S, F>(
    public_dir: &Path,
    marker_file_name: &str,
    generated_top_level_dirs: I,
    should_remove_additional_dir: F,
) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    F: Fn(&Path, &str) -> bool,
{
    if !public_dir.exists() {
        return Ok(());
    }

    if !public_dir.join(marker_file_name).is_file() {
        return Ok(());
    }

    remove_file_if_exists(&public_dir.join("index.html"))?;
    remove_file_if_exists(&public_dir.join("404.html"))?;

    let generated_top_level_dirs = generated_top_level_dirs
        .into_iter()
        .filter_map(|dir| {
            let dir = dir.as_ref();
            (!dir.is_empty()).then(|| dir.to_owned())
        })
        .collect::<HashSet<_>>();

    for dir in &generated_top_level_dirs {
        remove_dir_if_exists(&public_dir.join(dir))?;
    }

    for entry in fs::read_dir(public_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };

        if should_remove_additional_dir(&entry.path(), name) {
            fs::remove_dir_all(entry.path())?;
        }
    }

    Ok(())
}

fn remove_file_if_exists(path: &Path) -> std::io::Result<()> {
    if path.is_file() {
        fs::remove_file(path)?;
    }

    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_removes_generated_routes_and_preserves_static_assets() {
        let temp = tempfile::tempdir().expect("tempdir");
        let public_dir = temp.path();

        fs::write(public_dir.join("index.html"), "root").expect("write root index");
        fs::write(public_dir.join("404.html"), "not found").expect("write root 404");
        fs::create_dir_all(public_dir.join("demos")).expect("create demos dir");
        fs::write(public_dir.join("demos").join("index.html"), "stale demos")
            .expect("write demos index");
        fs::create_dir_all(public_dir.join("book")).expect("create book dir");
        fs::write(public_dir.join("book").join("index.html"), "book").expect("write book");
        fs::create_dir_all(public_dir.join("assets")).expect("create assets dir");
        fs::write(public_dir.join("assets").join("site.css"), "body {}").expect("write asset");

        mark_generated_route_cache(public_dir, ".generated-route-cache", "generated\n")
            .expect("mark route cache");
        cleanup_generated_route_cache(public_dir, ".generated-route-cache", ["demos"], |_, _| {
            false
        })
        .expect("cleanup route cache");

        assert!(!public_dir.join("index.html").exists());
        assert!(!public_dir.join("404.html").exists());
        assert!(!public_dir.join("demos").exists());
        assert!(public_dir.join("book").join("index.html").exists());
        assert!(public_dir.join("assets").join("site.css").exists());
    }

    #[test]
    fn cleanup_can_remove_additional_generated_dirs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let public_dir = temp.path();
        fs::create_dir_all(public_dir.join("zh").join("demos")).expect("create zh demos dir");
        fs::write(
            public_dir.join("zh").join("demos").join("index.html"),
            "stale",
        )
        .expect("write generated route");

        mark_generated_route_cache(public_dir, ".generated-route-cache", "generated\n")
            .expect("mark route cache");
        cleanup_generated_route_cache(
            public_dir,
            ".generated-route-cache",
            [] as [&str; 0],
            |path, name| name == "zh" && path.join("demos").is_dir(),
        )
        .expect("cleanup route cache");

        assert!(!public_dir.join("zh").exists());
    }
}
