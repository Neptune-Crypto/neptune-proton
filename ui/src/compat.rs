// Re-export the public API from the appropriate module
#[cfg(target_arch = "wasm32")]
pub use wasm32::*;

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm32::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm32 {
    use std::time::Duration;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{self, Navigator, Window, Clipboard};

    pub async fn sleep(duration: Duration) {
        gloo_timers::future::sleep(duration).await;
    }

    pub async fn clipboard_set(text: String) -> bool {
        match web_sys::window().map(|win: Window| win.navigator().clipboard()) {
            Some(clipboard) => {
                let promise = clipboard.write_text(&text);
                JsFuture::from(promise).await.is_ok()
            }
            None => false,
        }
    }

    pub async fn clipboard_get() -> Option<String> {
        let clipboard = web_sys::window()?.navigator().clipboard();
        let promise = clipboard.read_text();
        let js_value = wasm_bindgen_futures::JsFuture::from(promise).await.ok()?;

        js_value.as_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod non_wasm32 {
    use std::time::Duration;
    use dioxus::prelude::*;
    use dioxus_clipboard::prelude::*;

    pub async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    pub async fn clipboard_set(text: String) -> bool {
        let mut clipboard = use_clipboard();
        clipboard.set(text).is_ok()
    }

    pub async fn clipboard_get() -> Option<String> {
        let mut clipboard = use_clipboard();
        clipboard.get().ok()
    }
}
