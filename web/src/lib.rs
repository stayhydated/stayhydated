mod components;
mod pages;
mod site;

use std::path::Path;

pub use site::app::App;

pub fn sitemap_xml() -> String {
    site::render::render_sitemap()
}

pub fn cleanup_generated_route_cache(public_dir: impl AsRef<Path>) -> std::io::Result<()> {
    site::routing::cleanup_generated_route_cache(public_dir.as_ref())
}

pub fn mark_generated_route_cache(public_dir: impl AsRef<Path>) -> std::io::Result<()> {
    site::routing::mark_generated_route_cache(public_dir.as_ref())
}

#[cfg(test)]
mod tests {
    use crate::site::i18n::SiteLanguage;
    use crate::site::routing::{PageKind, SiteRoute};
    use serial_test::serial;
    use std::fs;

    #[test]
    #[serial]
    fn renders_english_project_index() {
        let html = crate::site::render::render_route_body(SiteRoute::new(
            SiteLanguage::EnUs,
            PageKind::Home,
        ))
        .expect("page should render");

        assert!(html.contains("stayhydated"));
        assert!(html.contains("href=\"/koruma/\""));
        assert!(html.contains("href=\"/es-fluent/\""));
        assert!(html.contains("validation"));
        assert!(html.contains("localization"));
        assert!(!html.contains("Type-safe Rust projects from stayhydated"));
    }

    #[test]
    #[serial]
    fn renders_simplified_chinese_project_index() {
        let html = crate::site::render::render_route_body(SiteRoute::new(
            SiteLanguage::ZhCn,
            PageKind::Home,
        ))
        .expect("page should render");

        assert!(html.contains("校验"));
        assert!(html.contains("本地化"));
        assert!(html.contains("project-switcher"));
        assert!(html.contains("项目索引"));
        assert!(html.contains("href=\"/koruma/\""));
        assert!(!html.contains("stayhydated 的类型安全 Rust 项目索引"));
    }

    #[test]
    fn parses_localized_routes() {
        assert_eq!(
            crate::site::routing::site_route_from_path_with_base_path(
                "/your_repo/zh/",
                Some("your_repo")
            ),
            SiteRoute::new(SiteLanguage::ZhCn, PageKind::Home)
        );
        assert_eq!(
            crate::site::routing::site_route_from_path("/fr/"),
            SiteRoute::new(SiteLanguage::FrFr, PageKind::Home)
        );
        assert_eq!(
            crate::site::routing::site_route_from_path("/unknown"),
            SiteRoute::new(SiteLanguage::EnUs, PageKind::Home)
        );
    }

    #[test]
    fn sitemap_includes_index_and_project_sites() {
        let sitemap = crate::sitemap_xml();

        assert!(sitemap.contains("<loc>https://stayhydated.github.io/</loc>"));
        assert!(sitemap.contains("<loc>https://stayhydated.github.io/koruma/</loc>"));
        assert!(sitemap.contains("<loc>https://stayhydated.github.io/es-fluent/</loc>"));
    }

    #[test]
    fn cleans_generated_route_cache_without_touching_project_dirs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let public_dir = temp.path();

        fs::write(public_dir.join("index.html"), "root").expect("write root index");
        fs::write(public_dir.join("404.html"), "not found").expect("write root 404");
        fs::create_dir_all(public_dir.join("zh")).expect("create zh dir");
        fs::write(public_dir.join("zh").join("index.html"), "stale zh").expect("write zh index");
        fs::create_dir_all(public_dir.join("koruma")).expect("create koruma dir");
        fs::write(public_dir.join("koruma").join("index.html"), "koruma").expect("write koruma");
        fs::create_dir_all(public_dir.join("assets")).expect("create assets dir");
        fs::write(public_dir.join("assets").join("site.css"), "body {}").expect("write asset");

        crate::mark_generated_route_cache(public_dir).expect("mark route cache");
        crate::cleanup_generated_route_cache(public_dir).expect("cleanup route cache");

        assert!(!public_dir.join("index.html").exists());
        assert!(!public_dir.join("404.html").exists());
        assert!(!public_dir.join("zh").exists());
        assert!(public_dir.join("koruma").join("index.html").exists());
        assert!(public_dir.join("assets").join("site.css").exists());
    }
}
