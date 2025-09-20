//=============================================================================
// File: src/components/qr_code.rs
//=============================================================================
use dioxus::prelude::*;
use qrcode::{QrCode, EcLevel};
use qrcode::render::svg;
use crate::compat::interval::Interval;
use std::time::Duration;

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
    // The maximum number of alphanumeric characters for a simple, scannable QR code.
    // Version 10, Level H is a good target.
    const CHUNK_SIZE: usize = 120;

    let uppercased_data = props.data.to_uppercase();

    // If the data is small enough, render a single static QR code.
    if uppercased_data.len() <= CHUNK_SIZE {
        match QrCode::with_error_correction_level(uppercased_data.as_bytes(), EcLevel::H) {
            Ok(code) => {
                let image = code.render::<svg::Color>()
                    .min_dimensions(200, 200)
                    .build();

                let tooltip_text = props.tooltip.as_deref().unwrap_or(&props.data);

                rsx! {
                    figure {
                        style: "margin: 0;",
                        div {
                            title: "{tooltip_text}",
                            dangerous_inner_html: "{image}"
                        }
                        if let Some(caption_text) = &props.caption {
                            figcaption {
                                style: "text-align: center; font-size: 14px; margin-top: 8px;",
                                "{caption_text}"
                            }
                        }
                    }
                }
            },
            Err(e) => rsx! {
                p {
                    style: "color: red; font-family: sans-serif; font-size: 14px; border: 1px solid red; padding: 10px; border-radius: 5px;",
                    "Error generating QR code: {e}"
                }
            }
        }
    } else {
        // Otherwise, render an animated, multipart QR code.
        rsx! {
            AnimatedQrCode {
                data: uppercased_data,
                tooltip: props.tooltip.clone(),
                caption: props.caption.clone()
            }
        }
    }
}


#[derive(Props, Clone, PartialEq)]
struct AnimatedQrCodeProps {
    data: String,
    tooltip: Option<String>,
    caption: Option<String>,
}

#[allow(non_snake_case)]
fn AnimatedQrCode(props: AnimatedQrCodeProps) -> Element {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    const CHUNK_SIZE: usize = 120;
    let mut current_frame = use_signal(|| 0);
    // `use_hook` is the correct hook in Dioxus 0.6.3 to initialize a
    // persistent, non-reactive value once.
    let stop_flag = use_hook(|| Arc::new(AtomicBool::new(false)));

    let data = props.data.clone();

    let frames = use_memo(move || {
        let chunks: Vec<_> = props.data.chars().collect::<Vec<char>>()
            .chunks(CHUNK_SIZE)
            .map(|c| c.iter().collect::<String>())
            .collect();

        let total_parts = chunks.len();

        chunks.into_iter().enumerate().map(|(i, chunk)| {
            format!("P{}/{}/{}", i + 1, total_parts, chunk)
        }).collect::<Vec<String>>()
    });

    // This effect runs once on mount to start the background timer.
    use_effect({
        let stop_flag = stop_flag.clone();
        move || {
            let frames = frames.to_owned();

            let stop_flag = stop_flag.clone();
            spawn(async move {
                loop {
                    // Check if the component has been unmounted.
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    // Use a standard async sleep.
                    tokio::time::sleep(Duration::from_millis(300)).await;

                    let total_frames = frames.read().len();
                    if total_frames > 0 {
                        current_frame.with_mut(|frame| {
                            *frame = (*frame + 1) % total_frames;
                        });
                    }
                }
            });
        }
    });

    // This runs when the component is unmounted.
    use_on_unmount({
        let stop_flag = stop_flag.clone();
        move || {
            // Signal the background task to stop. This does not trigger a rerender.
            stop_flag.store(true, Ordering::Relaxed);
        }
    });

    let frame_data = &frames.read()[*current_frame.read()];

    match QrCode::with_error_correction_level(frame_data.as_bytes(), EcLevel::L) {
        Ok(code) => {
            let image = code.render::<svg::Color>()
                .min_dimensions(200, 200)
                .build();

            let tooltip_text = props.tooltip.as_deref().unwrap_or(&data);
            let caption_text = props.caption.as_deref().unwrap_or(&data);
            let frame_text = format!("Part {} of {}", *current_frame.read() + 1, frames.read().len());

            rsx! {
                figure {
                    style: "margin: 0;",
                    div {
                        title: "{tooltip_text}",
                        dangerous_inner_html: "{image}"
                    }
                    figcaption {
                        style: "text-align: center; font-size: 14px; margin-top: 8px;",
                        "{caption_text}"
                    }
                    figcaption {
                        style: "text-align: center; font-size: 14px; margin-top: 8px;",
                        "{frame_text}"
                    }
                }
            }
        },
        Err(e) => rsx! {
             p {
                style: "color: red; font-family: sans-serif; font-size: 14px; border: 1px solid red; padding: 10px; border-radius: 5px;",
                "Error generating animated QR code: {e}"
            }
        }
    }
}