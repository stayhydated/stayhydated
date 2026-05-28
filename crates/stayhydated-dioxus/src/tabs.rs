use dioxus::prelude::*;
use dioxus_primitives::tabs::{self, TabContentProps, TabListProps, TabTriggerProps, TabsProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let base = attributes!(div {
        class: "collection-module-tabs"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        tabs::Tabs {
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            horizontal: props.horizontal,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TabList(props: TabListProps) -> Element {
    let base = attributes!(div {
        class: "collection-module-tab-list"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        tabs::TabList { attributes: merged, {props.children} }
    }
}

#[component]
pub fn TabTrigger(props: TabTriggerProps) -> Element {
    let base = attributes!(button {
        class: format!(
            "collection-module-tab {}",
            props.class.clone().unwrap_or_default()
        )
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        tabs::TabTrigger {
            class: None,
            id: props.id,
            value: props.value,
            index: props.index,
            disabled: props.disabled,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TabContent(props: TabContentProps) -> Element {
    let base = attributes!(div {
        class: format!(
            "collection-module-content {}",
            props.class.clone().unwrap_or_default()
        )
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        tabs::TabContent {
            class: None,
            value: props.value,
            id: props.id,
            index: props.index,
            attributes: merged,
            {props.children}
        }
    }
}
