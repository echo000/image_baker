use texture_smith_core::PorterImage;
use std::path::PathBuf;
use std::sync::Arc;

/// A self-contained droppable image slot component
#[derive(Clone)]
pub struct DroppableImageSlot {
    /// Label for this slot
    pub label: String,
    /// The currently loaded image (if any)
    pub image: Option<Arc<PorterImage>>,
    /// The path the image was loaded from (if any)
    pub path: Option<PathBuf>,
}

impl DroppableImageSlot {
    /// Create a new droppable image slot
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            image: None,
            path: None,
        }
    }

    /// Loads an image into this slot along with its source path
    pub fn load_image(&mut self, image: PorterImage, path: Option<PathBuf>) {
        self.image = Some(Arc::new(image));
        self.path = path;
    }

    /// Clear the image from this slot
    pub fn clear(&mut self) {
        self.image = None;
        self.path = None;
    }

    /// Get the directory containing the image file, if a path is set
    pub fn get_directory(&self) -> Option<PathBuf> {
        self.path
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    }
}
