//=============================================================================
// File: src/components/qr_code.rs
//=============================================================================
use base64::Engine;
use dioxus::prelude::*;
use futures::StreamExt;
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};

const STATIC_CHUNK_SIZE: usize = 120;

// The message now includes the filename for the save dialog.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub enum SaveFileAction {
    SaveSvg(String, String), // (svg_data, file_name)
}

#[derive(Props, Clone, PartialEq)]
pub struct QrCodeProps {
    pub data: String,
    #[props(optional)]
    pub tooltip: Option<String>,
    #[props(optional)]
    pub caption: Option<String>,
}

#[allow(non_snake_case)]
pub fn QrCode(props: QrCodeProps) -> Element {
    let uppercased_data = props.data.to_uppercase();

    #[cfg(not(target_arch = "wasm32"))]
    let save_file_coroutine =
        use_coroutine(|mut rx: UnboundedReceiver<SaveFileAction>| async move {
            while let Some(action) = rx.next().await {
                #[allow(irrefutable_let_patterns)]
                if let SaveFileAction::SaveSvg(svg_data, file_name) = action {
                    spawn(async move {
                        if let Some(path) = rfd::AsyncFileDialog::new()
                            .add_filter("SVG Image", &["svg"])
                            .set_file_name(&file_name)
                            .save_file()
                            .await
                        {
                            let _ = tokio::fs::write(path.path(), svg_data).await;
                        }
                    });
                }
            }
        });

    if uppercased_data.len() <= STATIC_CHUNK_SIZE {
        // --- STATIC QR CODE LOGIC WITH DOWNLOAD ---
        match QrCode::with_error_correction_level(uppercased_data.as_bytes(), EcLevel::H) {
            Ok(code) => {
                let svg_image_data =
                    use_memo(move || code.render::<svg::Color>().min_dimensions(200, 200).build());

                let svg_data_url = use_memo(move || {
                    let encoded =
                        base64::engine::general_purpose::STANDARD.encode(&*svg_image_data.read());
                    format!("data:image/svg+xml;base64,{encoded}")
                });

                let file_name = use_memo({
                    let filename_base = if let Some(ref caption) = props.caption {
                        caption.clone()
                    } else {
                        let data = uppercased_data.clone();
                        if data.len() > 24 {
                            format!("{}...{}", &data[..12], &data[data.len() - 12..])
                        } else {
                            data
                        }
                    };
                    let filename_base = filename_base.replace(' ', "_");
                    move || format!("{}-qr.svg", filename_base)
                });

                let tooltip_text = props.tooltip.as_deref().unwrap_or(&props.data);

                let download_element = {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        rsx! {
                            button {
                                onclick: move |_| {
                                    let svg_data = svg_image_data.read().clone();
                                    let name = file_name.read().clone();
                                    save_file_coroutine.send(SaveFileAction::SaveSvg(svg_data, name));
                                },
                                style: "font-size: 12px; margin-top: 10px; padding: 4px 8px;",
                                "Save QR to File"
                            }
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        rsx! {
                             a {
                                href: "{svg_data_url}",
                                download: "{file_name}",
                                style: "font-size: 12px; margin-top: 10px;",
                                "Download QR"
                            }
                        }
                    }
                };

                rsx! {
                    figure {
                        style: "margin: 0; display: flex; flex-direction: column; align-items: center;",
                        img {
                            src: "{svg_data_url}",
                            width: "200",
                            height: "200",
                            title: "{tooltip_text}",
                        }
                        if let Some(caption_text) = &props.caption {
                            figcaption {
                                style: "text-align: center; font-size: 14px; margin-top: 8px;",
                                "{caption_text}"
                            }
                        }
                        {download_element}
                    }
                }
            }
            Err(e) => rsx! {
                p {
                    style: "color: red; font-family: sans-serif; font-size: 14px; border: 1px solid red; padding: 10px; border-radius: 5px;",
                    "Error generating QR code: {e}"
                }
            },
        }
    } else {
        // --- ANIMATED QR CODE LOGIC ---
        let animated_svg = use_memo({
            let data = uppercased_data.clone();
            move || generate_animated_svg(&data)
        });

        let animated_svg_data_url = use_memo(move || {
            let svg_string = animated_svg.read();
            let base64_encoded = base64::engine::general_purpose::STANDARD.encode(&*svg_string);
            format!("data:image/svg+xml;base64,{base64_encoded}")
        });

        let file_name = use_memo({
            let filename_base = if let Some(ref caption) = props.caption {
                caption.clone()
            } else {
                let data = uppercased_data.clone();
                if data.len() > 24 {
                    format!("{}...{}", &data[..12], &data[data.len() - 12..])
                } else {
                    data
                }
            };
            let filename_base = filename_base.replace(' ', "_");
            move || format!("{}-qr.svg", filename_base)
        });

        let tooltip_text = props.tooltip.as_deref().unwrap_or(&props.data);
        let caption_text = props.caption.clone().unwrap_or_default();
        let frame_count = (uppercased_data.len() + STATIC_CHUNK_SIZE - 1) / STATIC_CHUNK_SIZE;

        let download_element = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                rsx! {
                    button {
                        onclick: move |_| {
                            let svg_data = animated_svg.read().clone();
                            let name = file_name.read().clone();
                            save_file_coroutine.send(SaveFileAction::SaveSvg(svg_data, name));
                        },
                        style: "font-size: 12px; margin-top: 10px; padding: 4px 8px;",
                        "Download SVG"
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                rsx! {
                     a {
                        href: "{animated_svg_data_url}",
                        download: "{file_name}",
                        style: "font-size: 12px; margin-top: 10px;",
                        "Download SVG"
                    }
                }
            }
        };

        rsx! {
            figure {
                style: "margin: 0; display: flex; flex-direction: column; align-items: center;",
                img {
                    src: "{animated_svg_data_url}",
                    width: "200",
                    height: "200",
                    title: "{tooltip_text}",
                }
                if !caption_text.is_empty() {
                    figcaption {
                        style: "text-align: center; font-size: 14px; margin-top: 8px;",
                        "{caption_text}"
                    }
                }
                figcaption {
                    style: "text-align: center; font-size: 12px; margin-top: 4px; color: #555;",
                    "Animated QR Code ({frame_count} parts)"
                }
                {download_element}
            }
        }
    }
}

/// Generates a self-contained, animated SVG string for a multipart QR code.
fn generate_animated_svg(data: &str) -> String {
    const CHUNK_SIZE: usize = 120;
    const FRAME_DURATION_MS: u32 = 300;

    let chunks: Vec<_> = data
        .chars()
        .collect::<Vec<char>>()
        .chunks(CHUNK_SIZE)
        .map(|c| c.iter().collect::<String>())
        .collect();

    let total_parts = chunks.len();
    if total_parts == 0 {
        return String::new();
    }

    let frames: Vec<_> = chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| format!("P{}/{}/{}", i + 1, total_parts, chunk))
        .collect();

    // --- Generate the first frame to establish the standard size ---
    let Some(first_frame_data) = frames.first() else {
        return String::new();
    };
    let Ok(first_code) =
        QrCode::with_error_correction_level(first_frame_data.as_bytes(), EcLevel::L)
    else {
        return String::new();
    };

    // Use the version and error correction level from the first frame for all subsequent frames.
    // WARNING: This approach assumes that no frame after the first will ever require a
    // larger QR code version. This can fail if the animation has 10 or more frames,
    // as the header "P10/..." is longer than "P9/...".
    let version = first_code.version();
    let ec_level = first_code.error_correction_level();

    let first_svg_str = first_code.render::<svg::Color>().build();

    let view_box = first_svg_str
        .split_once("viewBox=\"")
        .and_then(|(_, after)| after.split_once('"'))
        .map(|(vb, _)| vb)
        .unwrap_or("0 0 256 256");

    // --- Generate all frame contents, forcing each to the same version ---
    let frame_contents: Vec<String> = frames
        .iter()
        .filter_map(|frame_data| {
            QrCode::with_version(frame_data.as_bytes(), version, ec_level)
                .ok()
                .map(|code| {
                    let svg_str = code.render::<svg::Color>().build();
                    if let Some(path_start) = svg_str.find("<path") {
                        if let Some(end_svg) = svg_str.rfind("</svg>") {
                            return svg_str[path_start..end_svg].to_string();
                        }
                    }
                    String::new()
                })
        })
        .collect();

    let num_frames = frame_contents.len();
    if num_frames == 0 {
        return String::new();
    }

    let total_duration_ms = num_frames as u32 * FRAME_DURATION_MS;
    let frame_visibility_percentage = 100.0 / num_frames as f32;

    let style = format!(
        r#"
        .qr-frame {{ opacity: 0; animation: frame-fade {total_duration_ms}ms infinite; }}
        @keyframes frame-fade {{
            0% {{ opacity: 1; }}
            {frame_visibility_percentage:.2}% {{ opacity: 1; }}
            {next_percentage:.2}% {{ opacity: 0; }}
            100% {{ opacity: 0; }}
        }}
        "#,
        next_percentage = frame_visibility_percentage + 0.01
    );

    let body = frame_contents
        .into_iter()
        .enumerate()
        .map(|(i, content)| {
            let delay = i as u32 * FRAME_DURATION_MS;
            format!(r#"<g class="qr-frame" style="animation-delay: {delay}ms;">{content}</g>"#)
        })
        .collect::<String>();

    let final_svg = format!(
        r#"<svg width="200" height="200" viewBox="{view_box}" xmlns="http://www.w3.org/2000/svg">
            <style>{style}</style>
            <rect width="100%" height="100%" fill="white"/>
            {body}
        </svg>"#,
    );

    final_svg
}
