use dioxus::prelude::*;
use crate::Screen; // Adjust this import to where your Screen enum is

#[derive(Props, Clone, PartialEq)]
pub struct ActionLinkProps {
    // In 0.7, use Signal<T> instead of &UseState<T>. 
    // Signals are Copy, so we don't need lifetimes or references here.
    #[props(optional)]
    pub state: Option<Signal<Screen>>,

    #[props(optional)]
    pub to: Option<Screen>,

    // Note: MouseEvent is the standard type alias in Dioxus 0.7
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn ActionLink(props: ActionLinkProps) -> Element {
    rsx! {
        a {
            href: "#",
            // 1. We specify MouseEvent explicitly to satisfy the compiler
            onclick: move |evt: MouseEvent| {
                // 2. Prevent the "new tab" behavior
                evt.prevent_default();

                // 3. Handle Navigation
                // We use 'mut' to get write access to the Signal
                if let (Some(mut state_signal), Some(target)) = (props.state, &props.to) {
                    state_signal.set(target.clone());
                }

                // 4. Handle Custom Logic
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },
            {props.children}
        }
    }
}
