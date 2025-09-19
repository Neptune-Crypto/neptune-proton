use dioxus::prelude::*;

fn main() {

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();    

    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    ui::App()
}
