use bon::Builder;
use dioxus::{document, prelude::*};

use crate::select;

#[derive(Builder, Clone, Debug)]
#[builder(on(String, into))]
pub struct ProjectOption {
    pub id: String,
    pub mark: String,
    pub name: String,
    pub description: String,
    pub href: String,
}

impl ProjectOption {
    fn text_value(&self) -> String {
        format!("{} {}", self.name, self.description)
    }
}

impl PartialEq for ProjectOption {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ProjectOption {}

pub fn stayhydated_project_options() -> Vec<ProjectOption> {
    vec![
        ProjectOption::builder()
            .id("stayhydated")
            .mark("SH")
            .name("stayhydated")
            .description("Project index")
            .href("/")
            .build(),
        ProjectOption::builder()
            .id("koruma")
            .mark("K")
            .name("koruma")
            .description("Rust validation")
            .href("/koruma/")
            .build(),
        ProjectOption::builder()
            .id("es-fluent")
            .mark("EF")
            .name("es-fluent")
            .description("Rust localization")
            .href("/es-fluent/")
            .build(),
    ]
}

#[component]
pub fn ProjectLockup(project: ProjectOption, #[props(default)] compact: bool) -> Element {
    let class = if compact {
        "project-lockup is-compact"
    } else {
        "project-lockup"
    };

    rsx! {
        div { class,
            span { class: "brand-mark project-mark", "{project.mark}" }
            span { class: "brand-copy project-copy",
                if !project.description.is_empty() {
                    span { class: "brand-kicker project-description", "{project.description}" }
                }
                span { class: "brand-title project-name", "{project.name}" }
            }
        }
    }
}

#[component]
pub fn ProjectSelect(
    selected: ProjectOption,
    projects: Vec<ProjectOption>,
    #[props(default = "Project".to_string())] label: String,
) -> Element {
    let initial_selected = selected.clone();
    let mut selected_project = use_signal(move || Some(initial_selected));
    let selected_for_effect = selected.clone();

    use_effect(move || {
        let next_selected = Some(selected_for_effect.clone());
        if selected_project() != next_selected {
            selected_project.set(next_selected);
        }
    });

    let trigger_project = selected_project().unwrap_or_else(|| selected.clone());
    let on_value_change = move |next_project: Option<ProjectOption>| {
        let Some(next_project) = next_project else {
            return;
        };

        if Some(next_project.clone()) == selected_project() {
            return;
        }

        selected_project.set(Some(next_project.clone()));
        navigate_to_project(next_project.href);
    };

    rsx! {
        div { class: "project-switcher",
            select::Select::<ProjectOption> {
                value: Some(selected_project.into()),
                on_value_change,
                select::SelectTrigger {
                    aria_label: label,
                    ProjectLockup {
                        project: trigger_project,
                    }
                }
                select::SelectList {
                    aria_label: "Projects",
                    for (index, project) in projects.iter().enumerate() {
                        {
                            let active = Some(project.clone()) == selected_project();
                            let option_class = if active {
                                "project-select-option is-active".to_string()
                            } else {
                                "project-select-option".to_string()
                            };
                            rsx! {
                                select::SelectOption::<ProjectOption> {
                                    index,
                                    value: project.clone(),
                                    text_value: Some(project.text_value()),
                                    class: Some(option_class),
                                    ProjectLockup {
                                        project: project.clone(),
                                        compact: true,
                                    }
                                    if active {
                                        select::SelectItemIndicator {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn navigate_to_project(href: String) {
    let eval = document::eval(
        r#"
        const href = await dioxus.recv();
        if (typeof href === "string" && href.length > 0) {
            const target = new URL(href, window.location.href);
            if (target.href !== window.location.href) {
                window.location.assign(target.href);
            }
        }
        "#,
    );
    let _ = eval.send(href);
}
