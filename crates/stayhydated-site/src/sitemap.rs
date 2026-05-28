use std::fmt::Write as _;

pub fn render<I, P>(site_url: &str, paths: I) -> String
where
    I: IntoIterator<Item = P>,
    P: AsRef<str>,
{
    let mut entries = String::new();

    for path in paths {
        let url = absolute_url(site_url, path.as_ref());
        let _ = writeln!(entries, "  <url><loc>{url}</loc></url>");
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{entries}</urlset>\n"
    )
}

fn absolute_url(site_url: &str, path: &str) -> String {
    let base_url = if site_url.ends_with('/') {
        site_url.to_owned()
    } else {
        format!("{site_url}/")
    };
    let path = path.trim_start_matches('/');

    if path.is_empty() {
        base_url
    } else {
        format!("{base_url}{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_root_and_nested_paths() {
        let sitemap = render("https://example.test/project", ["/", "/demos/", "llms.txt"]);

        assert!(sitemap.contains("<loc>https://example.test/project/</loc>"));
        assert!(sitemap.contains("<loc>https://example.test/project/demos/</loc>"));
        assert!(sitemap.contains("<loc>https://example.test/project/llms.txt</loc>"));
    }
}
