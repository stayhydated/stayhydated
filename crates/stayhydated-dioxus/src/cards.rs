use dioxus::prelude::*;

#[component]
pub fn FeatureCard(
    label: String,
    title: String,
    body: String,
    #[props(default)] style: String,
) -> Element {
    rsx! {
        article { class: "feature-card motion-reveal",
            style,
            span { class: "card-label", "{label}" }
            h3 { "{title}" }
            p { "{body}" }
        }
    }
}

#[component]
pub fn SectionHeader(
    label: Option<String>,
    title: String,
    lead: Option<String>,
    #[props(default = "section-heading".to_string())] class: String,
) -> Element {
    rsx! {
        div { class,
            if let Some(label) = label {
                span { class: "panel-label", "{label}" }
            }
            h2 { "{title}" }
            if let Some(lead) = lead {
                p { "{lead}" }
            }
        }
    }
}

#[component]
pub fn CodeBlock(
    code: String,
    #[props(default = "code-sample".to_string())] class: String,
) -> Element {
    rsx! {
        pre { class, code { "{code}" } }
    }
}
