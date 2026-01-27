use crate::porter_image::PorterImage;
use std::sync::Arc;

/// A self-contained droppable image slot component
#[derive(Clone)]
pub struct DroppableImageSlot {
    /// Label for this slot
    pub label: String,
    /// The currently loaded image (if any)
    pub image: Option<Arc<PorterImage>>,
}

impl DroppableImageSlot {
    /// Create a new droppable image slot
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            image: None,
        }
    }

    /// Loads an image into this slot
    pub fn load_image(&mut self, image: PorterImage) {
        self.image = Some(Arc::new(image));
    }

    /// Clear the image from this slot
    pub fn clear(&mut self) {
        self.image = None;
    }
}
