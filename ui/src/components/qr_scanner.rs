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

        // --- Web-specific logic to get camera devices ---
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

            // Request permission to get device labels
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

        // --- Web-specific effect to start the video stream ---
        use_effect(move || {
            if !selected_device_id.read().is_empty() && !is_scanning() {
                spawn(async move {
                    is_scanning.set(true);
                    match start_video_stream(selected_device_id.read().clone()).await {
                        Ok(stream) => {
                            // Attach the stream to the video element immediately.
                            if let Some(video) = get_element_by_id::<HtmlVideoElement>("qr-video") {
                                video.set_src_object(Some(&stream));
                            }
                            video_stream.set(Some(stream)); // Save for cleanup
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

        // --- Cleanup effect using the idiomatic `use_on_unmount` hook ---
        use_on_unmount(move || {
            if let Some(stream) = video_stream.take() {
                stream
                    .get_tracks()
                    .for_each(&mut |track, _, _| web_sys::MediaStreamTrack::from(track).stop());
            }
        });

        /// This function is called repeatedly to scan for QR codes.
        let scan_frame = move || {
            let video_element: Option<HtmlVideoElement> =
                get_element_by_id("qr-video");
            let canvas_element: Option<HtmlCanvasElement> =
                get_element_by_id("qr-canvas");

            if let (Some(video), Some(canvas)) = (video_element, canvas_element) {
                let width = video.video_width();
                let height = video.video_height();

                if width == 0 || height == 0 {
                    return;
                } // Video not ready

                canvas.set_width(width);
                canvas.set_height(height);

                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

                ctx.draw_image_with_html_video_element(&video, 0.0, 0.0)
                    .unwrap();

                // Get image data from canvas
                let image_data_result =
                    ctx.get_image_data(0.0, 0.0, width as f64, height as f64);

                if let Ok(image_data) = image_data_result {
                    // Convert RGBA to grayscale for rqrr
                    let mut luma_data: Vec<u8> = image_data
                        .data()
                        .0
                        .chunks_exact(4)
                        .map(|pixel| {
                            // Using a standard luma conversion formula
                            (pixel[0] as f32 * 0.299
                                + pixel[1] as f32 * 0.587
                                + pixel[2] as f32 * 0.114) as u8
                        })
                        .collect();

                    // --- NEW: Contrast Enhancement Logic ---
                    // Find the min and max brightness values in the current frame.
                    let mut min_luma = 255u8;
                    let mut max_luma = 0u8;
                    for &pixel in &luma_data {
                        if pixel < min_luma { min_luma = pixel; }
                        if pixel > max_luma { max_luma = pixel; }
                    }

                    // If the contrast is low (i.e., the range of brightness is small),
                    // stretch the contrast to the full 0-255 range.
                    let luma_range = max_luma.saturating_sub(min_luma);
                    if luma_range > 0 && luma_range < 200 { // Heuristic threshold
                        let scale = 255.0 / luma_range as f32;
                        for pixel in &mut luma_data {
                            // Apply the formula: new_pixel = (old_pixel - min) * scale
                            let new_val = ((*pixel as f32 - min_luma as f32) * scale).round() as u8;
                            *pixel = new_val;
                        }
                    }
                    // --- End of new logic ---


                    // Use rqrr to decode the image
                    let mut img = rqrr::PreparedImage::prepare(
                        image::GrayImage::from_raw(width, height, luma_data).unwrap(),
                    );
                    let grids = img.detect_grids();

                    if let Some(grid) = grids.first() {
                        if let Ok((_meta, content)) = grid.decode() {
                            if !content.is_empty() {
                                on_scan.call(content);
                                is_scanning.set(false);
                                on_close.call(());
                            }
                        }
                    }
                }
            }
        };

        // --- Pre-build conditional UI elements to simplify the main rsx! macro ---

        let error_display: Option<Element> = if let Some(err) = error_message() {
            Some(rsx! { p { style: "color: var(--pico-color-red-500);", "{err}" } })
        } else {
            None // Render nothing if no error
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
                            scan_interval.forget(); // Leak to keep it running
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
                {scanner_display}
                {device_selector}

                // --- Action Buttons ---
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

    /// A private helper function to get a DOM element by its ID. This function
    /// only exists in the wasm32 module as it is a web-only concept.
    fn get_element_by_id<T: wasm_bindgen::JsCast>(id: &str) -> Option<T> {
        web_sys::window()?
            .document()?
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<T>().ok())
    }

    /// Helper function to start the video stream on web.
    async fn start_video_stream(device_id: String) -> Result<MediaStream, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()?;

        let mut constraints = MediaStreamConstraints::new();
        let mut video_constraints = web_sys::MediaTrackConstraints::new();

        if !device_id.is_empty() {
            video_constraints.device_id(&device_id.into());
        }

        // --- FIX: Correctly build the nested JS object for advanced constraints ---
        let advanced_constraint = js_sys::Object::new();

        // Build width object: { ideal: 4096 }
        let width_constraint = js_sys::Object::new();
        js_sys::Reflect::set(&width_constraint, &"ideal".into(), &4096.into())?;

        // Build height object: { ideal: 2160 }
        let height_constraint = js_sys::Object::new();
        js_sys::Reflect::set(&height_constraint, &"ideal".into(), &2160.into())?;

        // Set properties on the main advanced constraint object
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

    /// A component that provides multiple methods for scanning a QR code.
    /// On desktop, this is currently a placeholder.
    #[component]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        // Mark `on_scan` as used to prevent the compiler from renaming it with a
        // leading underscore, which would break the component's public API on
        // the `send` screen.
        let _ = on_scan;

        let mut error_message = use_signal(|| None::<String>);

        // By building the optional error message element outside the main rsx! block,
        // we simplify the code that the macro needs to parse.
        let error_display: Option<Element> = if let Some(err) = error_message() {
            Some(rsx! { p { style: "color: var(--pico-color-red-500);", "{err}" } })
        } else {
            None
        };

        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem; max-width: 500px; margin: auto;",
                h3 { "Scan QR Code" }

                // Render the pre-built error element here
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

