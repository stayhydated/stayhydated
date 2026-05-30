use crate::site::constants::{HOME_PAGE_DESCRIPTION, SITE_NAME};
use crate::site::i18n::SiteLanguage;
use dioxus::cli_config;
use dioxus::prelude::*;
use std::fmt::{self, Display};
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PageKind {
    Home,
}

impl PageKind {
    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::Home => "",
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Home => SITE_NAME,
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Home => HOME_PAGE_DESCRIPTION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SiteRoute {
    pub(crate) locale: SiteLanguage,
    pub(crate) page: PageKind,
}

impl SiteRoute {
    pub(crate) const fn new(locale: SiteLanguage, page: PageKind) -> Self {
        Self { locale, page }
    }

    pub(crate) fn output_dir(self) -> String {
        relative_path(self.locale, self.page)
    }

    pub(crate) fn path(self) -> String {
        let relative = self.output_dir();

        if relative.is_empty() {
            "/".to_string()
        } else {
            format!("/{relative}/")
        }
    }
}

pub(crate) fn all_routes() -> Vec<SiteRoute> {
    SiteLanguage::all()
        .map(|locale| SiteRoute::new(locale, PageKind::Home))
        .collect()
}

pub(crate) fn app_base_href() -> String {
    let base_path = cli_config::base_path();
    stayhydated_site::routing::base_href(base_path.as_deref())
}

pub(crate) fn page_href(locale: SiteLanguage, page: PageKind) -> String {
    stayhydated_site::routing::href(&app_base_href(), &relative_path(locale, page))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocaleSegment(SiteLanguage);

impl LocaleSegment {
    fn language(&self) -> SiteLanguage {
        self.0
    }
}

impl Display for LocaleSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.route_slug() {
            Some(slug) => f.write_str(&slug),
            None => Err(fmt::Error),
        }
    }
}

impl FromStr for LocaleSegment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SiteLanguage::from_route_slug(s)
            .map(Self)
            .ok_or_else(|| format!("unsupported locale route segment: {s}"))
    }
}

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub(crate) enum AppRoute {
    #[route("/", HomeRoute)]
    Home {},
    #[route("/:locale/", LocalizedHomeRoute)]
    LocalizedHome { locale: LocaleSegment },
}

pub(crate) fn app_route(locale: SiteLanguage, page: PageKind) -> AppRoute {
    match (locale.route_slug(), page) {
        (None, PageKind::Home) => AppRoute::Home {},
        (Some(_), PageKind::Home) => AppRoute::LocalizedHome {
            locale: LocaleSegment(locale),
        },
    }
}

#[cfg(test)]
pub(crate) fn site_route_from_path(path: &str) -> SiteRoute {
    site_route_from_path_with_base_path(path, None)
}

#[cfg(test)]
pub(crate) fn site_route_from_path_with_base_path(
    path: &str,
    base_path: Option<&str>,
) -> SiteRoute {
    let segments = normalized_path_segments(path, base_path);

    let locale = match segments.as_slice().split_first() {
        Some((first, _)) => {
            SiteLanguage::from_route_slug(first).unwrap_or_else(SiteLanguage::default)
        },
        None => SiteLanguage::default(),
    };

    SiteRoute::new(locale, PageKind::Home)
}

#[cfg(test)]
fn normalized_path_segments<'a>(path: &'a str, base_path: Option<&str>) -> Vec<&'a str> {
    stayhydated_site::routing::normalized_path_segments(path, base_path)
}

fn relative_path(locale: SiteLanguage, page: PageKind) -> String {
    let mut segments = Vec::new();

    if let Some(slug) = locale.route_slug() {
        segments.push(slug);
    }

    let page_segment = page.route();
    if !page_segment.is_empty() {
        segments.push(page_segment.to_string());
    }

    segments.join("/")
}

const GENERATED_ROUTE_CACHE_MARKER: &str = ".stayhydated-generated-route-cache";

pub(crate) fn mark_generated_route_cache(public_dir: &Path) -> std::io::Result<()> {
    stayhydated_site::route_cache::mark_generated_route_cache(
        public_dir,
        GENERATED_ROUTE_CACHE_MARKER,
        "Generated route cache owned by stayhydated web server.\n",
    )
}

pub(crate) fn cleanup_generated_route_cache(public_dir: &Path) -> std::io::Result<()> {
    let generated_top_level_dirs = all_routes()
        .into_iter()
        .filter_map(|route| {
            route
                .output_dir()
                .split('/')
                .next()
                .filter(|segment| !segment.is_empty())
                .map(str::to_string)
        })
        .collect::<Vec<_>>();

    stayhydated_site::route_cache::cleanup_generated_route_cache(
        public_dir,
        GENERATED_ROUTE_CACHE_MARKER,
        generated_top_level_dirs,
        |path, name| {
            SiteLanguage::from_route_slug(name).is_some() && contains_generated_route_cache(path)
        },
    )
}

fn contains_generated_route_cache(dir: &Path) -> bool {
    dir.join("index.html").is_file()
}

fn route_element(route: SiteRoute) -> Element {
    let i18n = match es_fluent_manager_dioxus::use_asset_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "page-shell", "failed to access localization context: {error}" }
            };
        },
    };
    let route_language = route.locale.lang();
    let i18n_result = if i18n.peek_requested_language() == route_language {
        Ok(i18n)
    } else {
        i18n.select_language(route_language)
            .map(|()| i18n)
            .map_err(|error| {
                format!(
                    "failed to select localized route '{}': {error}",
                    route.locale.html_lang()
                )
            })
    };

    match i18n_result {
        Ok(i18n) => {
            let _ = i18n.requested_language();
            let title = route.page.title();
            let description = route.page.description();

            rsx! {
                Title { "{title}" }
                Meta {
                    name: "description",
                    content: description,
                }
                {crate::pages::route_content(route)}
            }
        },
        Err(error) => rsx! {
            div { class: "page-shell", "failed: {error}" }
        },
    }
}

#[server(endpoint = "static_routes")]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    Ok(all_routes()
        .into_iter()
        .map(|route| page_href(route.locale, route.page))
        .collect())
}

#[component]
fn HomeRoute() -> Element {
    route_element(SiteRoute::new(SiteLanguage::default(), PageKind::Home))
}

#[component]
fn LocalizedHomeRoute(locale: LocaleSegment) -> Element {
    route_element(SiteRoute::new(locale.language(), PageKind::Home))
}
