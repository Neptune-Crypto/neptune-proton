// ui/src/components/currency_chooser.rs
#![allow(non_snake_case)]

use api::fiat_currency::FiatCurrency;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone, Copy)]
pub struct CurrencyInfo {
    pub short_name: &'static str,
    pub long_name: &'static str,
}

impl Default for CurrencyInfo {
    fn default() -> Self {
        Self {
            short_name: "NPT",
            long_name: "Neptune Cash",
        }
    }
}

impl From<FiatCurrency> for CurrencyInfo {
    fn from(currency: FiatCurrency) -> Self {
        Self {
            short_name: currency.code(),
            long_name: currency.name(),
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct CurrencyChooserProps {
    /// A signal holding the short_name of the currently displayed item.
    pub displayed_id: Signal<&'static str>,
    /// A signal holding the short_name of the user's preferred fiat currency.
    pub preferred_fiat_id: Signal<&'static str>,
    /// A vector of all available fiat currencies.
    pub all_fiats: Vec<CurrencyInfo>,
    #[props(optional)]
    pub style: Option<String>,
}

/// A specialized split-button component for toggling and selecting currencies.
pub fn CurrencyChooser(mut props: CurrencyChooserProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut filter_text = use_signal(|| "".to_string());

    let secondary_currency = CurrencyInfo::default();

    let displayed_id_val = *props.displayed_id.read();
    let display_text = if displayed_id_val == secondary_currency.short_name {
        secondary_currency.short_name
    } else {
        displayed_id_val
    };

    let preferred_fiat_long_name = props
        .all_fiats
        .iter()
        .find(|fiat| fiat.short_name == *props.preferred_fiat_id.read())
        .map(|fiat| fiat.long_name)
        .unwrap_or_else(|| &props.preferred_fiat_id.read());

    let tooltip = format!(
        "Click to toggle between {} and {}.",
        secondary_currency.long_name, preferred_fiat_long_name
    );

    let filtered_fiats = props
        .all_fiats
        .iter()
        .filter(|fiat| {
            let filter_lower = filter_text.read().to_lowercase();
            fiat.long_name.to_lowercase().contains(&filter_lower)
                || fiat.short_name.to_lowercase().contains(&filter_lower)
        })
        .copied()
        .collect::<Vec<_>>();

    rsx! {
        div {
            style: "{props.style.as_deref().unwrap_or(\"\")}",
            div {
                style: "position: relative; width: 4rem;",
                div {
                    class: "secondary",
                    style: "
                        display: flex;
                        align-items: center;
                        padding: 0;
                        line-height: 1.2;
                        font-size: 0.875rem;
                        cursor: pointer;
                        ",
                    div {
                        style: "flex-grow: 1; padding: 0.375rem 0.2rem; cursor: pointer; text-align: center;",
                        title: "{tooltip}",
                        onclick: move |_| {
                            let current_mode = *props.displayed_id.read();
                            if current_mode == secondary_currency.short_name {
                                props.displayed_id.set(*props.preferred_fiat_id.read());
                            } else {
                                props.displayed_id.set(secondary_currency.short_name);
                            }
                        },
                        "{display_text}"
                    }
                    div {
                        style: "border-left: 1px solid var(--pico-secondary-border); padding: 0.1rem 0.2rem; cursor: pointer;",
                        onclick: move |_| is_open.toggle(),
                        title: "Choose national currency.",
                        "↓"
                    }
                }
                if is_open() {
                    // Backdrop to catch clicks outside the dropdown
                    div {
                        style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 9; background: transparent;",
                        onclick: move |_| is_open.set(false),
                    }
                    div {
                        // Stop click propagation to prevent the backdrop from closing the dropdown
                        onclick: |e| e.stop_propagation(),
                        style: "
                            position: absolute;
                            min-width: 100%;
                            z-index: 10;
                            background-color: var(--pico-card-background-color);
                            border: 1px solid var(--pico-card-border-color);
                            border-radius: var(--pico-border-radius);
                            padding: 0.5rem;
                            margin-top: 0.25rem;
                        ",
                        input {
                            r#type: "text",
                            placeholder: "Search currencies...",
                            value: "{filter_text}",
                            oninput: move |evt| filter_text.set(evt.value()),
                            style: "margin-bottom: 0.5rem; width: 100%;",
                            onmounted: move |mounted| {
                                spawn(async move {
                                    mounted.data.set_focus(true).await.ok();
                                });
                            },
                        }
                        ul {
                            role: "listbox",
                            style: "list-style: none; margin: 0; padding: 0; max-height: 250px; overflow-y: auto;",
                            {
                                filtered_fiats
                                    .into_iter()
                                    .map(|fiat| {
                                        let is_preferred = *props.preferred_fiat_id.read() == fiat.short_name;
                                        let display_label = format!("{} - {}", fiat.short_name, fiat.long_name);
                                        rsx! {
                                            li {
                                                key: "{fiat.short_name}",
                                                style: "display: flex; align-items: center; cursor: pointer; padding: 0.3rem; white-space: nowrap;",
                                                onclick: move |_| {
                                                    props.preferred_fiat_id.set(fiat.short_name);
                                                    props.displayed_id.set(fiat.short_name);
                                                    is_open.set(false);
                                                },
                                                if is_preferred {
                                                    span {
                                                        style: "width: 1.5rem;",
                                                        "✓"
                                                    }
                                                } else {
                                                    span {
                                                        style: "width: 1.5rem; visibility: hidden;",
                                                        "✓"
                                                    }
                                                }
                                                span {

                                                    "{display_label}"
                                                }
                                            }
                                        }
                                    })
                            }
                        }
                    }
                }
            }
        }
    }
}
