// this file is present in order to bind the icon.ico file to the neptune-proton.exe on windows.
// It can hopefully go away once this dioxus PR is merged:
//   https://github.com/DioxusLabs/dioxus/pull/3753
fn main() {
    // Only compile resources on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();

        // This path is relative to the build.rs file.
        res.set_icon("icons/icon.ico");

        // Attempt to compile the resource
        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resource: {}", e);
            std::process::exit(1);
        }
    }
}
