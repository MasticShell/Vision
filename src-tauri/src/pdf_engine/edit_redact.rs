//! Secure redaction: rasterize affected pages in place and verify removal.
//!
//! M1 status: `apply_redactions()` and `apply_redactions_with_app()` have been
//! stubbed and return `CapabilityUnavailable`. Without rasterisation, the
//! original PDF content streams remain intact — the redaction would be
//! structurally incomplete and insecure. This is a security constraint, not an
//! inconvenience. `verify_redaction()` is pure lopdf and remains functional.
//! The full pipeline will be wired to PDFium in M2.

use crate::error::AppError;
use crate::pdf_engine::edit_overlay::PdfRectIn;
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};
use std::collections::HashSet;
use std::path::Path;

pub struct RedactRegion {
    pub page_index: u32,
    pub rect: PdfRectIn,
    pub fill: Option<String>,
    pub label: Option<String>,
}

/// Apply redactions to `path` in place.
///
/// **M1 stub — SECURITY**: Without rasterisation (previously provided by
/// `pdftoppm`), the original PDF content streams remain intact. A partial
/// implementation would produce a PDF that *looks* redacted but still contains
/// the sensitive data in its structure. Therefore this function returns
/// `CapabilityUnavailable` unconditionally rather than performing an insecure
/// partial operation. The full pipeline will be wired to PDFium in M2.
pub fn apply_redactions(_path: &Path, _regions: &[RedactRegion]) -> Result<(), AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Secure redaction temporarily unavailable",
        "Redaction has been disabled during the M1 architecture migration. \
         Without rasterisation the original content streams remain in the PDF \
         and the redaction would be insecure. A PDFium-based replacement is planned for M2.",
    ))
}

/// Same as [`apply_redactions`] — M1 stub.
///
/// See [`apply_redactions`] for the security rationale.
pub(crate) fn apply_redactions_with_app(
    _app: &tauri::AppHandle,
    _path: &Path,
    _regions: &[RedactRegion],
) -> Result<(), AppError> {
    Err(AppError::new(
        "CapabilityUnavailable",
        "Secure redaction temporarily unavailable",
        "Redaction has been disabled during the M1 architecture migration. \
         Without rasterisation the original content streams remain in the PDF \
         and the redaction would be insecure. A PDFium-based replacement is planned for M2.",
    ))
}

/// Page-content probe still in dest streams → `Err` (fail closed).
/// Empty probes + leftover intersecting `/Annots` `/Contents` or field `/V`
/// → `Ok(warnings)`; the leftover string may remain.
pub fn verify_redaction(
    dest: &Path,
    page_content_probes: &[&[u8]],
    regions: &[RedactRegion],
) -> Result<Vec<String>, AppError> {
    if !dest.is_file() {
        return Err(AppError::invalid_pdf(&dest.to_string_lossy()));
    }
    let mut doc = Document::load(dest)
        .map_err(|e| AppError::engine_failed(format!("Could not read the PDF: {e}")))?;
    let _ = doc.decompress();

    let pages = doc.get_pages();
    for region in regions {
        if pages.get(&(region.page_index + 1)).is_none() {
            return Err(redaction_page_missing(region.page_index));
        }
    }

    if !page_content_probes.is_empty() {
        if probe_remains_on_redacted_pages(&doc, page_content_probes, regions) {
            return Err(redaction_incomplete());
        }
    }

    let mut warnings = Vec::new();
    warn_intersecting_annots(&doc, regions, &mut warnings);
    warn_intersecting_fields(&doc, regions, &mut warnings);
    warn_leftover_thumbs_and_struct(&doc, regions, &mut warnings);
    if has_attachments(&doc) {
        warnings.push(
            "This PDF has attachments that were not removed. They may still hold sensitive data."
                .into(),
        );
    }
    Ok(warnings)
}

fn redaction_page_missing(page_index: u32) -> AppError {
    AppError::new(
        "REDACTION_INCOMPLETE",
        "Redaction could not be verified",
        format!(
            "A redaction region targets page {} which is not in the PDF. The file was not saved.",
            page_index + 1
        ),
    )
}

fn redaction_incomplete() -> AppError {
    AppError::new(
        "REDACTION_INCOMPLETE",
        "Redaction could not be verified",
        "Page content that should have been removed is still in the PDF. The file was not saved.",
    )
}

fn unique_redact_pages(regions: &[RedactRegion]) -> Vec<u32> {
    let mut seen = HashSet::new();
    let mut pages = Vec::new();
    for region in regions {
        if seen.insert(region.page_index) {
            pages.push(region.page_index);
        }
    }
    pages
}

fn haystack_contains(haystack: &[u8], probe: &[u8]) -> bool {
    !probe.is_empty() && haystack.windows(probe.len()).any(|w| w == probe)
}

/// Search each probe in every redacted-page dest haystack.
/// Unredacted siblings are not scanned (identical leftover text must not fail-close).
fn probe_remains_on_redacted_pages(
    doc: &Document,
    probes: &[&[u8]],
    regions: &[RedactRegion],
) -> bool {
    let pages = unique_redact_pages(regions);
    if pages.is_empty() {
        let haystack = page_content_haystack(doc);
        return probes.iter().any(|p| haystack_contains(&haystack, p));
    }
    let haystacks: Vec<Vec<u8>> = pages
        .iter()
        .map(|&page_index| page_content_haystack_for(doc, page_index))
        .collect();
    probes
        .iter()
        .any(|probe| haystacks.iter().any(|h| haystack_contains(h, probe)))
}

fn page_content_haystack(doc: &Document) -> Vec<u8> {
    let mut out = Vec::new();
    for &page_id in doc.get_pages().values() {
        append_page_streams(doc, page_id, &mut out);
    }
    out
}

fn page_content_haystack_for(doc: &Document, page_index: u32) -> Vec<u8> {
    let Some(&page_id) = doc.get_pages().get(&(page_index + 1)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    append_page_streams(doc, page_id, &mut out);
    out
}

fn append_page_streams(doc: &Document, page_id: ObjectId, out: &mut Vec<u8>) {
    if let Ok(bytes) = doc.get_page_content(page_id) {
        out.extend_from_slice(&bytes);
        out.push(b'\n');
    }
    for id in doc.get_page_contents(page_id) {
        if let Ok(stream) = doc.get_object(id).and_then(Object::as_stream) {
            out.extend_from_slice(&plain_stream(stream));
            out.push(b'\n');
        }
    }
    append_page_xobjects(doc, page_id, out);
}

fn append_page_xobjects(doc: &Document, page_id: ObjectId, out: &mut Vec<u8>) {
    let mut seen = HashSet::new();
    for resources in ancestor_resource_dicts(doc, page_id) {
        append_xobject_streams(doc, resources, out, &mut seen);
    }
}

fn ancestor_resource_dicts<'a>(doc: &'a Document, start: ObjectId) -> Vec<&'a Dictionary> {
    let mut dicts = Vec::new();
    let mut current = Some(start);
    let mut seen = HashSet::new();
    while let Some(id) = current {
        if !seen.insert(id) {
            break;
        }
        let Ok(node) = doc.get_dictionary(id) else {
            break;
        };
        if let Ok(res) = node.get(b"Resources") {
            if let Some(d) = dict_from(doc, res) {
                dicts.push(d);
            }
        }
        current = node.get(b"Parent").ok().and_then(|o| o.as_reference().ok());
    }
    dicts
}

fn append_xobject_streams(
    doc: &Document,
    resources: &Dictionary,
    out: &mut Vec<u8>,
    seen: &mut HashSet<ObjectId>,
) {
    let Ok(xo) = resources.get(b"XObject") else {
        return;
    };
    let Some(xobjects) = dict_from(doc, xo) else {
        return;
    };
    for (_, obj) in xobjects.iter() {
        let Ok(id) = obj.as_reference() else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let Ok(stream) = doc.get_object(id).and_then(Object::as_stream) else {
            continue;
        };
        out.extend_from_slice(&plain_stream(stream));
        out.push(b'\n');
        if stream
            .dict
            .get(b"Subtype")
            .ok()
            .and_then(|o| o.as_name().ok())
            == Some(b"Form")
        {
            if let Ok(res) = stream.dict.get(b"Resources") {
                if let Some(d) = dict_from(doc, res) {
                    append_xobject_streams(doc, d, out, seen);
                }
            }
        }
    }
}

/// Page Contents plus decoded Form/Image XObject streams (page-local and inherited).
pub(crate) fn collect_redact_probes_for_pages(
    doc: &Document,
    regions: &[RedactRegion],
) -> Result<Vec<Vec<u8>>, AppError> {
    let pages = doc.get_pages();
    let mut seen = HashSet::new();
    let mut probes = Vec::new();
    for region in regions {
        if !seen.insert(region.page_index) {
            continue;
        }
        let Some(&id) = pages.get(&(region.page_index + 1)) else {
            return Err(redaction_page_missing(region.page_index));
        };
        if let Ok(bytes) = doc.get_page_content(id) {
            if !bytes.is_empty() {
                probes.push(bytes);
            }
        }
        collect_xobject_probes(doc, id, &mut probes);
    }
    Ok(probes)
}

fn collect_xobject_probes(doc: &Document, page_id: ObjectId, probes: &mut Vec<Vec<u8>>) {
    let mut seen = HashSet::new();
    for resources in ancestor_resource_dicts(doc, page_id) {
        collect_xobject_probes_from_dict(doc, resources, probes, &mut seen);
    }
}

fn collect_xobject_probes_from_dict(
    doc: &Document,
    resources: &Dictionary,
    probes: &mut Vec<Vec<u8>>,
    seen: &mut HashSet<ObjectId>,
) {
    let Ok(xo) = resources.get(b"XObject") else {
        return;
    };
    let Some(xobjects) = dict_from(doc, xo) else {
        return;
    };
    for (_, obj) in xobjects.iter() {
        let Ok(id) = obj.as_reference() else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let Ok(stream) = doc.get_object(id).and_then(Object::as_stream) else {
            continue;
        };
        let subtype = stream
            .dict
            .get(b"Subtype")
            .ok()
            .and_then(|o| o.as_name().ok());
        if subtype != Some(b"Form") && subtype != Some(b"Image") {
            continue;
        }
        let bytes = plain_stream(stream);
        if !bytes.is_empty() {
            probes.push(bytes);
        }
        if subtype == Some(b"Form") {
            if let Ok(res) = stream.dict.get(b"Resources") {
                if let Some(d) = dict_from(doc, res) {
                    collect_xobject_probes_from_dict(doc, d, probes, seen);
                }
            }
        }
    }
}

fn dict_from<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Dictionary> {
    match obj {
        Object::Dictionary(d) => Some(d),
        Object::Reference(id) => doc.get_dictionary(*id).ok(),
        _ => None,
    }
}

fn plain_stream(stream: &Stream) -> Vec<u8> {
    stream
        .get_plain_content()
        .or_else(|_| stream.decompressed_content())
        .unwrap_or_else(|_| stream.content.clone())
}

fn warn_intersecting_annots(doc: &Document, regions: &[RedactRegion], warnings: &mut Vec<String>) {
    for (page_no, page_id) in doc.get_pages() {
        let page_index = page_no.saturating_sub(1);
        let page_regions: Vec<&RedactRegion> = regions
            .iter()
            .filter(|r| r.page_index == page_index)
            .collect();
        if page_regions.is_empty() {
            continue;
        }
        for annot_id in page_annot_ids(doc, page_id) {
            let Ok(annot) = doc.get_dictionary(annot_id) else {
                continue;
            };
            let Some(contents) = dict_string(doc, annot, b"Contents") else {
                continue;
            };
            if contents.trim().is_empty() {
                continue;
            }
            let Some(rect) = dict_rect(doc, annot, b"Rect") else {
                continue;
            };
            if page_regions.iter().any(|r| rects_intersect(rect, &r.rect)) {
                warnings.push(format!(
                    "An annotation on page {page_no} still contains text that intersects a redaction region."
                ));
            }
        }
    }
}

fn warn_intersecting_fields(doc: &Document, regions: &[RedactRegion], warnings: &mut Vec<String>) {
    let mut warned: HashSet<ObjectId> = HashSet::new();
    for (page_no, page_id) in doc.get_pages() {
        let page_index = page_no.saturating_sub(1);
        let page_regions: Vec<&RedactRegion> = regions
            .iter()
            .filter(|r| r.page_index == page_index)
            .collect();
        if page_regions.is_empty() {
            continue;
        }
        for annot_id in page_annot_ids(doc, page_id) {
            let Ok(annot) = doc.get_dictionary(annot_id) else {
                continue;
            };
            if !is_widget(annot) {
                continue;
            }
            let Some(value) = field_value(doc, annot_id) else {
                continue;
            };
            if value.trim().is_empty() {
                continue;
            }
            let Some(rect) = dict_rect(doc, annot, b"Rect") else {
                continue;
            };
            if page_regions.iter().any(|r| rects_intersect(rect, &r.rect)) {
                warned.insert(annot_id);
                warnings.push(format!(
                    "A form field on page {page_no} still holds a value that intersects a redaction region."
                ));
            }
        }
    }
    // After flatten-before-burn the widget is gone from /Annots; leftover /V
    // still lives on catalog /AcroForm /Fields (and parents).
    for field_id in acroform_field_ids(doc) {
        if !warned.insert(field_id) {
            continue;
        }
        let Some(value) = field_value(doc, field_id) else {
            continue;
        };
        if value.trim().is_empty() {
            continue;
        }
        if let Some(page_no) = field_intersects_regions(doc, field_id, regions) {
            warnings.push(format!(
                "A form field on page {page_no} still holds a value that intersects a redaction region."
            ));
        }
    }
}

fn acroform_field_ids(doc: &Document) -> Vec<ObjectId> {
    let mut out = Vec::new();
    let Ok(root) = doc.trailer.get(b"Root").and_then(Object::as_reference) else {
        return out;
    };
    let Ok(catalog) = doc.get_dictionary(root) else {
        return out;
    };
    let Ok(acro_obj) = catalog.get(b"AcroForm") else {
        return out;
    };
    let Some(acro) = dict_from(doc, acro_obj) else {
        return out;
    };
    let Ok(fields) = acro.get(b"Fields") else {
        return out;
    };
    let resolved = match fields {
        Object::Reference(id) => doc.get_object(*id).ok(),
        other => Some(other),
    };
    let Some(Object::Array(items)) = resolved else {
        return out;
    };
    let mut stack: Vec<ObjectId> = items.iter().filter_map(|o| o.as_reference().ok()).collect();
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        out.push(id);
        if let Ok(dict) = doc.get_dictionary(id) {
            if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
                for kid in kids {
                    if let Ok(kid_id) = kid.as_reference() {
                        stack.push(kid_id);
                    }
                }
            }
        }
    }
    out
}

fn field_intersects_regions(
    doc: &Document,
    field_id: ObjectId,
    regions: &[RedactRegion],
) -> Option<u32> {
    let mut rects = Vec::new();
    collect_field_widget_rects(doc, field_id, &mut rects, &mut HashSet::new());
    for (page_index, rect) in rects {
        for region in regions {
            if page_index != u32::MAX && region.page_index != page_index {
                continue;
            }
            if rects_intersect(rect, &region.rect) {
                return Some(if page_index == u32::MAX {
                    region.page_index + 1
                } else {
                    page_index + 1
                });
            }
        }
    }
    None
}

fn collect_field_widget_rects(
    doc: &Document,
    id: ObjectId,
    out: &mut Vec<(u32, [f64; 4])>,
    seen: &mut HashSet<ObjectId>,
) {
    if !seen.insert(id) {
        return;
    }
    let Ok(dict) = doc.get_dictionary(id) else {
        return;
    };
    if let Some(rect) = dict_rect(doc, dict, b"Rect") {
        let page_index = dict
            .get(b"P")
            .ok()
            .and_then(|o| o.as_reference().ok())
            .and_then(|pid| page_index_of(doc, pid))
            .unwrap_or(u32::MAX);
        out.push((page_index, rect));
    }
    if let Ok(Object::Array(kids)) = dict.get(b"Kids") {
        for kid in kids {
            if let Ok(kid_id) = kid.as_reference() {
                collect_field_widget_rects(doc, kid_id, out, seen);
            }
        }
    }
}

fn page_index_of(doc: &Document, page_id: ObjectId) -> Option<u32> {
    doc.get_pages()
        .iter()
        .find(|(_, &id)| id == page_id)
        .map(|(&page_no, _)| page_no.saturating_sub(1))
}

fn is_widget(annot: &Dictionary) -> bool {
    if annot
        .get(b"Subtype")
        .ok()
        .and_then(|o| o.as_name().ok())
        .is_some_and(|n| n == b"Widget")
    {
        return true;
    }
    annot.get(b"FT").is_ok() || annot.get(b"Parent").is_ok()
}

fn field_value(doc: &Document, mut id: ObjectId) -> Option<String> {
    for _ in 0..16 {
        let dict = doc.get_dictionary(id).ok()?;
        if let Some(v) = dict_string(doc, dict, b"V") {
            return Some(v);
        }
        id = dict.get(b"Parent").ok()?.as_reference().ok()?;
    }
    None
}

fn page_annot_ids(doc: &Document, page_id: ObjectId) -> Vec<ObjectId> {
    let Ok(page) = doc.get_dictionary(page_id) else {
        return Vec::new();
    };
    let Ok(annots) = page.get(b"Annots") else {
        return Vec::new();
    };
    let resolved = match annots {
        Object::Reference(id) => doc.get_object(*id).ok(),
        other => Some(other),
    };
    match resolved {
        Some(Object::Array(items)) => items.iter().filter_map(|o| o.as_reference().ok()).collect(),
        Some(Object::Reference(id)) => vec![*id],
        _ => Vec::new(),
    }
}

fn warn_leftover_thumbs_and_struct(
    doc: &Document,
    regions: &[RedactRegion],
    warnings: &mut Vec<String>,
) {
    let pages = doc.get_pages();
    let mut thumb = false;
    let mut seen = HashSet::new();
    for region in regions {
        if !seen.insert(region.page_index) {
            continue;
        }
        let Some(&page_id) = pages.get(&(region.page_index + 1)) else {
            continue;
        };
        let Ok(page) = doc.get_dictionary(page_id) else {
            continue;
        };
        if page.get(b"Thumb").is_ok() {
            thumb = true;
            break;
        }
    }
    if thumb {
        warnings.push(
            "A redacted page still has a thumbnail (/Thumb) that was not removed. It may still hold sensitive data."
                .into(),
        );
    }
    if has_struct_tree_root(doc) {
        warnings.push(
            "This PDF has a structure tree (/StructTreeRoot) that was not removed. It may still hold sensitive data."
                .into(),
        );
    }
}

fn has_struct_tree_root(doc: &Document) -> bool {
    let Ok(root) = doc.trailer.get(b"Root").and_then(Object::as_reference) else {
        return false;
    };
    let Ok(catalog) = doc.get_dictionary(root) else {
        return false;
    };
    catalog.get(b"StructTreeRoot").is_ok()
}

fn has_attachments(doc: &Document) -> bool {
    if catalog_has_attachments(doc) {
        return true;
    }
    for (_, page_id) in doc.get_pages() {
        for annot_id in page_annot_ids(doc, page_id) {
            let Ok(annot) = doc.get_dictionary(annot_id) else {
                continue;
            };
            if is_file_attachment(doc, annot) {
                return true;
            }
        }
    }
    false
}

fn catalog_has_attachments(doc: &Document) -> bool {
    let Ok(root) = doc.trailer.get(b"Root").and_then(Object::as_reference) else {
        return false;
    };
    let Ok(catalog) = doc.get_dictionary(root) else {
        return false;
    };
    if catalog.get(b"AF").is_ok() {
        return true;
    }
    let Ok(names) = catalog.get(b"Names") else {
        return false;
    };
    let Some(names) = dict_from(doc, names) else {
        return false;
    };
    names.get(b"EmbeddedFiles").is_ok()
}

fn is_file_attachment(doc: &Document, annot: &Dictionary) -> bool {
    if annot
        .get(b"Subtype")
        .ok()
        .and_then(|o| o.as_name().ok())
        .is_some_and(|n| n == b"FileAttachment")
    {
        return true;
    }
    let Ok(fs_obj) = annot.get(b"FS") else {
        return false;
    };
    let Some(fs) = dict_from(doc, fs_obj) else {
        return false;
    };
    fs.get(b"EF").is_ok()
}

fn dict_string(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<String> {
    let obj = resolve_obj(doc, dict.get(key).ok()?)?;
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).into_owned()),
        Object::Name(name) => Some(String::from_utf8_lossy(name).into_owned()),
        _ => None,
    }
}

fn dict_rect(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<[f64; 4]> {
    let obj = resolve_obj(doc, dict.get(key).ok()?)?;
    let arr = obj.as_array().ok()?;
    if arr.len() != 4 {
        return None;
    }
    let mut v = [0.0; 4];
    for (i, item) in arr.iter().enumerate() {
        v[i] = num(doc, item)?;
    }
    Some([
        v[0].min(v[2]),
        v[1].min(v[3]),
        v[0].max(v[2]),
        v[1].max(v[3]),
    ])
}

fn resolve_obj<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Object> {
    match obj {
        Object::Reference(id) => doc.get_object(*id).ok(),
        other => Some(other),
    }
}

fn num(doc: &Document, obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(i) => Some(*i as f64),
        Object::Real(r) => Some(*r as f64),
        Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| num(doc, o)),
        _ => None,
    }
}

fn rects_intersect(annot: [f64; 4], region: &PdfRectIn) -> bool {
    let rx0 = region.x;
    let ry0 = region.y;
    let rx1 = region.x + region.w;
    let ry1 = region.y + region.h;
    annot[0] < rx1 && annot[2] > rx0 && annot[1] < ry1 && annot[3] > ry0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_redactions_returns_capability_unavailable() {
        let res = apply_redactions(Path::new("dummy.pdf"), &[]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "CapabilityUnavailable");
    }
}
