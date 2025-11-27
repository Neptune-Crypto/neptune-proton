//=============================================================================
// File: src/components/qr_scanner.rs
//=============================================================================

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(any(
    feature = "dioxus-desktop",
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
))]
pub use self::unified::*;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios"))
))]
pub use self::server::*;

// Serde allows us to easily pass this struct between Rust and the JS WebView
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoDevice {
    pub id: String,
    pub label: String,
}

// Typed enum for handling messages from JavaScript
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsMessage {
    Status { msg: String },
    Error { msg: String },
    Content { value: String },
    DeviceList { devices: Vec<VideoDevice> },
}

#[cfg(any(
    feature = "dioxus-desktop",
    target_arch = "wasm32",
    target_os = "android",
    target_os = "ios"
))]
mod unified {
    use super::*;

    #[component]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        let mut error_message = use_signal(|| None::<String>);
        let mut scanned_parts = use_signal(HashMap::<usize, String>::new);
        let mut total_parts = use_signal(|| 0_usize);

        let mut video_devices = use_signal(Vec::<VideoDevice>::new);
        let mut selected_device_id = use_signal(String::new);
        let mut scanner_status = use_signal(|| "Initializing...".to_string());

        // Start Scanner Loop
        // This effect runs once on mount, and re-runs whenever 'selected_device_id' changes.
        use_effect(move || {
            let device_id = selected_device_id.read().clone();

            spawn(async move {
                scanner_status.set("Starting Camera...".into());
                let mut rx = platform_impl::start_scanner(&device_id).await;

                while let Some(msg) = rx.recv().await {
                    match msg {
                        JsMessage::Status { msg } => scanner_status.set(msg),
                        JsMessage::Error { msg } => error_message.set(Some(msg)),
                        JsMessage::Content { value } => {
                            handle_scan_result(
                                value,
                                on_scan,
                                on_close,
                                &mut scanned_parts,
                                &mut total_parts
                            );
                        },
                        JsMessage::DeviceList { devices } => {
                            // Update the device list if it has changed
                            if video_devices.read().len() != devices.len() {
                                // If no device is currently selected, pick the first one
                                if selected_device_id.read().is_empty() {
                                    if let Some(first) = devices.first() {
                                        selected_device_id.set(first.id.clone());
                                    }
                                }
                                video_devices.set(devices);
                            }
                        }
                    }
                }
            });
        });

        let error_display = error_message.read().as_ref().map(|err| rsx! {
            p { style: "color: var(--pico-color-red-500);", "{err}" }
        });

        let status_display = if error_message.read().is_none() {
            rsx! {
                p { style: "font-size: 0.8rem; color: #888;", "{scanner_status}" }
            }
        } else {
            rsx! {}
        };

        let progress_indicator = if *total_parts.read() > 0 {
            rsx! {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                    label { "Scan Progress: {scanned_parts.read().len()} of {total_parts.read()}" }
                    progress { max: "{total_parts.read()}", value: "{scanned_parts.read().len()}" }
                }
            }
        } else {
            rsx! {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 400px; margin: auto;",
                    label { "Aim camera at QR code..." }
                    progress {}
                }
            }
        };

        let device_selector = if !video_devices.read().is_empty() {
            rsx! {
                div {
                    style: "margin-top: 1rem; width: 100%; max-width: 400px; margin: auto;",
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

        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 1.5rem; max-width: 500px; margin: auto;",
                h3 { "Scan QR Code" }
                {error_display}
                {status_display}
                {progress_indicator}

                div {
                    style: "position: relative; width: 100%; max-width: 400px; margin: auto; border-radius: var(--pico-border-radius); overflow: hidden; border: 1px solid var(--pico-form-element-border-color);",
                    video {
                        id: "qr-video",
                        autoplay: true,
                        playsinline: true,
                        style: "width: 100%; height: auto; display: block; background-color: #000;",
                    }
                    canvas {
                        id: "qr-canvas",
                        style: "display: none;",
                    }
                }

                {device_selector}

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

    mod platform_impl {
        use super::{JsMessage};
        use dioxus::prelude::*;

        // some browsers do not support Barcode API.
        // so we bundle a JS impl.
        const JS_QR_SOURCE: &str = include_str!("../../assets/js/jsQR.js");

        pub async fn start_scanner(device_id: &str) -> tokio::sync::mpsc::UnboundedReceiver<JsMessage> {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let tx = std::sync::Arc::new(tx);

            let requested_device_id = device_id.to_string();

            // We inject the library + logic into the WebView
            let script = format!(r#"
                // 1. Inject the bundled JS Library
                {library_code}

                // 2. Main Scanner Logic
                const video = document.getElementById('qr-video');
                const canvas = document.getElementById('qr-canvas');
                if (!video || !canvas) return;

                if (video.srcObject) video.srcObject.getTracks().forEach(t => t.stop());

                let isRunning = true;
                let hasNativeAPI = ('BarcodeDetector' in window);
                let barcodeDetector = hasNativeAPI ? new BarcodeDetector({{formats: ['qr_code']}}) : null;

                async function run() {{
                    try {{
                        // Preference: Environment (rear) camera
                        let constraints = {{
                             video: {{
                                 facingMode: "environment"
                             }}
                        }};

                        // If specific ID requested, override
                        const reqId = "{req_id}";
                        if (reqId && reqId !== "") {{
                            constraints.video = {{ deviceId: {{ exact: reqId }} }};
                        }}

                        const stream = await navigator.mediaDevices.getUserMedia(constraints);

                        // Check if component unmounted while we were awaiting permissions
                        if (!video.isConnected) {{
                            stream.getTracks().forEach(t => t.stop());
                            return;
                        }}

                        video.srcObject = stream;
                        video.setAttribute('playsinline', 'true');
                        await video.play();

                        // --- Refresh Device List ---
                        try {{
                            const devices = await navigator.mediaDevices.enumerateDevices();
                            const videoDevices = devices
                                .filter(d => d.kind === 'videoinput')
                                .map(d => ({{ id: d.deviceId, label: d.label || "Camera " + (d.deviceId.substr(0,5)) }}));

                            dioxus.send({{type: "devicelist", devices: videoDevices}});
                        }} catch (e) {{
                            console.error("Enumerate error", e);
                        }}

                        dioxus.send({{type: "status", msg: "Scanning..."}});

                        const ctx = canvas.getContext('2d', {{ willReadFrequently: true }});

                        const scanFrame = async () => {{
                            // Self-terminating loop if element is removed from DOM
                            if (!video.isConnected) {{
                                if (stream) stream.getTracks().forEach(t => t.stop());
                                isRunning = false;
                                return;
                            }}

                            if (!isRunning) return;

                            if (video.readyState === video.HAVE_ENOUGH_DATA && video.videoWidth > 0) {{
                                try {{
                                    if (hasNativeAPI) {{
                                        const barcodes = await barcodeDetector.detect(video);
                                        if (barcodes.length > 0) {{
                                            dioxus.send({{type: "content", value: barcodes[0].rawValue}});
                                        }}
                                    }} else if (window.jsQR) {{
                                        // JS Fallback
                                        if (canvas.width !== video.videoWidth) {{
                                            canvas.width = video.videoWidth;
                                            canvas.height = video.videoHeight;
                                        }}
                                        ctx.drawImage(video, 0, 0);
                                        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

                                        const code = jsQR(imageData.data, imageData.width, imageData.height, {{
                                            inversionAttempts: "dontInvert",
                                        }});
                                        if (code) {{
                                            dioxus.send({{type: "content", value: code.data}});
                                        }}
                                    }}
                                }} catch (err) {{
                                    // ignore temporary errors
                                }}
                            }}

                            // Throttle
                            const delay = hasNativeAPI ? 100 : 200;
                            setTimeout(() => {{
                                if(isRunning) requestAnimationFrame(scanFrame);
                            }}, delay);
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
                while let Ok(msg) = eval.recv::<JsMessage>().await {
                    let _ = tx.send(msg);
                }
            });
            rx
        }
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "dioxus-desktop"),
    not(any(target_os = "android", target_os = "ios"))
))]
mod server {
    use dioxus::prelude::*;

    #[component]
    #[allow(unused_variables)]
    pub fn QrScanner(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
        unimplemented!("Server QR scanner not implemented")
    }
}