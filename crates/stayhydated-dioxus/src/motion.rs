use dioxus::prelude::*;

pub fn use_reveal_style(delay_ms: u64, distance_px: f32) -> String {
    format!("--motion-delay: {delay_ms}ms; --motion-distance: {distance_px:.2}px;")
}

#[component]
pub fn MotionReveal(children: Element, #[props(default)] style: String) -> Element {
    rsx! {
        div { class: "motion-reveal", style, {children} }
    }
}
