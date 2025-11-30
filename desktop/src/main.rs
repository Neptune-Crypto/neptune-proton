use std::io::Cursor;

use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::Config;
use dioxus::desktop::WindowBuilder;
use dioxus::prelude::*;
use image::ImageReader;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");

    launch_without_menubar();
    //    dioxus::launch(App);
}

fn launch_without_menubar() {
    // 1. Define a custom WindowBuilder
    let custom_window = WindowBuilder::new()
        .with_title("neptune-core dashboard")
        .with_window_icon(Some(load_icon()));

    // 2. Define a custom Desktop Config using the custom WindowBuilder
    let desktop_config = Config::new().with_menu(None).with_window(custom_window);

    // 3. Use LaunchBuilder instead of simple launch() and apply the config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(desktop_config)
        .launch(App);
}

fn load_icon() -> Icon {
    // 1. Load the PNG bytes at compile time
    let icon_bytes = include_bytes!("../icons/logo-128x128.png");

    // 2. Decode the image bytes using the `image` crate
    let reader = ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .expect("Failed to guess image format for icon");

    let image = reader.decode().expect("Failed to decode icon image");

    // 3. Convert to the RGBA format required by Icon::from_rgba
    let image_rgba = image.into_rgba8();
    let width = image_rgba.width();
    let height = image_rgba.height();
    let bytes = image_rgba.into_raw();

    // 4. Create the Icon
    Icon::from_rgba(bytes, width, height).expect("Failed to create window icon from RGBA bytes.")
}

#[component]
fn App() -> Element {
    ui::App()
}
