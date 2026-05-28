use dioxus::prelude::*;

use crate::{LocaleSelect, select};

#[component]
pub fn LanguageSelect<T: Clone + PartialEq + 'static>(
    label: String,
    selected: T,
    options: Vec<(T, String)>,
    on_change: Callback<T>,
) -> Element {
    let initial_selected = selected.clone();
    let mut selected_value = use_signal(move || Some(initial_selected));
    let selected_for_effect = selected;

    use_effect(move || {
        let next_selected = Some(selected_for_effect.clone());
        if selected_value() != next_selected {
            selected_value.set(next_selected);
        }
    });

    let placeholder = label.clone();
    let on_value_change = move |next_value: Option<T>| {
        let Some(next_value) = next_value else {
            return;
        };

        if Some(next_value.clone()) == selected_value() {
            return;
        }

        selected_value.set(Some(next_value.clone()));
        on_change.call(next_value);
    };

    rsx! {
        LocaleSelect { label,
            select::Select::<T> {
                value: Some(selected_value.into()),
                on_value_change,
                select::SelectTrigger {
                    select::SelectValue {
                        placeholder,
                        class: Some("header-locale-value".to_string()),
                    }
                }
                select::SelectList {
                    for (index, (value, option_label)) in options.iter().enumerate() {
                        select::SelectOption::<T> {
                            index,
                            value: value.clone(),
                            text_value: Some(option_label.clone()),
                            class: Some(if Some(value.clone()) == selected_value() {
                                "header-locale-option is-active".to_string()
                            } else {
                                "header-locale-option".to_string()
                            }),
                            "{option_label}"
                            if Some(value.clone()) == selected_value() {
                                select::SelectItemIndicator {}
                            }
                        }
                    }
                }
            }
        }
    }
}
