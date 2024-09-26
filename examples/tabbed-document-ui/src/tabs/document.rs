use crate::DocKey;

#[derive(Clone)]
pub struct DocumentTab {
    pub document_key: DocKey,
}

pub struct DocumentContainer {}

// impl DocumentContainer {
//     pub fn build_view(Doc) -> impl View {
//         let app_state: Arc<ApplicationState> = use_context().unwrap();
//         app_state.documents.with(|documents| {
//             let document_kind = documents.get(document_key).unwrap();
//             match document_kind {
//                 DocumentKind::TextDocument(text_document) => text_document.build_view().into_any(),
//                 DocumentKind::ImageDocument(image_document) => {
//                     image_document.build_view().into_any()
//                 }
//             }
//         })
//     }
// }
