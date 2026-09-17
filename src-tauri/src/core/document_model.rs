use serde::{Deserialize, Serialize};

/// Represents the identity of a document inside Vision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentId(pub String);

/// Minimal page descriptor holding basic dimensions. 
/// We do not load the full PDF objects here, just the info needed for the frontend to render the canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDescriptor {
    pub width: f32,
    pub height: f32,
    pub rotation: i32,
}

/// Logical metadata extracted from the document.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub pages_count: usize,
}

/// The core VisionDocument model.
/// This structure holds the LOGICAL state of the document. It does NOT contain the actual PDF bytes or FPDF_DOCUMENT handles.
/// The engines (like qpdf or PDFium) are invoked asynchronously using the `source_path` and `dirty_state`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionDocument {
    pub id: DocumentId,
    pub source_path: String,
    pub metadata: DocumentMetadata,
    
    // We only load page descriptors on demand or when strictly necessary.
    pub pages: Vec<PageDescriptor>,

    // Indicates if the document has modifications that are not yet saved to disk.
    pub is_dirty: bool,
    
    // Future expansion: pending logical operations (crop, rotate, delete page, etc.)
    // pub pending_operations: Vec<LogicalOperation>,
}

impl VisionDocument {
    /// Creates a new logical representation of a document.
    pub fn new(id: String, path: String) -> Self {
        Self {
            id: DocumentId(id),
            source_path: path,
            metadata: DocumentMetadata::default(),
            pages: Vec::new(),
            is_dirty: false,
        }
    }
}
