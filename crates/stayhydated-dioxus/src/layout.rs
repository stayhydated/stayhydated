use dioxus::prelude::*;
use dioxus_primitives::navbar::{self, NavbarProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelKind {
    Hero,
    Section,
    Code,
    PageTitle,
    Contribute,
}

impl PanelKind {
    fn class(self) -> &'static str {
        match self {
            Self::Hero => "hero",
            Self::Section => "section-band",
            Self::Code => "code-band",
            Self::PageTitle => "page-title-band",
            Self::Contribute => "contribute-panel",
        }
    }
}

#[component]
pub fn PageShell(children: Element) -> Element {
    rsx! {
        main { class: "page-shell", {children} }
    }
}

#[component]
pub fn Panel(
    kind: PanelKind,
    children: Element,
    #[props(default)] extra_class: String,
    #[props(default)] style: String,
) -> Element {
    let class = join_classes(kind.class(), &extra_class);
    rsx! {
        section { class, style, {children} }
    }
}

#[component]
pub fn SharedGrid(
    children: Element,
    #[props(default)] columns: Option<u8>,
    #[props(default)] extra_class: String,
) -> Element {
    let class = match columns {
        Some(2) => join_classes("grid columns-2", &extra_class),
        Some(3) => join_classes("grid columns-3", &extra_class),
        _ => join_classes("grid", &extra_class),
    };

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn PageHeaderShell(props: NavbarProps) -> Element {
    let base = attributes!(div {
        class: "page-header"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        navbar::Navbar {
            disabled: props.disabled,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn LocaleSelect(label: String, children: Element) -> Element {
    rsx! {
        div { class: "locale-switcher-dropdown",
            span { class: "locale-label", "{label}" }
            {children}
        }
    }
}

#[component]
pub fn BrandMark(label: String) -> Element {
    rsx! {
        span { class: "brand-mark", "{label}" }
    }
}

#[component]
pub fn BrandLockup(href: String, mark: String, kicker: String, title: String) -> Element {
    rsx! {
        a { class: "brand", href,
            BrandMark { label: mark }
            span { class: "brand-copy",
                span { class: "brand-kicker", "{kicker}" }
                span { class: "brand-title", "{title}" }
            }
        }
    }
}

#[component]
pub fn ButtonLink(
    href: String,
    label: String,
    #[props(default = "primary".to_string())] variant: String,
) -> Element {
    rsx! {
        a { class: format!("button-link {variant}"), href, "{label}" }
    }
}

#[component]
pub fn Hero(children: Element, side: Option<Element>) -> Element {
    rsx! {
        section { class: "hero motion-reveal",
            div { class: "hero-copy", {children} }
            if let Some(side) = side {
                {side}
            }
        }
    }
}

#[component]
pub fn HeroSidePanel(
    children: Element,
    #[props(default = "workflow-panel".to_string())] class: String,
) -> Element {
    rsx! {
        aside { class, {children} }
    }
}

#[component]
pub fn PageTitleBand(label: Option<String>, title: String, lead: Option<String>) -> Element {
    rsx! {
        section { class: "page-title-band motion-reveal",
            if let Some(label) = label {
                span { class: "panel-label", "{label}" }
            }
            h1 { "{title}" }
            if let Some(lead) = lead {
                p { "{lead}" }
            }
        }
    }
}

#[component]
pub fn FooterPanel(children: Element) -> Element {
    rsx! {
        footer { class: "site-footer", {children} }
    }
}

#[component]
pub fn ContributePanelShell(children: Element, #[props(default)] style: String) -> Element {
    rsx! {
        section { class: "contribute-panel motion-reveal", style,
            div { class: "contribute-copy", {children} }
        }
    }
}

#[component]
pub fn FullscreenDemoFrame(src: String, title: String) -> Element {
    rsx! {
        div { class: "fullscreen-demo",
            iframe {
                class: "fullscreen-demo-frame",
                src,
                title,
            }
        }
    }
}

fn join_classes(base: &str, extra: &str) -> String {
    if extra.trim().is_empty() {
        base.to_owned()
    } else {
        format!("{base} {extra}")
    }
}
