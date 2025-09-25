use dioxus::prelude::*;

fn main() {
    dioxus::logger::init(dioxus::logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    ui::App()
}
