//=============================================================================
// File: src/hooks/use_is_touch_device.rs
//=============================================================================

// Conditionally export the correct module based on the target platform,
// following the established pattern in `qr_scanner.rs`.

#[cfg(target_arch = "wasm32")]
pub use self::wasm32::*;

#[cfg(feature = "dioxus-desktop")]
pub use self::desktop::*;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub use self::mobile::*;

// Fallback for any other platform (like a server) where touch is not applicable.
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios"))
))]
pub use self::fallback::*;


/// # Desktop Implementation
/// Uses `dioxus_desktop::use_window` to evaluate JavaScript in the webview,
/// following the pattern in `qr_scanner.rs`.
#[cfg(feature = "dioxus-desktop")]
mod desktop {
    use dioxus::prelude::*;
    use dioxus_desktop::use_window;
    use serde_json::Value;

    pub fn use_is_touch_device() -> Signal<bool> {
        let mut is_touch_device = use_signal(|| false);
        let window = use_window();

        use_effect(move || {
            spawn(async move {
                let js_code = "return navigator.maxTouchPoints > 0;";
                if let Ok(Ok(Value::Bool(has_touch))) = window.webview.evaluate_script_with_return(js_code).await {
                    is_touch_device.set(has_touch);
                }
            });
        });

        is_touch_device
    }
}

/// # WebAssembly (WASM) Implementation
/// Uses the `web_sys` crate to directly query browser APIs, following the
/// idiomatic pattern demonstrated in `qr_scanner.rs`.
#[cfg(target_arch = "wasm32")]
mod wasm32 {
    use dioxus::prelude::*;

    pub fn use_is_touch_device() -> Signal<bool> {
        let mut is_touch_device = use_signal(|| false);

        use_effect(move || {
            if let Some(window) = web_sys::window() {
                let navigator = window.navigator();
                // `max_touch_points()` is a direct API call that returns an i32.
                let has_touch = navigator.max_touch_points() > 0;
                is_touch_device.set(has_touch);
            }
        });

        is_touch_device
    }
}

/// # Mobile Implementation
/// Assumes that mobile platforms are always touch-enabled.
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile {
    use dioxus::prelude::*;

    pub fn use_is_touch_device() -> Signal<bool> {
        let mut is_touch_device = use_signal(|| false);
        use_effect(move || {
            is_touch_device.set(true);
        });
        is_touch_device
    }
}

/// # Fallback/Server Implementation
/// Assumes that any other platform is not touch-enabled.
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios"))
))]
mod fallback {
    use dioxus::prelude::*;

    pub fn use_is_touch_device() -> Signal<bool> {
        use_signal(|| false)
    }
}

