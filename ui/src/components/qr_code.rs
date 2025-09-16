// ... imports
use dioxus::prelude::*;
use qrcode::{QrCode, Version, EcLevel};
use qrcode::render::svg;

#[derive(Props, Clone, PartialEq)]
pub struct QrCodeProps {
    data: String,
    #[props(optional)]
    tooltip: Option<String>,
    #[props(optional)]
    caption: Option<String>,
}

#[allow(non_snake_case)]
pub fn QrCode(props: QrCodeProps) -> Element {
    let uppercased_data = props.data.to_uppercase();
    
    // QrCode::new() fails for Generation addresses that are 3400+ chars.
    // So we force it to use qr-code version 40 in such cases.
    const LARGE_DATA_THRESHOLD: usize = 3000;

    let result = if uppercased_data.len() <= LARGE_DATA_THRESHOLD {
        QrCode::new(uppercased_data.as_bytes())
    } else {
        QrCode::with_version(
            uppercased_data.as_bytes(),
            Version::Normal(40),
            EcLevel::L,
        )
    };

match result {
        Ok(code) => {
            let image = code.render::<svg::Color>()
                .min_dimensions(200, 200)
                .build();
            
            // Use the provided tooltip, or default to the original data.
            let tooltip_text = props.tooltip.as_deref().unwrap_or(&props.data);

            rsx! {
                // Use a <figure> for semantic meaning
                figure {
                    style: "margin: 0;", // Reset default figure margins
                    div {
                        title: "{tooltip_text}",
                        dangerous_inner_html: "{image}"
                    }
                    // Conditionally render the caption if it's provided
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
}
