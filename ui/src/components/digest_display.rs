use dioxus::prelude::*;
use twenty_first::tip5::Digest;

use crate::components::pico::CopyButton;

/// A small helper component to display a Digest with a label and copy button.
#[component]
pub fn DigestDisplay(digest: Digest, as_code: bool) -> Element {
    let digest_str = digest.to_hex();
    let abbreviated_digest = format!(
        "{}...{}",
        &digest_str[0..12],
        &digest_str[digest_str.len() - 12..]
    );

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem;",
            if as_code {
                code {
                    title: "{digest_str}",
                    "{abbreviated_digest}"
                }
            } else {
                span {
                    title: "{digest_str}",
                    "{abbreviated_digest}"
                }
            }
            CopyButton {
                text_to_copy: digest_str,
            }
        }
    }
}
