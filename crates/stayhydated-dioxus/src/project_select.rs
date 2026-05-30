use bon::Builder;
use dioxus::{document, prelude::*};
use es_fluent::{EsFluent, FluentLocalizer, FluentLocalizerExt as _};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StayhydatedProject {
    Stayhydated,
    Koruma,
    EsFluent,
}

#[derive(Clone, Copy, Debug, EsFluent)]
pub enum ProjectSelectMessage {
    EsFluentDescription,
    KorumaDescription,
    ProjectListLabel,
    ProjectSelectorLabel,
    StayhydatedDescription,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProjectMetadata {
    id: &'static str,
    mark: &'static str,
    name: &'static str,
    href: &'static str,
}

impl StayhydatedProject {
    pub const ALL: [Self; 3] = [Self::Stayhydated, Self::Koruma, Self::EsFluent];

    const fn metadata(self) -> ProjectMetadata {
        match self {
            Self::Stayhydated => ProjectMetadata {
                id: "stayhydated",
                mark: "SH",
                name: "stayhydated",
                href: "/",
            },
            Self::Koruma => ProjectMetadata {
                id: "koruma",
                mark: "K",
                name: "koruma",
                href: "/koruma/",
            },
            Self::EsFluent => ProjectMetadata {
                id: "es-fluent",
                mark: "EF",
                name: "es-fluent",
                href: "/es-fluent/",
            },
        }
    }

    const fn description_message(self) -> Option<ProjectSelectMessage> {
        match self {
            Self::Stayhydated => Some(ProjectSelectMessage::StayhydatedDescription),
            Self::Koruma => Some(ProjectSelectMessage::KorumaDescription),
            Self::EsFluent => Some(ProjectSelectMessage::EsFluentDescription),
        }
    }

    pub fn option(self) -> ProjectOption {
        let metadata = self.metadata();
        self.option_with(metadata.name, "", metadata.href)
    }

    pub fn localized_option<L>(self, i18n: &L) -> ProjectOption
    where
        L: FluentLocalizer + ?Sized,
    {
        let metadata = self.metadata();
        let description = self
            .description_message()
            .map(|message| i18n.localize_message(&message))
            .unwrap_or_default();

        self.option_with(metadata.name, description, metadata.href)
    }

    pub fn option_with(
        self,
        name: impl Into<String>,
        description: impl Into<String>,
        href: impl Into<String>,
    ) -> ProjectOption {
        let metadata = self.metadata();
        ProjectOption::builder()
            .id(metadata.id)
            .mark(metadata.mark)
            .name(name)
            .description(description)
            .href(href)
            .build()
    }
}

impl From<StayhydatedProject> for ProjectOption {
    fn from(project: StayhydatedProject) -> Self {
        project.option()
    }
}

impl ProjectOption {
    fn has_same_content(&self, other: &Self) -> bool {
        self.id == other.id
            && self.mark == other.mark
            && self.name == other.name
            && self.description == other.description
            && self.href == other.href
    }

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
    StayhydatedProject::ALL
        .into_iter()
        .map(StayhydatedProject::option)
        .collect()
}

pub fn localized_stayhydated_project_options<L>(i18n: &L) -> Vec<ProjectOption>
where
    L: FluentLocalizer + ?Sized,
{
    StayhydatedProject::ALL
        .into_iter()
        .map(|project| project.localized_option(i18n))
        .collect()
}

#[derive(Clone, Props)]
pub struct ProjectLockupProps {
    pub project: ProjectOption,
    #[props(default)]
    pub compact: bool,
}

impl PartialEq for ProjectLockupProps {
    fn eq(&self, other: &Self) -> bool {
        self.compact == other.compact && self.project.has_same_content(&other.project)
    }
}

#[allow(non_snake_case)]
pub fn ProjectLockup(props: ProjectLockupProps) -> Element {
    let project = props.project;
    let compact = props.compact;
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

#[derive(Clone, Props)]
pub struct ProjectSelectProps {
    pub selected: ProjectOption,
    pub projects: Vec<ProjectOption>,
    #[props(default = "Project".to_string())]
    pub label: String,
    #[props(default = "Projects".to_string())]
    pub list_label: String,
}

impl PartialEq for ProjectSelectProps {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.list_label == other.list_label
            && self.selected.has_same_content(&other.selected)
            && project_options_have_same_content(&self.projects, &other.projects)
    }
}

#[allow(non_snake_case)]
pub fn ProjectSelect(props: ProjectSelectProps) -> Element {
    let selected = props.selected;
    let projects = props.projects;
    let label = props.label;
    let list_label = props.list_label;
    let initial_selected_id = selected.id.clone();
    let mut selected_project_id = use_signal(move || Some(initial_selected_id));
    let selected_id = selected.id.clone();

    use_effect(use_reactive((&selected_id,), move |(next_selected_id,)| {
        let next_selected = Some(next_selected_id);

        if selected_project_id() != next_selected {
            selected_project_id.set(next_selected);
        }
    }));

    let trigger_project = selected_project_id()
        .as_ref()
        .and_then(|current_id| {
            projects
                .iter()
                .find(|project| project.id == *current_id)
                .cloned()
        })
        .unwrap_or_else(|| selected.clone());
    let projects_for_change = projects.clone();
    let on_value_change = move |next_project_id: Option<String>| {
        let Some(next_project_id) = next_project_id else {
            return;
        };

        if Some(next_project_id.clone()) == selected_project_id() {
            return;
        }

        let Some(next_project) = projects_for_change
            .iter()
            .find(|project| project.id == next_project_id)
            .cloned()
        else {
            return;
        };

        selected_project_id.set(Some(next_project_id));
        navigate_to_project(next_project.href);
    };

    rsx! {
        div { class: "project-switcher",
            select::Select::<String> {
                value: Some(selected_project_id.into()),
                on_value_change,
                select::SelectTrigger {
                    aria_label: label,
                    ProjectLockup {
                        project: trigger_project,
                    }
                }
                select::SelectList {
                    aria_label: list_label,
                    for (index, project) in projects.iter().enumerate() {
                        {
                            let active = Some(project.id.clone()) == selected_project_id();
                            let option_class = if active {
                                "project-select-option is-active".to_string()
                            } else {
                                "project-select-option".to_string()
                            };
                            rsx! {
                                select::SelectOption::<String> {
                                    index,
                                    value: project.id.clone(),
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

fn project_options_have_same_content(left: &[ProjectOption], right: &[ProjectOption]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.has_same_content(right))
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
