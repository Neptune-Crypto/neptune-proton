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

    pub mod interval {

        pub struct Interval {
            inner: Option<gloo_timers::callback::Interval>,
            rx: Arc<Mutex<mpsc::Receiver<()>>>,
        }

        impl Interval {
            pub fn new(duration: Duration) -> Self {
                let (tx, rx) = mpsc::unbounded_channel();

                let tx_clone = tx.clone();
                let gloo_interval =  gloo_timers::callback::Interval::new(duration.as_millis() as u32, move || {
                    let _ = tx_clone.send(());
                });

                Self {
                    inner: Some(gloo_interval),
                    rx: Arc::new(Mutex::new(rx)),
                }
            }

            pub async fn tick(&mut self) {
                let mut rx_lock = self.rx.lock().await;
                let _ = rx_lock.recv().await;
            }
        }

        impl Drop for Interval {
            fn drop(&mut self) {
                if let Some(inner) = self.inner.take() {
                    drop(inner);
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

    pub mod interval {
        use tokio::time::{self, Duration};

        pub struct Interval {
            inner: tokio::time::Interval,
        }

        impl Interval {
            pub fn new(duration: Duration) -> Self {
                let mut interval = time::interval(duration);
                interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
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
}
