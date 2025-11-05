//=============================================================================
// File: src/components/qr_scanner.rs
//=============================================================================

// Conditionally export the correct module based on the target platform.
#[cfg(target_arch = "wasm32")]
pub use self::wasm32::*;

#[cfg(not(target_arch = "wasm32"))]
pub use self::server::*;

#[cfg(feature = "dioxus-desktop")]
pub use self::desktop::*;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub use self::mobile::*;

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

                let _stream_guard = StreamGuard(stream);

                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    let video_element: Option<HtmlVideoElement> = get_element_by_id("qr-video");
                    let canvas_element: Option<HtmlCanvasElement> = get_element_by_id("qr-canvas");

                    if let (Some(video), Some(canvas)) = (video_element, canvas_element) {
                        let (width, height) = (video.video_width(), video.video_height());
                        if width > 0 && height > 0 {
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

                            if let Ok(image_data) =
                                ctx.get_image_data(0.0, 0.0, width as f64, height as f64)
                            {
                                let luma_data: Vec<u8> = image_data
                                    .data()
                                    .0
                                    .chunks_exact(4)
                                    .map(|p| {
                                        (p[0] as f32 * 0.299
                                            + p[1] as f32 * 0.587
                                            + p[2] as f32 * 0.114) as u8
                                    })
                                    .collect();

                                if let Some(image) =
                                    image::GrayImage::from_raw(width, height, luma_data)
                                {
                                    let mut img = rqrr::PreparedImage::prepare(image);
                                    if let Some(grid) = img.detect_grids().first() {
                                        if let Ok((_meta, content)) = grid.decode() {
                                            if content.is_empty() {
                                                continue;
                                            }

                                            let mut scan_is_complete = false;
                                            if !content.starts_with('P')
                                                || content.chars().filter(|&c| c == '/').count()
                                                    != 2
                                            {
                                                on_scan.call(content);
                                                scan_is_complete = true;
                                            } else {
                                                let parts: Vec<&str> =
                                                    content.splitn(3, '/').collect();
                                                if parts.len() == 3 {
                                                    if let (Ok(part_num), Ok(total)) = (
                                                        parts[0][1..].parse::<usize>(),
                                                        parts[1].parse::<usize>(),
                                                    ) {
                                                        if *total_parts.read() == 0 {
                                                            total_parts.set(total);
                                                        }
                                                        scanned_parts.write().entry(part_num).or_insert_with(|| parts[2].to_string());
                                                        if scanned_parts.read().len() == total {
                                                            let mut result = String::new();
                                                            let mut reassembly_ok = true;
                                                            for i in 1..=total {
                                                                if let Some(chunk) =
                                                                    scanned_parts.read().get(&i)
                                                                {
                                                                    result.push_str(chunk);
                                                                } else {
                                                                    error_message.set(Some(
                                                                        format!(
                                                                            "Reassembly failed: Missing part {}",
                                                                            i
                                                                        ),
                                                                    ));
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
                                            if scan_is_complete {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                is_scanning.set(false);
                on_close.call(());
            }))
        });

        let error_display = error_message.read().as_ref().map(|err| rsx! {
                p {
                    style: "color: var(--pico-color-red-500);",
                    "{err}"
                }
            });
        let progress_indicator = if *is_scanning.read() {
            Some(if *total_parts.read() > 0 {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                        label {

                            "Scan Progress: {scanned_parts.read().len()} of {total_parts.read()}"
                        }
                        progress {
                            max: "{total_parts.read()}",
                            value: "{scanned_parts.read().len()}",
                        }
                    }
                }
            } else {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                        label {

                            "Aim camera at QR code..."
                        }
                        progress {

                        }
                    }
                }
            })
        } else {
            None
        };
        let scanner_display = if *is_scanning.read() {
            Some(rsx! {
                div {
                    style: "position: relative; width: 100%; max-width: 400px; margin: auto; border-radius: var(--pico-border-radius); overflow: hidden; border: 1px solid var(--pico-form-element-border-color);",
                    video {
                        id: "qr-video",
                        autoplay: true,
                        playsinline: true,
                    }
                    canvas {
                        id: "qr-canvas",
                        style: "display: none;",
                    }
                }
            })
        } else {
            None
        };
        let device_selector = if !video_devices.read().is_empty() {
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
                h3 {

                    "Scan QR Code"
                }
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

        let constraints = MediaStreamConstraints::new();
        constraints.set_video(&true.into());
        let stream = JsFuture::from(
            media_devices
                .get_user_media_with_constraints(&constraints)?,
        )
        .await?;

        MediaStream::from(stream)
            .get_tracks()
            .for_each(&mut |track, _, _| {
                web_sys::MediaStreamTrack::from(track).stop();
            });
        let devices = JsFuture::from(media_devices.enumerate_devices()?).await?;
        Ok(js_sys::Array::from(&devices)
            .iter()
            .map(MediaDeviceInfo::from)
            .filter(|d| d.kind() == MediaDeviceKind::Videoinput)
            .collect())
    }

    fn get_element_by_id<T: wasm_bindgen::JsCast>(id: &str) -> Option<T> {
        web_sys::window()?
            .document()?
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<T>().ok())
    }

    async fn start_video_stream(device_id: String) -> Result<MediaStream, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let media_devices = navigator.media_devices()?;
        let constraints = MediaStreamConstraints::new();
        let video_constraints = web_sys::MediaTrackConstraints::new();

        if !device_id.is_empty() {
            video_constraints.set_device_id(&device_id.into());
        }

        let advanced_constraint = js_sys::Object::new();
        let width_constraint = js_sys::Object::new();
        js_sys::Reflect::set(&width_constraint, &"ideal".into(), &4096.into())?;
        let height_constraint = js_sys::Object::new();
        js_sys::Reflect::set(&height_constraint, &"ideal".into(), &2160.into())?;
        js_sys::Reflect::set(
            &advanced_constraint,
            &"width".into(),
            &width_constraint,
        )?;
        js_sys::Reflect::set(
            &advanced_constraint,
            &"height".into(),
            &height_constraint,
        )?;

        video_constraints.set_advanced(&js_sys::Array::of1(&advanced_constraint));
        constraints.set_video(&video_constraints.into());
        constraints.set_audio(&JsValue::from(false));

        let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = JsFuture::from(stream_promise).await?;
        Ok(MediaStream::from(stream))
    }
}

/// Contains the QR scanner implementation for desktop platforms.
#[cfg(feature = "dioxus-desktop")]
mod desktop {
    // This code block will only be compiled if the `desktop` feature is enabled.
    use base64::engine::{general_purpose::STANDARD as BASE64_STANDARD, Engine};
    use dioxus::prelude::*;
    use futures::StreamExt;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use dioxus_desktop::use_window;

    #[derive(Debug)]
    enum Message {
        Frame((Vec<u8>, u32, u32)),
        Content(String),
        Error(String),
        Done,
    }

    fn start_camera_thread(
        tx: tokio::sync::mpsc::UnboundedSender<Message>,
        stop_signal: Arc<Mutex<bool>>,
    ) {
        std::thread::spawn(move || {
            let cam_result = camera_capture::create(0).and_then(|builder| {
                builder
                    .fps(15.0)
                    .map_err(|e| std::io::Error::other(format!("{:?}", e)))
                    .and_then(|b| b.start().map_err(|e| std::io::Error::other(format!("{:?}", e))))
            });

            let cam = match cam_result {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Message::Error(e.to_string()));
                    return;
                }
            };

            let mut last_scan_time = Instant::now();
            let scan_interval = Duration::from_millis(100);

            for frame in cam {
                if *stop_signal.lock().unwrap() {
                    break;
                }

                let (width, height) = (frame.width(), frame.height());
                let raw_pixels: Vec<u8> = frame.to_vec();

                // Always send the frame for display to ensure smooth video.
                let _ = tx.send(Message::Frame((raw_pixels.clone(), width, height)));

                // Only perform expensive QR detection on a timer.
                if last_scan_time.elapsed() >= scan_interval {
                    last_scan_time = Instant::now();
                    let luma_data: Vec<u8> = raw_pixels
                        .chunks_exact(3)
                        .map(|p| {
                            ((p[0] as f32 * 0.299)
                                + (p[1] as f32 * 0.587)
                                + (p[2] as f32 * 0.114)) as u8
                        })
                        .collect();

                    if let Some(image) = image::GrayImage::from_raw(width, height, luma_data) {
                        let mut img = rqrr::PreparedImage::prepare(image);
                        if let Some(grid) = img.detect_grids().first() {
                            if let Ok((_meta, content)) = grid.decode() {
                                if !content.is_empty() {
                                    let _ = tx.send(Message::Content(content));
                                }
                            }
                        }
                    }
                }
            }
            let _ = tx.send(Message::Done);
        });
    }

    #[component]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        let mut error_message = use_signal(|| None::<String>);
        let mut is_scanning = use_signal(|| false);
        let mut scanned_parts = use_signal(HashMap::<usize, String>::new);
        let mut total_parts = use_signal(|| 0_usize);
        let stop_signal = use_hook(|| Arc::new(Mutex::new(false)));
        let stop_signal2 = stop_signal.clone();
        let stop_signal3 = stop_signal.clone();
        let mut frame_dims = use_signal(|| (0u32, 0u32));

        let window = use_window();

        // This coroutine runs on the main thread and handles all state updates.
        let state_updater = use_coroutine(move |mut rx: futures_channel::mpsc::UnboundedReceiver<Message>| {
            // Clone the Arcs/handles needed for the async block inside.
            let window = window.clone();
            let on_scan = on_scan;
            let on_close = on_close;
            let stop_signal = stop_signal.clone();

            async move {
                while let Some(msg) = rx.next().await {
                    match msg {
                        Message::Error(e) => {
                            error_message.set(Some(e));
                            is_scanning.set(false);
                        }
                        Message::Frame((rgb_pixels, width, height)) => {
                            if frame_dims.read().0 != width || frame_dims.read().1 != height {
                                frame_dims.set((width, height));
                            }
                            let mut rgba_pixels =
                                Vec::with_capacity((width * height * 4) as usize);
                            for chunk in rgb_pixels.chunks_exact(3) {
                                rgba_pixels.push(chunk[0]); // R
                                rgba_pixels.push(chunk[1]); // G
                                rgba_pixels.push(chunk[2]); // B
                                rgba_pixels.push(255); // A
                            }
                            let base64_frame = BASE64_STANDARD.encode(&rgba_pixels);

                            let js_code = format!(
                                r#"
                                try {{
                                    const canvas = document.getElementById('qr-canvas');
                                    if (canvas) {{
                                        const ctx = canvas.getContext('2d');
                                        if (ctx) {{
                                            const binary_string = window.atob('{base64_frame}');
                                            const len = binary_string.length;
                                            const bytes = new Uint8Array(len);
                                            for (let i = 0; i < len; i++) {{
                                                bytes[i] = binary_string.charCodeAt(i);
                                            }}
                                            const imageData = new ImageData(new Uint8ClampedArray(bytes.buffer), {width}, {height});
                                            ctx.putImageData(imageData, 0, 0);
                                        }}
                                    }}
                                }} catch (e) {{
                                    console.error("Failed to render frame:", e);
                                }}
                                "#,
                            );
                            let _ = window.webview.evaluate_script(&js_code);
                        }
                        Message::Content(content) => {
                            if !content.starts_with('P')
                                || content.chars().filter(|&c| c == '/').count() != 2
                            {
                                on_scan.call(content);
                                *stop_signal.lock().unwrap() = true;
                            } else {
                                let parts: Vec<&str> = content.splitn(3, '/').collect();
                                if parts.len() == 3 {
                                    if let (Ok(part_num), Ok(total)) = (
                                        parts[0][1..].parse::<usize>(),
                                        parts[1].parse::<usize>(),
                                    ) {
                                        if *total_parts.read() == 0 {
                                            total_parts.set(total);
                                        }
                                        scanned_parts
                                            .write()
                                            .entry(part_num)
                                            .or_insert_with(|| parts[2].to_string());
                                        if scanned_parts.read().len() == *total_parts.read() {
                                            let mut result = String::new();
                                            let reassembly_ok =
                                                (1..=*total_parts.read()).all(|i| {
                                                    scanned_parts
                                                        .read()
                                                        .get(&i)
                                                        .map(|chunk| result.push_str(chunk))
                                                        .is_some()
                                                });
                                            if reassembly_ok {
                                                on_scan.call(result);
                                                *stop_signal.lock().unwrap() = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Message::Done => {
                            on_close.call(());
                            is_scanning.set(false);
                            break;
                        }
                    }
                }
            }
        });

        // This effect manages the background camera thread. Its only job
        // is to forward messages to the state_updater coroutine.
        use_effect(move || {
            let state_updater_clone = state_updater;
            let stop_signal_clone = stop_signal2.clone();

            spawn(async move {
                is_scanning.set(true);
                error_message.set(None);
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                start_camera_thread(tx, stop_signal_clone);

                while let Some(msg) = rx.recv().await {
                    state_updater_clone.send(msg);
                }
            });
        });

        use_drop({
            let stop_signal = stop_signal3.clone();
            move || {
                *stop_signal.lock().unwrap() = true;
            }
        });

        let error_display = error_message.read().as_ref().map(|err| rsx! {
                p {
                    style: "color: var(--pico-color-red-500);",
                    "{err}"
                }
            });
        let progress_indicator = if *is_scanning.read() {
            Some(if *total_parts.read() > 0 {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                        label {

                            "Scan Progress: {scanned_parts.read().len()} of {total_parts.read()}"
                        }
                        progress {
                            max: "{total_parts.read()}",
                            value: "{scanned_parts.read().len()}",
                        }
                    }
                }
            } else {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                        label {

                            "Aim camera at QR code..."
                        }
                        progress {

                        }
                    }
                }
            })
        } else {
            None
        };
        let (width, height) = *frame_dims.read();
        let scanner_display = if *is_scanning.read() || (width > 0 && height > 0) {
            Some(rsx! {
                div {
                    style: "position: relative; width: 100%; max-width: 400px; margin: auto; border-radius: var(--pico-border-radius); overflow: hidden; border: 1px solid var(--pico-form-element-border-color);",
                    canvas {
                        id: "qr-canvas",
                        width: "{width}",
                        height: "{height}",
                        style: "width: 100%; height: auto; display: block; background-color: #333;",
                    }
                }
            })
        } else {
            None
        };

        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem; max-width: 500px; margin: auto;",
                h3 {

                    "Scan QR Code"
                }
                {error_display}
                {progress_indicator}
                {scanner_display}
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
}

#[cfg(not(target_arch = "wasm32"))]
mod server {
    use dioxus::prelude::*;

    #[component]
    #[allow(unused_variables)]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        unimplemented!()
    }
}


/// Contains the QR scanner implementation for mobile platforms.
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile {
    use dioxus::prelude::*;

    #[component]
    #[allow(unused_variables)]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        // This is a stub for mobile. You would integrate a mobile-specific camera
        // crate here and enable it via features in Cargo.toml.
        rsx! {
            div {
                style: "color: var(--pico-color-red-500); border: 1px solid var(--pico-color-red-500); padding: 1rem; border-radius: var(--pico-border-radius);",
                h4 {

                    "Not Implemented"
                }
                p {

                    "QR code scanning is not yet available on mobile devices."
                }
                button {
                    style: "margin-top: 1rem;",
                    onclick: move |_| on_close.call(()),
                    "Close"
                }
            }
        }
    }
}