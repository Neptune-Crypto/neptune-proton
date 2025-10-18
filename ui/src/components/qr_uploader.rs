//=============================================================================
// File: src/components/qr_uploader.rs
//=============================================================================
use crate::compat;
use crate::components::pico::Button;
use crate::components::qr_processor::{QrProcessResult, QrProcessor};
use dioxus::prelude::*;

mod svg_reader {
    use image::GrayImage;
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use quick_xml::Writer;
    use resvg::tiny_skia;
    use usvg::{fontdb, Transform, Tree};

    /// Extracts the viewBox and the visual data for each frame from our SVG format.
    pub fn extract_svg_details(svg_data: &str) -> Result<(String, Vec<String>), String> {
        let view_box = svg_data
            .split_once("viewBox=\"")
            .and_then(|(_, after)| after.split_once('"'))
            .map(|(vb, _)| vb.to_string())
            .ok_or_else(|| "Could not find viewBox attribute in the SVG file.".to_string())?;

        let mut reader = Reader::from_str(svg_data);
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut frames = Vec::new();
        let mut is_in_frame = false;
        let mut writer = Writer::new(Vec::new());

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e))
                    if e.name().as_ref() == b"g"
                        && e.attributes().any(|a| {
                            a.is_ok_and(|a| {
                                a.key.as_ref() == b"class" && a.value.as_ref() == b"qr-frame"
                            })
                        }) =>
                {
                    is_in_frame = true;
                    writer.get_mut().clear();
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"g" && is_in_frame => {
                    is_in_frame = false;
                    let frame_str =
                        String::from_utf8(writer.get_mut().to_vec()).map_err(|e| e.to_string())?;
                    frames.push(frame_str);
                }
                Ok(Event::Eof) => break,
                Ok(event) => {
                    if is_in_frame {
                        writer.write_event(&event).map_err(|e| e.to_string())?;
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "XML Error at position {}: {:?}",
                        reader.buffer_position(),
                        e
                    ))
                }
            }
            buf.clear();
        }

        if frames.is_empty() {
            if let Some(path_start) = svg_data.find("<path") {
                if let Some(end_svg) = svg_data.rfind("</svg>") {
                    frames.push(svg_data[path_start..end_svg].to_string());
                }
            }
        }

        if frames.is_empty() {
            Err("No valid QR frames found in the SVG file.".to_string())
        } else {
            Ok((view_box, frames))
        }
    }

    /// Renders a single SVG frame string to a pixel buffer.
    pub fn render_svg_frame(frame_svg_content: &str, view_box: &str) -> Result<GrayImage, String> {
        let full_svg = format!(
            r#"<svg width="400" height="400" viewBox="{}" xmlns="http://www.w3.org/2000/svg">
                <rect width="100%" height="100%" fill="white"/>
                {}
            </svg>"#,
            view_box, frame_svg_content
        );

        let fontdb = fontdb::Database::new();
        let rtree = Tree::from_data(full_svg.as_bytes(), &usvg::Options::default(), &fontdb)
            .map_err(|e| format!("usvg parse error: {}", e))?;

        let pixmap_size = rtree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| "Failed to create pixmap".to_string())?;

        resvg::render(&rtree, Transform::identity(), &mut pixmap.as_mut());

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
    let mut status_message = use_signal(|| "Waiting for file selection...".to_string());

    use_effect(move || {
        spawn(async move {
            let file_content = match compat::read_file("svg").await {
                Ok(Some(content)) => content,
                Ok(None) => {
                    on_close.call(());
                    return;
                }
                Err(e) => {
                    upload_error.set(Some(format!("Failed to read file: {}", e)));
                    return;
                }
            };

            is_processing.set(true);
            status_message.set("Extracting frames...".to_string());

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

            for (i, frame_svg) in frames.iter().enumerate() {
                status_message.set(format!("Processing frame {} of {}", i + 1, frames.len()));
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
                    QrProcessResult::Complete(data) => {
                        on_scan.call(data);
                        return;
                    }
                    QrProcessResult::Incomplete(_, _) => {
                        compat::sleep(std::time::Duration::from_millis(1)).await;
                    }
                    QrProcessResult::Error(_) => {}
                }
            }

            if !processor.is_complete() {
                upload_error.set(Some(
                    "Scanned all frames, but the QR code is still incomplete.".to_string(),
                ));
            }
            is_processing.set(false);
        });
    });

    rsx! {
        div {
            h3 { "Processing QR Image" }
            if let Some(err) = upload_error() {
                div { p { style: "color: var(--pico-color-red-500);", "{err}" } }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; min-width: 300px;",
                    p { "{status_message}" },
                    if *is_processing.read() {
                        progress {
                            max: "{upload_progress().1}",
                            value: "{upload_progress().0}"
                        }
                    }
                }
            }
            div {
                style: "margin-top: 1rem;",
                Button { on_click: move |_| on_close.call(()), "Close" }
            }
        }
    }
}
