use image::ImageDocument;
use text::TextDocument;

pub mod text;
pub mod image;

pub enum DocumentKind {
    TextDocument(TextDocument),
    ImageDocument(ImageDocument),
}
