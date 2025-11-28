//=============================================================================
// File: src/components/qr_scanner.rs
//=============================================================================

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Platform Implementation Selector ---

// Tier 1 (Web/Mobile): Uses WebView/JS APIs.
// Now strictly for WASM and Mobile targets.
#[cfg(any(
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
))]
use self::web_impl as platform_impl;

// Tier 2 (Desktop): Uses Native Rust (Nokhwa).
// UNIFIED: Linux, Windows, and macOS all use the native path now.
#[cfg(all(feature = "dioxus-desktop", any(target_os = "linux", target_os = "windows", target_os = "macos")))]
use self::native_impl as platform_impl;

// Tier 3 (Server/Unknown): Stub
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios", target_os = "linux", target_os = "windows", target_os = "macos"))
))]
use self::server_impl as platform_impl;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoDevice {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScannerMessage {
    Status { msg: String },
    Error { msg: String },
    Content { value: String },
    DeviceList { devices: Vec<VideoDevice> },
    // Used specifically by Desktop to push frames to the UI
    FrameBase64 { data: String, width: u32, height: u32 },
}

#[component]
pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
    let mut error_message = use_signal(|| None::<String>);
    let mut scanned_parts = use_signal(HashMap::<usize, String>::new);
    let mut total_parts = use_signal(|| 0_usize);

    let mut video_devices = use_signal(Vec::<VideoDevice>::new);
    let mut selected_device_id = use_signal(String::new);
    let mut scanner_status = use_signal(|| "Initializing...".to_string());

    // Controls the horizontal flip (Mirroring)
    let mut mirror_feed = use_signal(|| true);

    // --- Main Logic Loop ---
    use_effect(move || {
        // Rerun the effect whenever the selected_device_id changes
        let device_id = selected_device_id.read().clone();

        spawn(async move {
            scanner_status.set("Starting Camera...".into());
            let mut rx = platform_impl::start_scanner(&device_id).await;

            while let Some(msg) = rx.recv().await {
                match msg {
                    ScannerMessage::Status { msg } => scanner_status.set(msg),
                    ScannerMessage::Error { msg } => {
                        // The error filtering is essential here, as global errors may still be caught.
                        let is_cosmetic_linux_error = cfg!(all(feature = "dioxus-desktop", target_os = "linux")) &&
                                                      msg.contains("CameraFormat: Failed to Fufill");

                        if !is_cosmetic_linux_error {
                            error_message.set(Some(msg));
                        }
                    },
                    ScannerMessage::Content { value } => {
                        handle_scan_result(value, on_scan, on_close, &mut scanned_parts, &mut total_parts);
                    },
                    ScannerMessage::DeviceList { devices } => {
                        if video_devices.read().len() != devices.len() {
                            if selected_device_id.read().is_empty() {
                                if let Some(first) = devices.first() {
                                    selected_device_id.set(first.id.clone());
                                }
                            }
                            video_devices.set(devices);
                        }
                    },
                    ScannerMessage::FrameBase64 { data, width, height } => {
                        // Nokhwa (Desktop) uses this to render frames via JS eval
                        let js = format!(
                            r#"
                            try {{
                                const canvas = document.getElementById('qr-canvas');
                                if (canvas) {{
                                    if (canvas.width !== {width}) {{
                                        canvas.width = {width};
                                        canvas.height = {height};
                                    }}
                                    const ctx = canvas.getContext('2d');
                                    const img = new Image();
                                    img.onload = () => ctx.drawImage(img, 0, 0);
                                    img.src = "data:image/jpeg;base64,{data}";
                                }}
                            }} catch(e) {{ console.error(e); }}
                            "#
                        );
                        let _ = document::eval(&js);
                    }
                }
            }
        });
    });

    let error_display = error_message.read().as_ref().map(|err| rsx! {
        p { style: "color: var(--pico-color-red-500);", "{err}" }
    });

    // Determine status text
    let is_scanning_live = scanner_status.read().contains("Live Feed");

    let status_text = if *total_parts.read() > 0 {
        // Multi-part scan in progress
        format!("Scan Progress: {} of {}", scanned_parts.read().len(), total_parts.read())
    } else if is_scanning_live {
        // Live feed is active, show prompt to user
        "Aim camera at QR code...".to_string()
    } else {
        // Initializing or loading
        scanner_status.read().clone()
    };


    let progress_indicator = if *total_parts.read() > 0 {
        rsx! {
            // Display progress bar for multi-part scan
            div {
                class: "mt-2 mb-4",
                style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                label { "{status_text}" }
                progress { max: "{total_parts.read()}", value: "{scanned_parts.read().len()}" }
            }
        }
    } else {
        rsx! {
            // Display spinner/status for initial state or live feed prompt
            div {
                class: "mt-2 mb-4",
                style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                label { "{status_text}" }
                progress {}
            }
        }
    };

    // FIX: Placeholder for the commented-out <select> field
    let device_selector_hidden = if !video_devices.read().is_empty() {
        rsx! {
            // This div replaces the previous cycling button/placeholder
            div {
                // Style to hide the select element entirely but keep it in the code
                style: "position: absolute; width: 0; height: 0; overflow: hidden; opacity: 0;",
                select {
                    aria_label: "Select Camera",
                    onchange: move |event| selected_device_id.set(event.value()),
                    for device in video_devices.read().iter() {
                        option {
                            key: "{device.id}",
                            value: "{device.id}",
                            selected: *selected_device_id.read() == device.id,
                            "{device.label}"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    };

    let flip_style = if *mirror_feed.read() { "scaleX(-1)" } else { "scaleX(1)" };
    let flip_button_text = "Flip \u{21C6}".to_string();

    // --- UI Layout ---
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 0.5rem; max-width: 500px; margin: auto;",

            // Inject the hidden select field here (FIX 1)
            {device_selector_hidden}

            // Only show error or the unified status/progress block, never both.
            if let Some(err_rsx) = error_display {
                {err_rsx}
            } else {
                // This div controls the spacing for the progress/status block
                div {
                    class: "my-2",
                    {progress_indicator}
                }
            }

            div {
                style: "position: relative; width: 100%; max-width: 400px; margin: auto; border-radius: var(--pico-border-radius); overflow: hidden; border: 1px solid var(--pico-form-element-border-color);",

                // Tier 1: WASM/Mobile uses <video>
                if cfg!(any(target_arch = "wasm32", target_os = "android", target_os = "ios")) {
                    video {
                        id: "qr-video",
                        autoplay: true,
                        playsinline: true,
                        style: "width: 100%; height: auto; display: block; background-color: #000; transform: {flip_style};",
                    }
                }

                // Tier 2: Desktop (Linux/Windows/Mac) uses <canvas> for Native Frames
                if cfg!(all(feature = "dioxus-desktop", any(target_os = "linux", target_os = "windows", target_os = "macos"))) {
                    canvas {
                        id: "qr-canvas",
                        style: format!("width: 100%; height: auto; display: block; background-color: #000; transform: {};", flip_style),
                    }
                }
            }

            // Controls: Flip and Cancel
            div {
                // Use space-around to separate the two buttons nicely
                style: "display: flex; justify-content: space-around; align-items: center; width: 100%; max-width: 400px; margin: 1rem auto 0 auto; gap: 1rem;",

                button {
                    class: "secondary",
                    style: "white-space: nowrap; margin: 0; min-width: 100px;",
                    onclick: move |_| mirror_feed.toggle(),
                    "{flip_button_text}"
                }

                button {
                    onclick: move |_| { on_close.call(()); },
                    style: "margin: 0; min-width: 100px;",
                    "Cancel"
                }
            }
        }
    }
}

// --- Shared Logic ---

fn handle_scan_result(
    content: String,
    on_scan: EventHandler<String>,
    on_close: EventHandler<()>,
    scanned_parts: &mut Signal<HashMap<usize, String>>,
    total_parts: &mut Signal<usize>
) {
    if !content.starts_with('P') || content.chars().filter(|&c| c == '/').count() != 2 {
        on_scan.call(content);
        on_close.call(());
    } else {
        let parts: Vec<&str> = content.splitn(3, '/').collect();
        if parts.len() == 3 {
            if let (Ok(part_num), Ok(total)) = (parts[0][1..].parse::<usize>(), parts[1].parse::<usize>()) {
                if *total_parts.read() == 0 { total_parts.set(total); }
                scanned_parts.write().entry(part_num).or_insert_with(|| parts[2].to_string());
                if scanned_parts.read().len() == *total_parts.read() {
                    let mut result = String::new();
                    let reassembly_ok = (1..=*total_parts.read()).all(|i| scanned_parts.read().get(&i).map(|chunk| result.push_str(chunk)).is_some());
                    if reassembly_ok {
                        on_scan.call(result);
                        on_close.call(());
                    }
                }
            }
        }
    }
}

//=============================================================================
// IMPLEMENTATION 1: WEB/MOBILE (JS / WebView)
//=============================================================================
#[cfg(any(
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
))]
mod web_impl {
    use super::{ScannerMessage, VideoDevice};
    use dioxus::prelude::*;

    const JS_QR_SOURCE: &str = include_str!("../../assets/js/jsQR.js");

    pub async fn start_scanner(device_id: &str) -> tokio::sync::mpsc::UnboundedReceiver<ScannerMessage> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let tx = std::sync::Arc::new(tx);
        let requested_device_id = device_id.to_string();

        let script = format!(r#"
            // 1. Inject the bundled JS Library
            {library_code}

            // 2. Main Scanner Logic
            const video = document.getElementById('qr-video');
            const canvas = document.getElementById('qr-canvas');
            if (!video) return;

            if (video.srcObject) video.srcObject.getTracks().forEach(t => t.stop());

            let isRunning = true;
            let hasNativeAPI = ('BarcodeDetector' in window);
            let barcodeDetector = hasNativeAPI ? new BarcodeDetector({{formats: ['qr_code']}}) : null;

            async function run() {{
                try {{
                    let constraints = {{ video: {{ facingMode: "environment" }} }};
                    const reqId = "{req_id}";
                    if (reqId && reqId !== "") constraints.video = {{ deviceId: {{ exact: reqId }} }};

                    const stream = await navigator.mediaDevices.getUserMedia(constraints);

                    if (!video.isConnected) {{ stream.getTracks().forEach(t => t.stop()); return; }}

                    video.srcObject = stream;
                    video.setAttribute('playsinline', 'true');
                    await video.play();

                    dioxus.send({{type: "status", msg: "Scanning (Live Feed)..."}});

                    try {{
                        const devices = await navigator.mediaDevices.enumerateDevices();
                        const videoDevices = devices
                            .filter(d => d.kind === 'videoinput')
                            .map(d => ({{ id: d.deviceId, label: d.label || "Camera " + (d.deviceId.substr(0,5)) }}));
                        dioxus.send({{type: "devicelist", devices: videoDevices}});
                    }} catch (e) {{}}

                    const ctx = canvas.getContext('2d', {{ willReadFrequently: true }});

                    const scanFrame = async () => {{
                        if (!video.isConnected) {{ if (stream) stream.getTracks().forEach(t => t.stop()); isRunning = false; return; }}
                        if (!isRunning) return;

                        if (video.readyState === video.HAVE_ENOUGH_DATA && video.videoWidth > 0) {{
                            try {{
                                if (hasNativeAPI) {{
                                    const barcodes = await barcodeDetector.detect(video);
                                    if (barcodes.length > 0) dioxus.send({{type: "content", value: barcodes[0].rawValue}});
                                }} else if (window.jsQR && canvas) {{
                                    if (canvas.width !== video.videoWidth) {{ canvas.width = video.videoWidth; canvas.height = video.videoHeight; }}
                                    ctx.drawImage(video, 0, 0);
                                    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
                                    const code = jsQR(imageData.data, imageData.width, imageData.height, {{ inversionAttempts: "dontInvert" }});
                                    if (code) dioxus.send({{type: "content", value: code.data}});
                                }}
                            }} catch (err) {{}}
                        }}
                        setTimeout(() => {{ if(isRunning) requestAnimationFrame(scanFrame); }}, hasNativeAPI ? 100 : 200);
                    }};
                    scanFrame();
                }} catch(e) {{
                    dioxus.send({{ type: "error", msg: e.toString() }});
                }}
            }}
            run();
        "#, library_code = JS_QR_SOURCE, req_id = requested_device_id);

        spawn(async move {
            let mut eval = document::eval(&script);
            while let Ok(msg) = eval.recv::<ScannerMessage>().await {
                let _ = tx.send(msg);
            }
        });
        rx
    }
}

//=============================================================================
// IMPLEMENTATION 2: LINUX/WINDOWS/MACOS (Native Rust / Nokhwa)
//=============================================================================
#[cfg(all(feature = "dioxus-desktop", any(target_os = "linux", target_os = "windows", target_os = "macos")))]
mod native_impl {
    use super::{ScannerMessage, VideoDevice};
    use base64::engine::{general_purpose::STANDARD as BASE64_STANDARD, Engine};
    use nokhwa::pixel_format::RgbFormat;
    use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
    use nokhwa::Camera;
    use std::collections::HashSet;
    use std::thread;

    pub async fn start_scanner(device_id: &str) -> tokio::sync::mpsc::UnboundedReceiver<ScannerMessage> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let req_index = if let Ok(idx) = device_id.parse::<u32>() {
            CameraIndex::Index(idx)
        } else {
            CameraIndex::Index(0)
        };

        thread::spawn(move || {
            let requested = RequestedFormat::new::<RgbFormat>(
                RequestedFormatType::Closest(nokhwa::utils::CameraFormat::new_from(
                    640,
                    480,
                    nokhwa::utils::FrameFormat::MJPEG,
                    30,
                )),
            );

            // --- Camera Initialization ---
            let camera_result = Camera::new(req_index.clone(), requested);

            let mut camera = match camera_result {
                Ok(mut c) => {
                    let _ = c.open_stream();
                    c
                },
                Err(e) if e.to_string().contains("Could not get device property CameraFormat") || e.to_string().contains("CameraFormat: Failed to Fufill") => {
                    // Cosmetic error, silent exit.
                    return;
                }
                Err(e) => {
                    // This is a FATAL error (Camera object couldn't even be created). Report and exit.
                    let error_msg = format!("Camera Init Failed: {}", e.to_string());
                    let _ = tx.send(ScannerMessage::Error { msg: error_msg });
                    return;
                }
            };

            // Enumerate Devices
            // Enumerate Devices and Deduplicate
            if let Ok(devices) = nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
                let mut seen_labels = HashSet::new();
                let list: Vec<VideoDevice> = devices.into_iter().filter_map(|d| {
                    let label = d.human_name();
                    // Deduplicate cameras based on their human-readable name
                    if seen_labels.insert(label.clone()) {
                        Some(VideoDevice {
                            id: if let CameraIndex::Index(n) = d.index() { n.to_string() } else { "0".into() },
                            label,
                        })
                    } else {
                        None
                    }
                }).collect();
                let _ = tx.send(ScannerMessage::DeviceList { devices: list });
            }

            let mut last_scan = std::time::Instant::now();
            let mut is_first_frame = true;

            loop {
                if tx.is_closed() { break; }

                if let Ok(frame) = camera.frame() {
                    if let Ok(decoded) = frame.decode_image::<RgbFormat>() {

                        // FIX: Send status only after the first successful frame
                        if is_first_frame {
                            let _ = tx.send(ScannerMessage::Status { msg: "Scanning (Live Feed)...".into() });
                            is_first_frame = false;
                        }

                        let width = decoded.width();
                        let height = decoded.height();

                        let mut jpeg_data = Vec::new();
                        let dyn_img = image::DynamicImage::ImageRgb8(decoded.clone());

                        // 1. Send Frame to UI (MJPEG quality 60 for speed)
                        let mut writer = std::io::Cursor::new(&mut jpeg_data);
                        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 60);

                        if encoder.encode_image(&dyn_img).is_ok() {
                            let b64 = BASE64_STANDARD.encode(&jpeg_data);
                            if tx.send(ScannerMessage::FrameBase64 { data: b64, width, height }).is_err() { break; }
                        }

                        // 2. Scan for QR (throttled to 5fps)
                        if last_scan.elapsed().as_millis() > 200 {
                            last_scan = std::time::Instant::now();
                            let gray_img = dyn_img.to_luma8();
                            let mut img = rqrr::PreparedImage::prepare(gray_img);
                            if let Some(grid) = img.detect_grids().first() {
                                if let Ok((_, content)) = grid.decode() {
                                    if tx.send(ScannerMessage::Content { value: content }).is_err() { break; }
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        rx
    }
}

//=============================================================================
// IMPLEMENTATION 3: SERVER / UNKNOWN (Stub)
//=============================================================================
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios", target_os = "linux", target_os = "windows", target_os = "macos"))
))]
mod server_impl {
    use super::ScannerMessage;
    pub async fn start_scanner(_: &str) -> tokio::sync::mpsc::UnboundedReceiver<ScannerMessage> {
        tokio::sync::mpsc::unbounded_channel().1
    }
}