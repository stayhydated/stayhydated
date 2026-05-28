use crate::site::constants::GITHUB_ORG_URL;
use crate::site::i18n::{SiteChromeMessage, SiteLanguage};
use crate::site::routing::{PageKind, app_route};
use dioxus::prelude::*;
use dioxus::router::{navigator, try_router};
use dioxus_free_icons::{Icon, icons::ld_icons::LdGithub};
use stayhydated_dioxus::{
    LanguageSelect, ProjectOption, ProjectSelect, stayhydated_project_options,
};

#[component]
pub(crate) fn PageHeader(locale: SiteLanguage, current_page: PageKind) -> Element {
    let i18n = match es_fluent_manager_dioxus::use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => return rsx! { header { class: "page-header", "failed: {error}" } },
    };
    let brand_kicker = i18n.localize_message(&SiteChromeMessage::BrandKicker);
    let site_name = i18n.localize_message(&SiteChromeMessage::SiteName);
    let github_label = i18n.localize_message(&SiteChromeMessage::NavGithub);

    rsx! {
        header { class: "page-header",
            ProjectSelect {
                selected: ProjectOption::builder()
                    .id("stayhydated")
                    .mark("SH")
                    .name(site_name)
                    .description(brand_kicker)
                    .href(crate::site::routing::page_href(locale, PageKind::Home))
                    .build(),
                projects: stayhydated_project_options(),
                label: "Project selector".to_string(),
            }
            div { class: "header-cluster",
                a {
                    class: "icon-nav-link",
                    href: GITHUB_ORG_URL,
                    target: "_blank",
                    rel: "noreferrer",
                    aria_label: github_label.clone(),
                    title: github_label,
                    Icon {
                        width: 20,
                        height: 20,
                        icon: LdGithub,
                    }
                }
                LocaleSwitcher { locale, current_page }
            }
        }
    }
}

#[component]
fn LocaleSwitcher(locale: SiteLanguage, current_page: PageKind) -> Element {
    let i18n = match es_fluent_manager_dioxus::use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => return rsx! { div { class: "locale-switcher-dropdown", "failed: {error}" } },
    };
    let locale_label = i18n.localize_message(&SiteChromeMessage::LocaleLabel);
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
