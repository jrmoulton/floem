use std::sync::Arc;
use slotmap::DefaultKey;
use floem::reactive::{SignalWith, use_context};
use floem::{IntoView, View};
use floem::views::dyn_container;
use crate::ApplicationState;
use crate::documents::DocumentKind;

#[derive(Clone)]
pub struct DocumentTab {
    pub document_key: DefaultKey,
}

pub struct DocumentContainer {}

impl DocumentContainer {
    pub fn build_view(document_key: DefaultKey) -> impl View {
        dyn_container(
        move || {
            document_key
        },
        move |document_key|{
            let app_state: Arc<ApplicationState> = use_context().unwrap();
            app_state.documents.with(|documents| {
                let document_kind = documents.get(document_key).unwrap();
                match document_kind {
                    DocumentKind::TextDocument(text_document) => {
                        text_document.build_view().into_any()
                    },
                    DocumentKind::ImageDocument(image_document) => {
                        image_document.build_view().into_any()
                    },
                }
            })
        })
    }
}