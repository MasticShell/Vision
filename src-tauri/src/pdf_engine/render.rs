//! Page rendering for in-app review/preview.
//!
//! M1 status: All Poppler-based rendering pipelines (pdftoppm, pdftotext) have
//! been removed. Rendering capabilities return `CapabilityUnavailable` or `false`.
//! PDFium integration will be introduced in M2 to provide fast, in-process
//! page rendering without external CLI dependencies.

use crate::error::AppError;
use crate::models::{JobHandle, PagePick, RenderedThumb};
use std::sync::Arc;

/// Extract the text of each page (for in-app search).
///
/// **M1 stub**: Disabled during M1 migration; returns `CapabilityUnavailable`.
pub fn page_texts(_app: &tauri::AppHandle, _input: &str) -> Result<Vec<String>, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Feature temporarily unavailable",
        "Text extraction is disabled during the M1 architecture migration.",
    ))
}

/// Extract one page into a tiny standalone PDF and return it base64-encoded.
/// Returns `None` to fallback gracefully to the default viewer.
pub fn page_pdf_b64(
    _app: &tauri::AppHandle,
    _input: &str,
    _page: u32,
) -> Result<Option<String>, AppError> {
    Ok(None)
}

/// Visually compare page A and page B.
///
/// **M1 stub**: Disabled during M1 migration; returns `CapabilityUnavailable`.
pub fn diff_pages(
    _app: &tauri::AppHandle,
    _a_path: &str,
    _a_page: u32,
    _b_path: &str,
    _b_page: u32,
    _size: u32,
) -> Result<crate::models::DiffResult, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Feature temporarily unavailable",
        "Diff is disabled during the M1 architecture migration.",
    ))
}

/// Whether a page renderer is available (controls the "Pick visually" UI).
///
/// In M1, returns `false` because Poppler has been removed and PDFium is
/// scheduled for M2.
pub fn available(_app: &tauri::AppHandle) -> bool {
    false
}

/// Render thumbnails for the given pages.
///
/// **M1 stub**: Disabled during M1 migration; returns `CapabilityUnavailable`.
pub fn render_thumbnails(
    _app: &tauri::AppHandle,
    _input: &str,
    _pages: &[u32],
    _size: u32,
) -> Result<Vec<RenderedThumb>, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Feature temporarily unavailable",
        "Rendering thumbnails is disabled during the M1 architecture migration.",
    ))
}

/// Export each page of the combined document to an image file.
///
/// **M1 stub**: Disabled during M1 migration; returns `CapabilityUnavailable`.
pub fn to_images(
    _app: &tauri::AppHandle,
    _handle: &Arc<JobHandle>,
    _job_id: &str,
    _picks: &[PagePick],
    _out_dir: &str,
    _format: &str,
    _dpi: u32,
) -> Result<Vec<String>, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Feature temporarily unavailable",
        "Exporting to images is disabled during the M1 architecture migration.",
    ))
}

// ---- tiny dependency-free helpers ----------------------------------------

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn base64(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        s.push(B64[((n >> 18) & 63) as usize] as char);
        s.push(B64[((n >> 12) & 63) as usize] as char);
        s.push(if chunk.len() > 1 {
            B64[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        s.push(if chunk.len() > 2 {
            B64[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    s
}
