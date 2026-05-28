use crate::components::PageHeader;
use crate::site::constants::{
    ES_FLUENT_GITHUB_URL, ES_FLUENT_SITE_URL, KORUMA_GITHUB_URL, KORUMA_SITE_URL,
};
use crate::site::i18n::{HomeMessage, SiteLanguage};
use crate::site::routing::PageKind;
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdBookOpen, LdGithub, LdLanguages, LdShieldCheck},
};
use es_fluent_manager_dioxus::use_i18n;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProjectKind {
    Koruma,
    EsFluent,
}

#[derive(Clone, Debug, PartialEq)]
struct ProjectSummary {
    kind: ProjectKind,
    title: String,
    site_href: &'static str,
    source_href: &'static str,
}

#[component]
pub(crate) fn HomePage(locale: SiteLanguage) -> Element {
    let validation_style = crate::components::use_reveal_style(0, 18.0);
    let localization_style = crate::components::use_reveal_style(80, 18.0);
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => return rsx! { div { class: "page-shell", "failed: {error}" } },
    };

    let validation_title = i18n.localize_message(&HomeMessage::ValidationSectionTitle);
    let localization_title = i18n.localize_message(&HomeMessage::LocalizationSectionTitle);
    let site_action = i18n.localize_message(&HomeMessage::ProjectSiteAction);
    let source_action = i18n.localize_message(&HomeMessage::ProjectSourceAction);
    let koruma_project = ProjectSummary {
        kind: ProjectKind::Koruma,
        title: i18n.localize_message(&HomeMessage::KorumaTitle),
        site_href: KORUMA_SITE_URL,
        source_href: KORUMA_GITHUB_URL,
    };
    let es_fluent_project = ProjectSummary {
        kind: ProjectKind::EsFluent,
        title: i18n.localize_message(&HomeMessage::EsFluentTitle),
        site_href: ES_FLUENT_SITE_URL,
        source_href: ES_FLUENT_GITHUB_URL,
    };

    rsx! {
        div { class: "page-shell",
            PageHeader { locale, current_page: PageKind::Home }
            main { class: "stack project-directory",
                ProjectSection {
                    title: validation_title,
                    project: koruma_project,
                    site_action: site_action.clone(),
                    source_action: source_action.clone(),
                    style: validation_style,
                }
                ProjectSection {
                    title: localization_title,
                    project: es_fluent_project,
                    site_action,
                    source_action,
                    style: localization_style,
                }
            }
        }
    }
}

#[component]
fn ProjectSection(
    title: String,
    project: ProjectSummary,
    site_action: String,
    source_action: String,
    style: String,
) -> Element {
    rsx! {
        section { class: "section-band motion-reveal project-directory-section", style,
            div { class: "section-heading directory-heading",
                h2 { "{title}" }
            }
            div { class: "project-list",
                ProjectCard {
                    project,
                    site_action,
                    source_action,
                }
            }
        }
    }
}

#[component]
fn ProjectCard(project: ProjectSummary, site_action: String, source_action: String) -> Element {
    rsx! {
        article { class: "directory-project-row",
            div { class: "directory-project-title",
                span { class: "project-card-icon",
                    match project.kind {
                        ProjectKind::Koruma => rsx!(Icon {
                            width: 24,
                            height: 24,
                            icon: LdShieldCheck,
                        }),
                        ProjectKind::EsFluent => rsx!(Icon {
                            width: 24,
                            height: 24,
                            icon: LdLanguages,
                        }),
                    }
                }
                h3 { "{project.title}" }
            }
            div { class: "project-card-actions",
                a { class: "project-card-action primary", href: project.site_href,
                    Icon {
                        width: 17,
                        height: 17,
                        icon: LdBookOpen,
                    }
                    span { "{site_action}" }
                }
                a {
                    class: "project-card-action",
                    href: project.source_href,
                    target: "_blank",
                    rel: "noreferrer",
                    Icon {
                        width: 17,
                        height: 17,
                        icon: LdGithub,
                    }
                    span { "{source_action}" }
                }
            }
        }
    }
}
