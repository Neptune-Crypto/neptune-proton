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

            if let Some(icon) = props.icon {
                {
                    rsx! {
                        style {
                            "
                            /* 1. Force SVG to fill the clamped box & show overflow */
                            .empty-state-icon-wrapper svg {{
                                width: 100% !important;
                                height: 100% !important;
                                min-width: 100% !important;
                                min-height: 100% !important;
                                overflow: visible !important; 
                            }}

                            /* 2. FIX FOR 3D/STAR WARS OFFSET */
                            /* Target all groups/paths inside the SVG */
                            .empty-state-icon-wrapper svg * {{
                                /* Force the origin to be the center of the VIEWBOX, not the element */
                                /* 'view-box' is crucial for scene-wide 3D effects like scrolling text */
                                transform-box: view-box !important; 
                                
                                /* Ensure rotation happens around the center */
                                transform-origin: center center !important;
                            }}
                            "
                        }
                        div {
                            class: "empty-state-icon-wrapper",
                            style: "
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        
                        /* 1. FORCE SQUARE & SCALING */
                        width: clamp(100px, 30vmin, 1000px);
                        height: clamp(100px, 30vmin, 1000px);
                        font-size: clamp(100px, 30vmin, 1000px);
                        
                        /* 2. NUCLEAR OPTION FOR CLIPPING */
                        /* This forces the browser to cut off anything that drifts outside */
                        overflow: hidden; 
                        
                        /* 3. RELATIVE POSITIONING */
                        /* Ensures absolute children inside the SVG know where '0,0' is */
                        position: relative;
                        
                        margin-bottom: 1rem;
                        color: var(--pico-primary-background); 
                        opacity: 0.8;                            
                            ",
                            
                            {icon}
                        }
                    }
                }
            }

            h4 {
                style: "margin-bottom: 0.5rem; color: var(--pico-h4-color);",
                "{props.title}"
            }

            if let Some(desc) = props.description {
                p {
                    style: "max-width: 600px; margin: 0 auto 1.5rem auto;",
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
