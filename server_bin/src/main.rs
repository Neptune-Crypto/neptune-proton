use dioxus_fullstack::{launch, prelude::*};
use std::net::SocketAddr;

// Import server functions from the api crate.
// The `#[server]` macro in the `api` crate handles making them available.
// No explicit registration is usually needed with current Dioxus versions
// if `api` is a dependency.

// Assuming your main application component `App` is defined in the `web` crate
// and the `Route` enum is also defined there or is accessible.
use api::App; // Or ui::App if that's where your root component is

// If your router's Route enum is defined in the `web` crate (or `ui`):
// use web::Route;
// For this example, let's define a simple Route here or assume it's in `web`.
#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ServerRoute { // Renamed to avoid conflict if web::Route exists
    #[route("/")]
    Index,
}


#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Server listening on http://{}", addr);

    dioxus_fullstack::launch::launch_server(
        addr,
        App, // The root component of your Dioxus application from the `web` (or `ui`) crate
        dioxus_fullstack::router::FullstackRouterConfig::<api::Route>::builder() // Assuming web::Route
            .build().unwrap(),
    ).await;
}