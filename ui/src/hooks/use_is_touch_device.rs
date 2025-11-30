//=============================================================================
// File: src/hooks/use_is_touch_device.rs
//=============================================================================

// Conditionally export the correct module based on the target platform,
// following the established pattern in `qr_scanner.rs`.

// Fallback for any other platform (like a server) where touch is not applicable.
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios"))
))]
pub use self::fallback::*;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use self::mobile::*;
#[cfg(target_arch = "wasm32")]
pub use self::web_desktop::*;

/// # Unified Desktop & Web (WASM) Implementation
/// Uses the `use_document` hook to get a handle to the document, which
/// provides a cross-platform `.eval()` method.
#[cfg(any(feature = "dioxus-desktop", target_arch = "wasm32"))]
mod web_desktop {
    use dioxus::prelude::*;

    pub fn use_is_touch_device() -> Signal<bool> {
        let mut is_touch_device = use_signal(|| false);

        use_effect(move || {
            // We need to handle the case where the document might not be available yet
            spawn(async move {
                let js_code = r#"
                    return navigator.maxTouchPoints > 0 || 'ontouchstart' in window;
                "#;

                // Call eval on the document handle. This is the correct API.
                if let Ok(result) = document::eval(js_code).await {
                    // The result is a serde_json::Value, so the rest of the logic is the same.
                    if let Ok(has_touch) = serde_json::from_value::<bool>(result) {
                        is_touch_device.set(has_touch);
                    }
                }
            });
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
