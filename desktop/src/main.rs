use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");

    launch_without_menubar();
//    dioxus::launch(App);
}

fn launch_without_menubar() {

    // 1. Define a custom WindowBuilder
    let custom_window = WindowBuilder::new()
        .with_title("neptune-core dashboard");

    // 2. Define a custom Desktop Config using the custom WindowBuilder
    let desktop_config = Config::new()
        .with_menu(None)
        .with_window(custom_window);

    // 3. Use LaunchBuilder instead of simple launch() and apply the config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(desktop_config)
        .launch(App);    
}

#[component]
fn App() -> Element {
    ui::App()
}
