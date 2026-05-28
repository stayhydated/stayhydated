use dioxus::prelude::*;

#[component]
pub fn SharedStyles() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("./theme.css") }
        document::Stylesheet { href: asset!("./layout.css") }
        document::Stylesheet { href: asset!("./navigation.css") }
        document::Stylesheet { href: asset!("./cards.css") }
        document::Stylesheet { href: asset!("./motion.css") }
        document::Stylesheet { href: asset!("./demo.css") }
    }
}
