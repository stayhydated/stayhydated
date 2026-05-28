pub fn base_href(base_path: Option<&str>) -> String {
    match base_path {
        Some(base_path) => {
            let base_path = base_path.trim_matches('/');
            if base_path.is_empty() {
                "/".to_string()
            } else {
                format!("/{base_path}/")
            }
        },
        None => "/".to_string(),
    }
}

pub fn href(base_href: &str, route: &str) -> String {
    let base_href = trailing_slash(base_href);
    let route = route.trim_matches('/');

    if route.is_empty() {
        base_href
    } else {
        format!("{base_href}{route}/")
    }
}

pub fn site_root_prefix(output_dir: &str) -> String {
    if output_dir.is_empty() {
        return "./".to_string();
    }

    "../".repeat(
        output_dir
            .split('/')
            .filter(|segment| !segment.is_empty())
            .count(),
    )
}

pub fn normalized_path_segments<'a>(path: &'a str, base_path: Option<&str>) -> Vec<&'a str> {
    let segments = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let base_path_segments = base_path
        .into_iter()
        .flat_map(|base_path| base_path.trim_matches('/').split('/'))
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if base_path_segments.is_empty()
        || !segments
            .as_slice()
            .starts_with(base_path_segments.as_slice())
    {
        segments
    } else {
        segments[base_path_segments.len()..].to_vec()
    }
}

fn trailing_slash(value: &str) -> String {
    if value.ends_with('/') {
        value.to_owned()
    } else {
        format!("{value}/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_href_normalizes_optional_base_paths() {
        assert_eq!(base_href(None), "/");
        assert_eq!(base_href(Some("")), "/");
        assert_eq!(base_href(Some("/project/")), "/project/");
    }

    #[test]
    fn href_joins_routes_under_base_href() {
        assert_eq!(href("/project/", ""), "/project/");
        assert_eq!(href("/project", "demos"), "/project/demos/");
        assert_eq!(href("/", "/book/"), "/book/");
    }

    #[test]
    fn root_prefix_tracks_output_depth() {
        assert_eq!(site_root_prefix(""), "./");
        assert_eq!(site_root_prefix("demos"), "../");
        assert_eq!(site_root_prefix("zh/demos"), "../../");
    }

    #[test]
    fn normalized_segments_strip_matching_base_path() {
        assert_eq!(
            normalized_path_segments("/repo/zh/demos/", Some("repo")),
            ["zh", "demos"]
        );
        assert_eq!(
            normalized_path_segments("/other/zh/demos/", Some("repo")),
            ["other", "zh", "demos"]
        );
    }
}
