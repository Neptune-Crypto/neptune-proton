//=============================================================================
// File: src/components/qr_scanner.rs
//=============================================================================

// Conditionally export the correct module based on the target architecture.
#[cfg(target_arch = "wasm32")]
pub use self::wasm32::*;

#[cfg(not(target_arch = "wasm32"))]
pub use self::non_wasm32::*;

/// Contains the QR scanner implementation for the WebAssembly target.
#[cfg(target_arch = "wasm32")]
mod wasm32 {
    use dioxus::prelude::*;
    use std::collections::HashMap;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        HtmlCanvasElement, HtmlVideoElement, MediaDeviceInfo, MediaDeviceKind, MediaStream,
        MediaStreamConstraints,
    };

    // **FIX**: This guard struct ensures the camera stream is stopped when this
    // object goes out of scope, whether the task completes or is cancelled.
    struct StreamGuard(MediaStream);

    impl Drop for StreamGuard {
        fn drop(&mut self) {
            self.0.get_tracks().for_each(&mut |track, _, _| {
                web_sys::MediaStreamTrack::from(track).stop();
            });
        }
    }

    #[component]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        let mut error_message = use_signal(|| None::<String>);
        let mut is_scanning = use_signal(|| false);
        let mut video_devices = use_signal(Vec::<MediaDeviceInfo>::new);
        let mut selected_device_id = use_signal(String::new);
        // **FIX**: The `video_stream` signal is no longer needed; the guard handles cleanup.
        let mut scanned_parts = use_signal(HashMap::<usize, String>::new);
        let mut total_parts = use_signal(|| 0_usize);

        use_hook(move || {
            spawn(async move {
                match enumerate_devices().await {
                    Ok(devices) => {
                        if let Some(first_device) = devices.first() {
                            selected_device_id.set(first_device.device_id());
                        }
                        video_devices.set(devices);
                    }
                    Err(e) => {
                        let err_str = e.as_string().unwrap_or_else(|| "Unknown error".into());
                        error_message.set(Some(format!("Could not list cameras: {}", err_str)));
                    }
                }
            });
        });

        use_memo(move || {
            let device_id = selected_device_id.read().clone();

            if device_id.is_empty() {
                return None;
            }

            Some(spawn(async move {
                is_scanning.set(true);
                let stream = match start_video_stream(device_id).await {
                    Ok(s) => {
                        if let Some(video) = get_element_by_id::<HtmlVideoElement>("qr-video") {
                            video.set_src_object(Some(&s));
                        }
                        error_message.set(None);
                        s
                    }
                    Err(e) => {
                        let err_str = e.as_string().unwrap_or_else(|| "Unknown error".into());
                        error_message.set(Some(format!("Failed to start camera: {}", err_str)));
                        is_scanning.set(false);
                        return;
                    }
                };

                // **FIX**: The stream is placed in the guard. When `_stream_guard` is
                // dropped at the end of the task, the camera will be stopped.
                let _stream_guard = StreamGuard(stream);

                loop {
                    let video_element: Option<HtmlVideoElement> = get_element_by_id("qr-video");
                    let canvas_element: Option<HtmlCanvasElement> = get_element_by_id("qr-canvas");

                    if let (Some(video), Some(canvas)) = (video_element, canvas_element) {
                        let (width, height) = (video.video_width(), video.video_height());
                        if width > 0 && height > 0 {
                            canvas.set_width(width);
                            canvas.set_height(height);

                            let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                            ctx.draw_image_with_html_video_element(&video, 0.0, 0.0).unwrap();

                            if let Ok(image_data) = ctx.get_image_data(0.0, 0.0, width as f64, height as f64) {
                                let luma_data: Vec<u8> = image_data.data().0.chunks_exact(4).map(|p| (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) as u8).collect();

                                if let Some(image) = image::GrayImage::from_raw(width, height, luma_data) {
                                    let mut img = rqrr::PreparedImage::prepare(image);
                                    if let Some(grid) = img.detect_grids().first() {
                                        if let Ok((_meta, content)) = grid.decode() {
                                            if content.is_empty() { continue; }

                                            let mut scan_is_complete = false;
                                            if !content.starts_with('P') || content.chars().filter(|&c| c == '/').count() != 2 {
                                                on_scan.call(content);
                                                scan_is_complete = true;
                                            } else {
                                                let parts: Vec<&str> = content.splitn(3, '/').collect();
                                                if parts.len() == 3 {
                                                    if let (Ok(part_num), Ok(total)) = (parts[0][1..].parse::<usize>(), parts[1].parse::<usize>()) {
                                                        if *total_parts.read() == 0 { total_parts.set(total); }
                                                        scanned_parts.write().entry(part_num).or_insert_with(|| parts[2].to_string());
                                                        if scanned_parts.read().len() == total {
                                                            let mut result = String::new();
                                                            let mut reassembly_ok = true;
                                                            for i in 1..=total {
                                                                if let Some(chunk) = scanned_parts.read().get(&i) {
                                                                    result.push_str(chunk);
                                                                } else {
                                                                    error_message.set(Some(format!("Reassembly failed: Missing part {}", i)));
                                                                    reassembly_ok = false;
                                                                    break;
                                                                }
                                                            }
                                                            if reassembly_ok {
                                                                on_scan.call(result);
                                                                scan_is_complete = true;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if scan_is_complete { break; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                }

                // **FIX**: The explicit cleanup block is no longer needed.
                is_scanning.set(false);
                on_close.call(());
            }))
        });

        // --- UI Rendering ---
        let error_display: Option<Element> = if let Some(err) = error_message.read().as_ref() {
            Some(rsx! { p { style: "color: var(--pico-color-red-500);", "{err}" } })
        } else {
            None
        };

        let progress_indicator: Option<Element> = if *is_scanning.read() {
            Some( if *total_parts.read() > 0 {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                        label { "Scan Progress: {scanned_parts.read().len()} of {total_parts.read()}" },
                        progress { max: "{total_parts.read()}", value: "{scanned_parts.read().len()}" }
                    }
                }
            } else {
                rsx! {
                     div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                        label { "Aim camera at QR code..." },
                        progress {}
                    }
                }
            })
        } else {
            None
        };

        let scanner_display: Option<Element> = if *is_scanning.read() {
            Some(rsx! {
                div {
                    style: "position: relative; width: 100%; max-width: 400px; margin: auto; border-radius: var(--pico-border-radius); overflow: hidden; border: 1px solid var(--pico-form-element-border-color);",
                    video { id: "qr-video", autoplay: true, playsinline: true }
                    canvas { id: "qr-canvas", style: "display: none;" }
                }
            })
        } else {
            None
        };

        let device_selector: Option<Element> = if !video_devices.read().is_empty() {
            let devices = video_devices.read();
            let options = devices.iter().enumerate().map(|(i, device)| {
                let device_id = device.device_id();
                let is_selected = *selected_device_id.read() == device_id;
                let mut label = device.label();
                if label.is_empty() {
                    label = format!("Camera {}", i + 1);
                }
                rsx! {
                    option {
                        key: "{device_id}",
                        value: "{device_id}",
                        selected: is_selected,
                        "{label}"
                    }
                }
            });
            Some(rsx! {
               select {
                   aria_label: "Select Camera",
                   onchange: move |event| {
                       selected_device_id.set(event.value());
                   },
                   {options}
               }
           })
        } else {
            None
        };

        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem; max-width: 500px; margin: auto;",
                h3 { "Scan QR Code" }
                {error_display}
                {progress_indicator}
                {scanner_display}
                {device_selector}
                div {
                    style: "display: flex; flex-direction: column; gap: 0.75rem; margin-top: 1rem;",
                    button {
                        class: "secondary",
                        onclick: move |_| {
                            on_close.call(());
                        },
                        "Cancel"
                    }
                }
            }
        }
    }

    async fn enumerate_devices() -> Result<Vec<MediaDeviceInfo>, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()?;
        let stream = JsFuture::from(media_devices.get_user_media_with_constraints(MediaStreamConstraints::new().video(&true.into()))?).await?;
        MediaStream::from(stream).get_tracks().for_each(&mut |track, _, _| { web_sys::MediaStreamTrack::from(track).stop(); });
        let devices = JsFuture::from(media_devices.enumerate_devices()?).await?;
        Ok(js_sys::Array::from(&devices).iter().map(MediaDeviceInfo::from).filter(|d| d.kind() == MediaDeviceKind::Videoinput).collect())
    }

    fn get_element_by_id<T: wasm_bindgen::JsCast>(id: &str) -> Option<T> {
        web_sys::window()?.document()?.get_element_by_id(id).and_then(|element| element.dyn_into::<T>().ok())
    }

    async fn start_video_stream(device_id: String) -> Result<MediaStream, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()?;
        let mut constraints = MediaStreamConstraints::new();
        let mut video_constraints = web_sys::MediaTrackConstraints::new();
        if !device_id.is_empty() {
            video_constraints.device_id(&device_id.into());
        }
        let advanced_constraint = js_sys::Object::new();
        let width_constraint = js_sys::Object::new();
        js_sys::Reflect::set(&width_constraint, &"ideal".into(), &4096.into())?;
        let height_constraint = js_sys::Object::new();
        js_sys::Reflect::set(&height_constraint, &"ideal".into(), &2160.into())?;
        js_sys::Reflect::set(&advanced_constraint, &"width".into(), &width_constraint)?;
        js_sys::Reflect::set(&advanced_constraint, &"height".into(), &height_constraint)?;
        video_constraints.advanced(&js_sys::Array::of1(&advanced_constraint));
        constraints.video(&video_constraints.into());
        constraints.audio(&JsValue::from(false));
        let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = JsFuture::from(stream_promise).await?;
        Ok(MediaStream::from(stream))
    }
}

/// Contains the QR scanner implementation for native (non-WASM) targets.
#[cfg(not(target_arch = "wasm32"))]
mod non_wasm32 {
    use dioxus::prelude::*;
    #[component]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        let _ = on_scan;
        let error_message = use_signal(|| None::<String>);

        let error_display: Option<Element> = if let Some(err) = error_message.read().as_ref() {
            Some(rsx! { p { style: "color: var(--pico-color-red-500);", "{err}" } })
        } else {
            None
        };
        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem; max-width: 500px; margin: auto;",
                h3 { "Scan QR Code" }
                {error_display}
                p { "Desktop camera access is not yet implemented." }
                div {
                    style: "display: flex; flex-direction: column; gap: 0.75rem; margin-top: 1rem;",
                    button {
                        class: "secondary",
                        onclick: move |_| { on_close.call(()); },
                        "Cancel"
                    }
                }
            }
        }
    }
}