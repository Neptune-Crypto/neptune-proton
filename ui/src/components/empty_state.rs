// File: src/components/empty_state.rs
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct EmptyStateProps {
    title: String,
    #[props(default)]
    description: Option<String>,
    #[props(default)]
    primary_action: Option<Element>,
    #[props(default)]
    icon: Option<Element>,
}

#[component]
pub fn EmptyState(props: EmptyStateProps) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                padding: 2rem;
                text-align: center;
                color: var(--pico-muted-color);
                border: 2px dashed var(--pico-card-border-color);
                border-radius: var(--pico-border-radius);
                background-color: var(--pico-card-sectioning-background-color);
                margin: 1rem 0;
            ",
            
            // Icon Container
            if let Some(icon) = props.icon {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        
                        /* PURE SCALING (VMIN): */
                        /* 20% of the viewport's smaller dimension (width or height). */
                        /* No min, no max. Just pure scaling. */
                        width: 20vmin;
                        height: 20vmin;
                        
                        /* Ensure Emojis match the box size exactly */
                        font-size: 20vmin;
                        
                        margin-bottom: 1rem;
                        color: var(--pico-primary-background); 
                        opacity: 0.8;
                    ",
                    {icon}
                }
            }

            h4 {
                style: "margin-bottom: 0.5rem; color: var(--pico-h4-color);",
                "{props.title}"
            }

            if let Some(desc) = props.description {
                p {
                    style: "max-width: 400px; margin: 0 auto 1.5rem auto;",
                    "{desc}"
                }
            }
            
            if let Some(action) = props.primary_action {
                div {
                    {action}
                }
            }
        }
    }
}
