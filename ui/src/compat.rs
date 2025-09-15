// Re-export the public API from the appropriate module
#[cfg(target_arch = "wasm32")]
pub use wasm32::*;

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm32::*;

use dioxus_clipboard::prelude::*;

pub async fn clipboard_set(text: String) -> bool {
    let mut clipboard = use_clipboard();
    clipboard.set(text).is_ok()
}

pub async fn clipboard_get() -> Option<String> {
    let mut clipboard = use_clipboard();
    clipboard.get().ok()
}

#[cfg(target_arch = "wasm32")]
pub mod wasm32 {
    use std::time::Duration;

    pub async fn sleep(duration: Duration) {
        gloo_timers::future::sleep(duration).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod non_wasm32 {
    use std::time::Duration;

    pub async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
