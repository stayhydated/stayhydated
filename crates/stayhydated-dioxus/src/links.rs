use dioxus::prelude::*;

#[component]
pub fn RouteLink<R: Routable + Clone + PartialEq + 'static>(
    route: R,
    href: String,
    class: String,
    label: String,
) -> Element {
    if try_router().is_some() {
        rsx! {
            Link {
                class,
                to: route,
                "{label}"
            }
        }
    } else {
        rsx! {
            a {
                class,
                href,
                "{label}"
            }
        }
    }
}

#[component]
pub fn NavLink<R: Routable + Clone + PartialEq + 'static>(
    route: R,
    href: String,
    label: String,
    #[props(default)] active: bool,
) -> Element {
    rsx! {
        RouteLink {
            route,
            href,
            class: if active {
                "header-nav-item is-active".to_string()
            } else {
                "header-nav-item".to_string()
            },
            label,
        }
    }
}

#[component]
pub fn BackLink<R: Routable + Clone + PartialEq + 'static>(
    route: R,
    href: String,
    label: String,
) -> Element {
    rsx! {
        RouteLink {
            route,
            href,
            class: "back-pill".to_string(),
            label,
        }
    }
}

#[component]
pub fn RouteCardLink<R: Routable + Clone + PartialEq + 'static>(
    route: R,
    href: String,
    label: String,
    title: String,
    body: String,
    body_class: String,
    action: String,
) -> Element {
    if try_router().is_some() {
        rsx! {
            Link {
                class: "demo-card",
                to: route,
                div { class: "card-label", "{label}" }
                h2 { "{title}" }
                p { class: body_class, "{body}" }
                span { class: "card-link", "{action}" }
            }
        }
    } else {
        rsx! {
            a {
                class: "demo-card",
                href,
                div { class: "card-label", "{label}" }
                h2 { "{title}" }
                p { class: body_class, "{body}" }
                span { class: "card-link", "{action}" }
            }
        }
    }
}
