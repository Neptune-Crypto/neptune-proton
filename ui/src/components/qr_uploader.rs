//=============================================================================
// File: src/components/qr_uploader.rs
//=============================================================================
use crate::components::pico::{Button, Modal};
use crate::components::qr_processor::{QrProcessResult, QrProcessor};
use crate::compat;
use dioxus::prelude::*;

mod svg_reader {
    use image::GrayImage;
    use resvg::tiny_skia;
    use usvg::{fontdb, Tree, Transform};

    /// Extracts the viewBox and the visual data for each frame from our SVG format.
    pub fn extract_svg_details(svg_data: &str) -> Result<(String, Vec<String>), String> {
        // First, extract the viewBox attribute from the main <svg> tag.
        let view_box = svg_data
            .split_once("viewBox=\"")
            .and_then(|(_, after)| after.split_once('"'))
            .map(|(vb, _)| vb.to_string())
            .ok_or_else(|| "Could not find viewBox attribute in the SVG file.".to_string())?;

        let mut frames = Vec::new();

        // Extract the content within each <g class="qr-frame">...</g> tag.
        for g_tag in svg_data.split(r#"<g class="qr-frame""#).skip(1) {
            if let Some(content_end) = g_tag.find("</g>") {
                if let Some(content_start) = g_tag.find('>') {
                    let content = &g_tag[content_start + 1 .. content_end];
                    frames.push(content.to_string());
                }
            }
        }

        // Fallback for static QR codes that don't use the <g> frame structure.
        if frames.is_empty() {
            if let Some(path_start) = svg_data.find("<path") {
                if let Some(path_end) = svg_data[path_start..].find("/>") {
                    let path_slice = &svg_data[path_start .. path_start + path_end + 2];
                    frames.push(path_slice.to_string());
                }
            }
        }

        if frames.is_empty() {
             Err("No valid QR frames found in the SVG file.".to_string())
        } else {
            Ok((view_box, frames))
        }
    }

    /// Renders a single SVG frame string to a pixel buffer using the correct viewBox.
    pub fn render_svg_frame(frame_svg_content: &str, view_box: &str) -> Result<GrayImage, String> {
        // Wrap the frame content in a valid SVG container with the correct viewBox.
        let full_svg = format!(
            r#"<svg width="400" height="400" viewBox="{}" xmlns="http://www.w3.org/2000/svg">
                <rect width="100%" height="100%" fill="white"/>
                {}
            </svg>"#,
            view_box,
            frame_svg_content
        );

        let fontdb = fontdb::Database::new();

        let rtree = Tree::from_data(full_svg.as_bytes(), &usvg::Options::default(), &fontdb)
            .map_err(|e| format!("usvg parse error: {}", e))?;

        let pixmap_size = rtree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| "Failed to create pixmap".to_string())?;

        resvg::render(
            &rtree,
            Transform::identity(),
            &mut pixmap.as_mut(),
        );

        let luma_data: Vec<u8> = pixmap
            .data()
            .chunks_exact(4)
            .map(|p| (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) as u8)
            .collect();

        GrayImage::from_raw(pixmap.width(), pixmap.height(), luma_data)
             .ok_or_else(|| "Failed to create GrayImage from buffer".to_string())
    }
}

#[component]
pub fn QrUploader(on_scan: EventHandler<String>, on_close: EventHandler<()>) -> Element {
    let mut upload_progress = use_signal(|| (0, 0));
    let mut upload_error = use_signal(|| None::<String>);
    let mut is_processing = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            is_processing.set(true);
            let file_content = match compat::read_file("svg").await {
                Ok(Some(content)) => content,
                Ok(None) => { on_close.call(()); return; }
                Err(e) => {
                    upload_error.set(Some(format!("Failed to read file: {}", e)));
                    is_processing.set(false);
                    return;
                }
            };

            // 1. Extract both viewBox and frames from the SVG
            let (view_box, frames) = match svg_reader::extract_svg_details(&file_content) {
                Ok(details) => details,
                Err(e) => {
                    upload_error.set(Some(e));
                    is_processing.set(false);
                    return;
                }
            };

            upload_progress.set((0, frames.len()));
            let mut processor = QrProcessor::new();

            // 2. Loop through frames and render using the extracted viewBox
            for (i, frame_svg) in frames.iter().enumerate() {
                upload_progress.set((i + 1, frames.len()));

                let pixel_buffer = match svg_reader::render_svg_frame(frame_svg, &view_box) {
                    Ok(pb) => pb,
                    Err(e) => {
                        upload_error.set(Some(format!("Failed to render SVG frame: {}", e)));
                        is_processing.set(false);
                        return;
                    }
                };
                match processor.process_image(pixel_buffer) {
                    QrProcessResult::Complete(data) => { on_scan.call(data); return; }
                    QrProcessResult::Incomplete(_, _) => { compat::sleep(std::time::Duration::from_millis(10)).await; }
                    QrProcessResult::Error(e) => {
                        // For animated SVGs, it's possible the first frame is intentionally blank
                        // or unscannable, so we'll only show an error if ALL frames fail.
                        // We continue the loop.
                    }
                }
            }

            if !processor.is_complete() {
                let final_error = upload_error().unwrap_or_else(||
                    "Scanned all frames, but the QR code is still incomplete.".to_string()
                );
                upload_error.set(Some(final_error));
            }
            is_processing.set(false);
        });
    });

    rsx! {
        div {
            h3 { "Processing QR Image" }
            if let Some(err) = upload_error() {
                div { p { style: "color: var(--pico-color-red-500);", "{err}" } }
            } else if *is_processing.read() {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; min-width: 300px;",
                    p { "Please wait..." },
                    label { "Processing frame {upload_progress().0} of {upload_progress().1}" },
                    progress { max: "{upload_progress().1}", value: "{upload_progress().0}" }
                }
            }
            div {
                style: "margin-top: 1rem;",
                Button { on_click: move |_| on_close.call(()), "Close" }
            }
        }
    }
}