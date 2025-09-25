// Re-export the public API from the appropriate module
#[cfg(target_arch = "wasm32")]
pub use wasm32::*;

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm32::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm32 {
    use std::time::Duration;
    use tokio::sync::oneshot;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{self, Clipboard, FileReader, HtmlElement, HtmlInputElement, Navigator, Window};

    pub mod interval {
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        use tokio::sync::mpsc;

        pub struct Interval {
            inner: Option<gloo_timers::callback::Interval>,
            rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>,
        }

        impl Interval {
            pub fn new(duration: Duration) -> Self {
                let (tx, rx) = mpsc::unbounded_channel();
                let gloo_interval =
                    gloo_timers::callback::Interval::new(duration.as_millis() as u32, move || {
                        let _ = tx.send(());
                    });

                Self {
                    inner: Some(gloo_interval),
                    rx: Arc::new(Mutex::new(rx)),
                }
            }

            pub async fn tick(&mut self) {
                if let Some(mut rx_lock) = self.rx.try_lock().ok() {
                    let _ = rx_lock.recv().await;
                }
            }
        }

        impl Drop for Interval {
            fn drop(&mut self) {
                if let Some(inner) = self.inner.take() {
                    inner.cancel();
                }
            }
        }
    }

    pub async fn sleep(duration: Duration) {
        gloo_timers::future::sleep(duration).await;
    }

    pub async fn clipboard_set(text: String) -> bool {
        match web_sys::window().map(|win: Window| win.navigator().clipboard()) {
            Some(clipboard) => {
                let promise = clipboard.write_text(&text);
                JsFuture::from(promise).await.is_ok()
            }
            _ => false,
        }
    }

    pub async fn clipboard_get() -> Option<String> {
        let clipboard = web_sys::window()?.navigator().clipboard();
        let promise = clipboard.read_text();
        let js_value = JsFuture::from(promise).await.ok()?;
        js_value.as_string()
    }

    pub async fn read_file(extension: &str) -> Result<Option<String>, String> {
        let (tx, rx) = oneshot::channel();
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let body = document.body().expect("no body");
        let input: HtmlInputElement = document
            .create_element("input")
            .map_err(|e| e.as_string().unwrap_or_default())?
            .dyn_into()
            .map_err(|_| "Failed to cast to HtmlInputElement".to_string())?;
        input.set_type("file");
        input.set_accept(&format!(".{}", extension));
        /*
                input
                    .dyn_ref::<HtmlElement>()
                    .expect("input is not an HtmlElement")
                    .style()
                    .set_property("display", "none")
                    .map_err(|e| e.as_string().unwrap_or_default())?;
        */

        let onchange_closure = Closure::once(move |event: web_sys::Event| {
            let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let onload_closure = Closure::once(move |_: web_sys::ProgressEvent| {
                    let result = reader_clone.result().unwrap();
                    let _ = tx.send(Ok(result.as_string()));
                });
                reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                reader.read_as_text(&file).unwrap();
                onload_closure.forget();
            } else {
                let _ = tx.send(Ok(None));
            }
        });
        input.set_onchange(Some(onchange_closure.as_ref().unchecked_ref()));
        onchange_closure.forget();

        body.append_child(&input)
            .map_err(|e| e.as_string().unwrap_or_default())?;
        input.click();
        body.remove_child(&input)
            .map_err(|e| e.as_string().unwrap_or_default())?;

        rx.await.map_err(|e| e.to_string())?
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod non_wasm32 {
    use dioxus_clipboard::prelude::*;
    use std::time::Duration;

    pub mod interval {
        use tokio::time::{self, Duration, MissedTickBehavior};
        pub struct Interval {
            inner: tokio::time::Interval,
        }
        impl Interval {
            pub fn new(duration: Duration) -> Self {
                let mut interval = time::interval(duration);
                interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
                Self { inner: interval }
            }
            pub async fn tick(&mut self) {
                self.inner.tick().await;
            }
        }
    }

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

    /// Prompts the user to select a file and reads its content as a string.
    pub async fn read_file(extension: &str) -> Result<Option<String>, String> {
        let file_handle = rfd::AsyncFileDialog::new()
            .add_filter("SVG Files", &[extension])
            .pick_file()
            .await;

        if let Some(handle) = file_handle {
            let content = tokio::fs::read_to_string(handle.path())
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }
}
