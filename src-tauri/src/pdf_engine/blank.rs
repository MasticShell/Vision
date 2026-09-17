//! Blank-page detection.
//!
//! M1 status: the rasterisation step (pdftoppm) has been stubbed out.
//! The detection entrypoint returns `CapabilityUnavailable`. The pixel-analysis
//! logic (`luma_stats`, `is_blank`, `threshold_for`) is preserved — it is
//! architecture-independent and will be wired to PDFium in M2.
//!
//! Detection only *reports* pages — removal is done by the existing
//! delete/assemble commands, so there is no new write path here.

use crate::error::AppError;
use crate::models::JobHandle;
use std::sync::Arc;

/// Pixels with a luma below this count as "content" (ink).
#[cfg(test)]
const DARK_LUMA: u8 = 240;

/// A near-uniform page (flat scanner gray) has a luma stddev below this.
#[cfg(test)]
const UNIFORM_STDDEV: f64 = 4.0;

/// The uniform-page rule only applies to *light* pages — a solid dark page
/// (e.g. a full-bleed photo or a black cover) is uniform but not blank.
#[cfg(test)]
const UNIFORM_MIN_MEAN: f64 = 160.0;

/// Map a sensitivity preset to the maximum dark-pixel fraction of a blank page.
/// Unknown values fall back to "normal". Preserved for M2 PDFium wiring; tested in `tests`.
#[cfg(test)]
pub(crate) fn threshold_for(sensitivity: &str) -> f64 {
    match sensitivity {
        "strict" => 0.0005,   // 0.05 % — only truly empty pages
        "aggressive" => 0.01, // 1 %    — also catches specks / punch holes
        _ => 0.003,           // 0.3 %  — "normal"
    }
}

/// Dark-pixel fraction, mean and standard deviation of grayscale pixels.
/// Preserved for M2 PDFium wiring; tested in `tests`.
#[cfg(test)]
pub(crate) fn luma_stats(pixels: &[u8]) -> (f64, f64, f64) {
    if pixels.is_empty() {
        return (0.0, 255.0, 0.0);
    }
    let n = pixels.len() as f64;
    let mut dark: u64 = 0;
    let mut sum: f64 = 0.0;
    for &p in pixels {
        if p < DARK_LUMA {
            dark += 1;
        }
        sum += p as f64;
    }
    let mean = sum / n;
    let var = pixels
        .iter()
        .map(|&p| {
            let d = p as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    (dark as f64 / n, mean, var.sqrt())
}

/// Blank-page decision from the page's luma statistics.
/// Preserved for M2 PDFium wiring; tested in `tests`.
#[cfg(test)]
pub(crate) fn is_blank(dark_fraction: f64, mean: f64, stddev: f64, threshold: f64) -> bool {
    dark_fraction < threshold || (stddev < UNIFORM_STDDEV && mean > UNIFORM_MIN_MEAN)
}

/// Detect blank pages of `input` (1-based page numbers).
///
/// **M1 stub**: The rasterisation step previously used Poppler's `pdftoppm`.
/// Returns `CapabilityUnavailable` unconditionally. The pixel-analysis helpers
/// (`luma_stats`, `is_blank`) remain and will be wired to PDFium in M2.
pub fn detect_blank_pages(
    _app: &tauri::AppHandle,
    _handle: &Arc<JobHandle>,
    _job_id: &str,
    _input: &str,
    _sensitivity: &str,
) -> Result<Vec<u32>, AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Blank page detection temporarily unavailable",
        "Blank page detection has been disabled during the M1 architecture migration. \
         The pipeline that previously relied on Poppler (pdftoppm) for rasterisation \
         has been removed. A PDFium-based replacement is planned for M2.",
    ))
}

#[cfg(test)]
mod tests {
    use super::{is_blank, luma_stats, threshold_for};

    #[test]
    fn thresholds_match_presets() {
        assert_eq!(threshold_for("strict"), 0.0005);
        assert_eq!(threshold_for("normal"), 0.003);
        assert_eq!(threshold_for("aggressive"), 0.01);
        assert_eq!(threshold_for("anything else"), 0.003);
    }

    #[test]
    fn pure_white_page_is_blank() {
        let pixels = vec![255u8; 10_000];
        let (dark, mean, stddev) = luma_stats(&pixels);
        assert_eq!(dark, 0.0);
        assert!(is_blank(dark, mean, stddev, threshold_for("strict")));
    }

    #[test]
    fn page_with_text_is_not_blank() {
        // 2% dark pixels — above every preset threshold, plenty of variance.
        let mut pixels = vec![255u8; 10_000];
        for p in pixels.iter_mut().take(200) {
            *p = 0;
        }
        let (dark, mean, stddev) = luma_stats(&pixels);
        assert!((dark - 0.02).abs() < 1e-9);
        assert!(!is_blank(dark, mean, stddev, threshold_for("aggressive")));
    }

    #[test]
    fn uniform_scanner_gray_is_blank() {
        // Flat light gray: every pixel counts as "dark" (215 < 240) so the
        // fraction rule alone would keep it — the stddev rule must catch it.
        let pixels = vec![215u8; 10_000];
        let (dark, mean, stddev) = luma_stats(&pixels);
        assert_eq!(dark, 1.0);
        assert!(is_blank(dark, mean, stddev, threshold_for("normal")));
    }

    #[test]
    fn uniform_dark_page_is_not_blank() {
        // A solid black page is uniform but must never be called blank.
        let pixels = vec![10u8; 10_000];
        let (dark, mean, stddev) = luma_stats(&pixels);
        assert!(!is_blank(dark, mean, stddev, threshold_for("aggressive")));
    }

    #[test]
    fn sensitivity_ordering_holds() {
        // A page with 0.5% dark pixels: blank for aggressive, kept for strict.
        let mut pixels = vec![255u8; 10_000];
        for p in pixels.iter_mut().take(50) {
            *p = 0;
        }
        let (dark, mean, stddev) = luma_stats(&pixels);
        assert!(is_blank(dark, mean, stddev, threshold_for("aggressive")));
        assert!(!is_blank(dark, mean, stddev, threshold_for("strict")));
    }
}
