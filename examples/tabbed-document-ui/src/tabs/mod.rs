use crate::tabs::document::DocumentTab;
use crate::tabs::home::HomeTab;

pub mod home;
pub mod document;

#[derive(Clone)]
pub enum TabKind {
    Home(HomeTab),
    Document(DocumentTab),
}
