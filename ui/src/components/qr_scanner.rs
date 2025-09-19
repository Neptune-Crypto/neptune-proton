//=============================================================================
// File: src/components/qr_scanner.rs
//=============================================================================

// Conditionally export the correct module based on the target architecture.
// This allows other parts of the app to simply `use qr_scanner::QrScanner`
// without worrying about the platform.
#[cfg(target_arch = "wasm32")]
pub use self::wasm32::*;

#[cfg(not(target_arch = "wasm32"))]
pub use self::non_wasm32::*;

/// Contains the QR scanner implementation for the WebAssembly target.
#[cfg(target_arch = "wasm32")]
mod wasm32 {
    use dioxus::prelude::*;
    use std::collections::HashMap; // Import HashMap
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        HtmlCanvasElement, HtmlVideoElement, MediaDeviceInfo, MediaDeviceKind, MediaStream,
        MediaStreamConstraints,
    };

    /// A component that provides multiple methods for scanning a QR code.
    /// It will call the `on_scan` event handler when a valid QR code is decoded.
    #[component]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        let mut error_message = use_signal(|| None::<String>);
        let mut is_scanning = use_signal(|| false);

        // Platform-specific state
        let mut video_devices = use_signal(|| Vec::<MediaDeviceInfo>::new());
        let mut selected_device_id = use_signal(String::new);
        let mut video_stream = use_signal::<Option<MediaStream>>(|| None);

        // --- NEW: State for animated QR code reassembly ---
        let mut scanned_parts = use_signal(HashMap::<usize, String>::new);
        let mut total_parts = use_signal(|| 0_usize);

        // --- Web-specific logic to get camera devices (Unchanged) ---
        use_resource(move || async move {
            let window = web_sys::window().expect("no global `window` exists");
            let navigator = window.navigator();
            let media_devices = match navigator.media_devices() {
                Ok(md) => md,
                Err(_) => {
                    error_message.set(Some("Could not access media devices.".to_string()));
                    return;
                }
            };
            match JsFuture::from(
                media_devices
                    .get_user_media_with_constraints(
                        web_sys::MediaStreamConstraints::new().video(&JsValue::from(true)),
                    )
                    .unwrap(),
            )
            .await
            {
                Ok(stream) => {
                    let stream = MediaStream::from(stream);
                    stream
                        .get_tracks()
                        .for_each(&mut |track, _, _| web_sys::MediaStreamTrack::from(track).stop());
                }
                Err(_) => {
                    error_message.set(Some(
                        "Camera permission denied. Please enable camera access.".to_string(),
                    ));
                    return;
                }
            };
            match JsFuture::from(media_devices.enumerate_devices().unwrap()).await {
                Ok(devices) => {
                    let devices_array: js_sys::Array = devices.into();
                    let mut found_devices = Vec::new();
                    for device in devices_array.iter() {
                        let device_info = MediaDeviceInfo::from(device);
                        if device_info.kind() == MediaDeviceKind::Videoinput {
                            found_devices.push(device_info);
                        }
                    }
                    if found_devices.is_empty() {
                        error_message.set(Some("No video cameras found.".to_string()));
                    } else {
                        if let Some(first_device) = found_devices.first() {
                            selected_device_id.set(first_device.device_id());
                        }
                        video_devices.set(found_devices);
                    }
                }
                Err(_) => {
                    error_message.set(Some("Error enumerating devices.".to_string()));
                }
            }
        });

        // --- FIXED: `use_effect` to remove warning ---
        use_effect(move || {
            // The check for `!is_scanning()` has been removed to prevent the read/write warning.
            // The effect's only job is to react to a change in the selected camera.
            if !selected_device_id.read().is_empty() {
                spawn(async move {
                    is_scanning.set(true);
                    match start_video_stream(selected_device_id.read().clone()).await {
                        Ok(stream) => {
                            if let Some(video) = get_element_by_id::<HtmlVideoElement>("qr-video") {
                                video.set_src_object(Some(&stream));
                            }
                            // Clean up previous stream before setting new one.
                            if let Some(old_stream) = video_stream.replace(Some(stream)) {
                                old_stream.get_tracks().for_each(&mut |track, _, _| {
                                    web_sys::MediaStreamTrack::from(track).stop();
                                });
                            }
                            error_message.set(None);
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Failed to start camera: {:?}", e)));
                            is_scanning.set(false);
                        }
                    }
                });
            }
        });

        // --- Cleanup effect (Unchanged) ---
        use_on_unmount(move || {
            if let Some(stream) = video_stream.read().as_ref() {
                stream
                    .get_tracks()
                    .for_each(&mut |track, _, _| web_sys::MediaStreamTrack::from(track).stop());
            }
        });

        /// This function is called repeatedly to scan for QR codes.
        let scan_frame = move || {
            let video_element: Option<HtmlVideoElement> = get_element_by_id("qr-video");
            let canvas_element: Option<HtmlCanvasElement> = get_element_by_id("qr-canvas");

            if let (Some(video), Some(canvas)) = (video_element, canvas_element) {
                let (width, height) = (video.video_width(), video.video_height());
                if width == 0 || height == 0 { return; }

                canvas.set_width(width);
                canvas.set_height(height);

                let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                ctx.draw_image_with_html_video_element(&video, 0.0, 0.0).unwrap();
                let image_data_result = ctx.get_image_data(0.0, 0.0, width as f64, height as f64);

                if let Ok(image_data) = image_data_result {
                    let luma_data: Vec<u8> = image_data.data().0.chunks_exact(4).map(|p| (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) as u8).collect();
                    if let Some(image) = image::GrayImage::from_raw(width, height, luma_data) {
                        let mut img = rqrr::PreparedImage::prepare(image);
                        let grids = img.detect_grids();

                        if let Some(grid) = grids.first() {
                            if let Ok((_meta, content)) = grid.decode() {
                                if content.is_empty() { return; }

                                // --- SURGICAL FIX: Separated logic for static vs. animated ---

                                // Case 1: Animated Frame Detected
                                if content.starts_with('P') && content.chars().filter(|&c| c == '/').count() == 2 {
                                    let parts: Vec<&str> = content.splitn(3, '/').collect();
                                    if parts.len() == 3 {
                                        if let (Ok(part_num), Ok(total)) = (parts[0][1..].parse::<usize>(), parts[1].parse::<usize>()) {
                                            if *total_parts.read() == 0 {
                                                total_parts.set(total);
                                            }
                                            scanned_parts.write().insert(part_num, parts[2].to_string());

                                            if scanned_parts.read().len() == total {
                                                let mut result = String::new();
                                                for i in 1..=total {
                                                    if let Some(chunk) = scanned_parts.read().get(&i) {
                                                        result.push_str(chunk);
                                                    } else {
                                                        error_message.set(Some(format!("Reassembly failed: Missing part {}", i)));
                                                        return;
                                                    }
                                                }
                                                is_scanning.set(false);
                                                on_scan.call(result);
                                                on_close.call(());
                                            }
                                        }
                                    }
                                }
                                // Case 2: Static QR Code (YOUR ORIGINAL LOGIC, REORDERED TO FIX PANIC)
                                else {
                                    is_scanning.set(false); // Update state BEFORE closing
                                    on_scan.call(content);
                                    on_close.call(());     // Close component as the FINAL action
                                }
                            }
                        }
                    }
                }
            }
        };

        // --- UI Rendering (with addition of progress indicator) ---
        let error_display: Option<Element> = if let Some(err) = error_message() {
            Some(rsx! { p { style: "color: var(--pico-color-red-500);", "{err}" } })
        } else {
            None
        };

        let progress_indicator: Option<Element> = if is_scanning() {
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

        let scanner_display: Option<Element> = if is_scanning() {
            Some(rsx! {
                div {
                    style: "position: relative; width: 100%; max-width: 400px; margin: auto; border-radius: var(--pico-border-radius); overflow: hidden; border: 1px solid var(--pico-form-element-border-color);",
                    video {
                        id: "qr-video",
                        autoplay: true,
                        playsinline: true,
                        oncanplay: move |_| {
                            use gloo_timers::callback::Interval;
                            let scan_interval = Interval::new(250, scan_frame);
                            scan_interval.forget();
                        },
                    }
                    canvas { id: "qr-canvas", style: "display: none;" }
                }
            })
        } else {
            None
        };

        let device_selector: Option<Element> = if !video_devices.read().is_empty() && !is_scanning() {
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
                        class: "secondary outline",
                        onclick: move |_| { error_message.set(Some("File upload not yet implemented.".to_string())); },
                        "Upload Image"
                    }
                    button {
                        class: "secondary outline",
                        onclick: move |_| { error_message.set(Some("Paste from clipboard not yet implemented.".to_string())); },
                        "Paste from Clipboard"
                    }
                    hr {}
                    button {
                        class: "secondary",
                        onclick: move |_| { on_close.call(()); },
                        "Cancel"
                    }
                }
            }
        }
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
        let mut error_message = use_signal(|| None::<String>);
        let error_display: Option<Element> = if let Some(err) = error_message() {
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
                        class: "secondary outline",
                        onclick: move |_| { error_message.set(Some("File upload not yet implemented.".to_string())); },
                        "Upload Image"
                    }
                    button {
                        class: "secondary outline",
                        onclick: move |_| { error_message.set(Some("Paste from clipboard not yet implemented.".to_string())); },
                        "Paste from Clipboard"
                    }
                    hr {}
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

