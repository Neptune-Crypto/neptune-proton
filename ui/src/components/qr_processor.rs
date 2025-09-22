//=============================================================================
// File: src/components/qr_processor.rs
//=============================================================================
use image::GrayImage;
use std::collections::HashMap;

/// The result of processing a single QR image frame.
pub enum QrProcessResult {
    /// The QR code is part of an animation and is not yet complete.
    /// Provides (parts_found, parts_total).
    Incomplete(usize, usize),
    /// The QR code was successfully scanned and reassembled. Contains the full data.
    Complete(String),
    /// An error occurred during scanning or reassembly.
    Error(String),
}

/// A stateful processor for handling static and animated QR codes from image buffers.
#[derive(Default)]
pub struct QrProcessor {
    scanned_parts: HashMap<usize, String>,
    total_parts: Option<usize>,
    is_complete: bool,
}

impl QrProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// Processes a single image buffer, updating internal state for animated QRs.
    pub fn process_image(&mut self, image_buffer: GrayImage) -> QrProcessResult {
        if self.is_complete {
            return QrProcessResult::Error("Processing already completed.".to_string());
        }

        let mut prepared_image = rqrr::PreparedImage::prepare(image_buffer);
        let grids = prepared_image.detect_grids();

        let Some(grid) = grids.first() else {
            return QrProcessResult::Error("No QR code found in image.".to_string());
        };

        let Ok((_meta, content)) = grid.decode() else {
            return QrProcessResult::Error("Failed to decode QR content.".to_string());
        };

        // Case 1: Simple, non-animated QR code
        if !content.starts_with('P') || content.chars().filter(|&c| c == '/').count() != 2 {
            self.is_complete = true;
            return QrProcessResult::Complete(content);
        }

        // Case 2: Animated QR code part
        let parts: Vec<&str> = content.splitn(3, '/').collect();
        if parts.len() != 3 {
            return QrProcessResult::Error(format!("Invalid animated QR frame format: {}", content));
        }

        let (Ok(part_num), Ok(total)) = (parts[0][1..].parse::<usize>(), parts[1].parse::<usize>()) else {
            return QrProcessResult::Error(format!("Invalid part/total in frame: {}", content));
        };

        // Initialize total_parts if this is the first frame we've seen
        if self.total_parts.is_none() {
            self.total_parts = Some(total);
        }

        // Insert the part if we haven't seen it before
        self.scanned_parts.entry(part_num).or_insert_with(|| parts[2].to_string());

        let num_scanned = self.scanned_parts.len();
        let total_expected = self.total_parts.unwrap_or(0);

        // Check if we have all the parts
        if total_expected > 0 && num_scanned == total_expected {
            let mut result = String::new();
            for i in 1..=total_expected {
                if let Some(chunk) = self.scanned_parts.get(&i) {
                    result.push_str(chunk);
                } else {
                    // This case is rare but possible if a part is missed
                    return QrProcessResult::Error(format!("Reassembly failed: Missing part {}", i));
                }
            }
            self.is_complete = true;
            return QrProcessResult::Complete(result);
        }

        QrProcessResult::Incomplete(num_scanned, total_expected)
    }
}
