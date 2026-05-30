use crate::site::constants::SITE_NAME;
use crate::site::i18n::{SiteLanguage, SiteMessage};
use crate::site::routing::{PageKind, app_route};
use dioxus::prelude::*;
use dioxus::router::{navigator, try_router};
use stayhydated_dioxus::{
    LanguageSelect, ProjectOption, ProjectSelect, ProjectSelectMessage, StayhydatedProject,
};

#[component]
pub(crate) fn PageHeader(locale: SiteLanguage, current_page: PageKind) -> Element {
    let i18n = match es_fluent_manager_dioxus::use_asset_i18n() {
        Ok(i18n) => i18n,
        Err(error) => return rsx! { header { class: "page-header", "failed: {error}" } },
    };
    let brand_kicker = i18n.localize_message(&SiteMessage::BrandKicker);
    let selected_project = StayhydatedProject::Stayhydated.option_with(
        SITE_NAME,
        brand_kicker.clone(),
        crate::site::routing::page_href(locale, PageKind::Home),
    );
    let projects = localized_project_options(selected_project.clone(), &i18n);
    let project_selector_label = i18n.localize_message(&ProjectSelectMessage::ProjectSelectorLabel);
    let project_list_label = i18n.localize_message(&ProjectSelectMessage::ProjectListLabel);

    rsx! {
        header { class: "page-header",
            ProjectSelect {
                selected: selected_project,
                projects,
                label: project_selector_label,
                list_label: project_list_label,
            }
            div { class: "header-cluster",
                LocaleSwitcher { locale, current_page }
            }
        }
    }
}

fn localized_project_options(
    current_site: ProjectOption,
    i18n: &impl es_fluent::FluentLocalizer,
) -> Vec<ProjectOption> {
    vec![
        current_site,
        StayhydatedProject::Koruma.localized_option(i18n),
        StayhydatedProject::EsFluent.localized_option(i18n),
    ]
}

#[component]
fn LocaleSwitcher(locale: SiteLanguage, current_page: PageKind) -> Element {
    let i18n = match es_fluent_manager_dioxus::use_asset_i18n() {
        Ok(i18n) => i18n,
        Err(error) => return rsx! { div { class: "locale-switcher-dropdown", "failed: {error}" } },
    };
    let locale_label = i18n.localize_message(&SiteMessage::LocaleLabel);
    let language_links = SiteLanguage::all()
        .map(|candidate| {
            let label = i18n.localize_message(&candidate);
            (candidate, label)
        })
        .collect::<Vec<_>>();
    let on_locale_changed = move |next_locale: SiteLanguage| {
        if next_locale == locale {
            return;
        }

        if try_router().is_some() {
            let _ = navigator().push(app_route(next_locale, current_page));
        }
    };

    rsx! {
        LanguageSelect::<SiteLanguage> {
            label: locale_label,
            selected: locale,
            options: language_links,
            on_change: on_locale_changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::constants::{ES_FLUENT_SITE_URL, KORUMA_SITE_URL};
    use es_fluent::{FluentLocalizer, FluentValue};
    use std::collections::HashMap;

    struct TestLocalizer;

    impl FluentLocalizer for TestLocalizer {
        fn localize<'a>(
            &self,
            id: &str,
            args: Option<&HashMap<&str, FluentValue<'a>>>,
        ) -> Option<String> {
            self.localize_in_domain("web", id, args)
        }

        fn localize_in_domain<'a>(
            &self,
            domain: &str,
            id: &str,
            _args: Option<&HashMap<&str, FluentValue<'a>>>,
        ) -> Option<String> {
            match (domain, id) {
                ("stayhydated-dioxus", "project_select_message-EsFluentDescription") => {
                    Some("Rust 本地化".to_string())
                },
                ("stayhydated-dioxus", "project_select_message-KorumaDescription") => {
                    Some("Rust 校验".to_string())
                },
                _ => None,
            }
        }
    }

    #[test]
    fn project_options_use_localized_descriptions() {
        let projects = localized_project_options(
            StayhydatedProject::Stayhydated.option_with("stayhydated", "项目索引", "/zh/"),
            &TestLocalizer,
        );

        assert_eq!(projects[0].description, "项目索引");
        assert_eq!(projects[0].href, "/zh/");
        assert_eq!(projects[1].description, "Rust 校验");
        assert_eq!(projects[1].href, KORUMA_SITE_URL);
        assert_eq!(projects[2].description, "Rust 本地化");
        assert_eq!(projects[2].href, ES_FLUENT_SITE_URL);
    }
}
