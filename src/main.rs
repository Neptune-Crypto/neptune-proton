// The dioxus prelude contains a ton of common items used in dioxus apps. It's a good idea to import wherever you
// need dioxus
use dioxus::prelude::*;

// use components::Hero;
use views::{Blog, Home, Navbar};

use components::pico::{
    Accordion, Button, ButtonType, Card, Container, Grid, Input, Modal,
};

/// Define a components module that contains all shared components for our app.
mod components;
/// Define a views module that contains the UI for all Layouts and Routes for our app.
mod views;

/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
///
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great for wrapping
    // many routes with a common UI like a navbar.
    #[layout(Navbar)]
        // The route attribute defines the URL pattern that a specific route matches. If that pattern matches the URL,
        // the component for that route will be rendered. The component name that is rendered defaults to the variant name.
        #[route("/")]
        Home {},
        // The route attribute can include dynamic parameters that implement [`std::str::FromStr`] and [`std::fmt::Display`] with the `:` syntax.
        // In this case, id will match any integer like `/blog/123` or `/blog/-456`.
        #[route("/blog/:id")]
        // Fields of the route variant will be passed to the component as props. In this case, the blog component must accept
        // an `id` prop of type `i32`.
        Blog { id: i32 },
}

// We can import assets in dioxus with the `asset!` macro. This macro takes a path to an asset relative to the crate root.
// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
// const FAVICON: Asset = asset!("/assets/favicon.ico");
// The asset macro also minifies some assets like CSS and JS to make bundled smaller
// const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    // The `launch` function is the main entry point for a dioxus app. It takes a component and renders it with the platform feature
    // you have enabled
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
/*
#[component]
fn App() -> Element {
    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx! {
        // In addition to element and text (which we will see later), rsx can contain other components. In this case,
        // we are using the `document::Link` component to add a link to our favicon and main CSS file into the head of our app.
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }


        // The router component renders the route enum we defined above. It will handle synchronization of the URL and render
        // the layouts and components for the active route.
        Router::<Route> {}
    }
}
*/

#[component]
pub fn App() -> Element {

    let mut is_modal_open = use_signal(|| false);

    // Styles to create a centered, fixed-size container
    // let container_style = r#"
    //     width: 400px;
    //     height: 800px;
    //     margin: auto; /* Center horizontally */
    //     margin-top: 0vh; /* Some space from the top */
    //     box-shadow: 0px 0px 20px rgba(0,0,0,0.15); /* Nice shadow effect */
    //     border: 1px solid #eee;
    //     overflow-y: auto; /* Allow scrolling within the container if content overflows */
    // "#;

    rsx! {
        // Link the Pico CSS file from your assets directory
        document::Stylesheet { href: asset!("/assets/css/pico.cyan.min.css") }

        // Use the Container component
        Container {
            // Header
            h1 { "Dioxus Pico Components Demo" }
            p { "A demonstration of reusable components using Pico.css and Dioxus." }

            // Grid Layout
            Grid {
                Card {
                    h2 { "Buttons" }
                    p { "A set of versatile buttons." }
                    footer {
                        Button { on_click: move |_| is_modal_open.set(true), "Open Modal" }
                        Button { button_type: ButtonType::Secondary, "Secondary" }
                        Button { button_type: ButtonType::Contrast, "Contrast" }
                    }
                }
                Card {
                    h2 { "Forms" }
                    Input { label: "Your Name", name: "name", placeholder: "John Doe" }
                }
            }

            // Accordion
            Accordion {
                title: "Click to learn more",
                p { "Here is some hidden content inside the accordion. It's great for FAQs or collapsing long sections of text." }
            }
        }

        // The Modal component. It's only rendered when is_modal_open is true.
        Modal {
            is_open: is_modal_open,
            h2 { "This is a modal" }
            p { "You can put any content you want inside a modal. It's great for confirmations, forms, or displaying extra information." }
            footer {
                Button {
                    button_type: ButtonType::Secondary,
                    on_click: move |_| is_modal_open.set(false),
                    "Cancel"
                }
                Button {
                    on_click: move |_| is_modal_open.set(false),
                    "Confirm"
                }
            }
        }
    }


    // rsx! {
    //     // A full-page wrapper to center our container
    //     div {
    //         style: "width: 100vw; height: 800px; background-color: #f0f2f5;",
    //         div {
    //             // The fixed-size container
    //             style: "{container_style}",

    //             // Your app's actual content goes here
    //             h1 { padding: "1rem", "My App" }
    //             p { padding: "1rem", "This content is inside the fixed container." }
    //         }
    //     }
    // }
}
