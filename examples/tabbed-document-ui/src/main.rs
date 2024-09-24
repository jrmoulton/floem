use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use floem::{IntoView, View};
use floem::peniko::Color;
use floem::reactive::{create_rw_signal, provide_context, RwSignal, SignalGet, SignalUpdate, use_context};
use floem::view_tuple::ViewTuple;
use floem::views::{button, Decorators, dyn_stack, empty, h_stack, label, v_stack};
use crate::documents::image::ImageDocument;
use crate::documents::text::TextDocument;

pub mod documents {
    pub mod text {
        use std::path::PathBuf;

        pub struct TextDocument {
            path: PathBuf,
            content: String,
            selection: (usize, usize),
        }
    }

    pub mod image {
        use std::path::PathBuf;

        pub struct ImageDocument {
            path: PathBuf,
            // TODO content: ImageContent(...)
            coordinate: Option<(usize, usize)>
        }
    }
}

enum DocumentKind {
    TextDocument(TextDocument),
    ImageDocument(ImageDocument),
}

#[derive(Clone)]
struct HomeTab {
}

#[derive(Clone)]
struct DocumentTab {
    id: String,
}

#[derive(Clone)]
enum TabKind {
    Home(HomeTab),
    Document(DocumentTab),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TabId(String);

struct ApplicationState {
    documents: HashMap<String, DocumentKind>,

    tabs: RwSignal<HashMap<TabId, TabKind>>,

    show_home_on_startup: bool,
}

fn app_view() -> impl IntoView {

    v_stack((
        h_stack((
            button(||"New").on_click_stop(move |_event|{

                println!("New pressed");

                let app_state: Arc<ApplicationState> = use_context().unwrap();

                app_state.tabs.update(|tabs|{
                    tabs.insert(
                        TabId("home-tab-id".to_string()),
                        TabKind::Home(HomeTab {})
                    );
                });

            }),
            button(||"Open"),
        ))
            .style(|s| s
                .width_full()
                .background(Color::parse("#eeeeee").unwrap())
            ),
        dyn_stack(
            move || {
                let app_state: Option<Arc<ApplicationState>> = use_context();

                app_state.unwrap().tabs.get()
            },
            move |(tab_id, _tab_kind)| tab_id.clone(),
            move|(tab_id, tab_kind)| {
                match tab_kind {
                    TabKind::Home(_home_tab) => {
                        label(||"Home")
                    }
                    TabKind::Document(_document_tab) => {
                        label(||"Document")
                    }
                }
            }
        )
    ))
        .style(|s| s
            .width_full()
            .height_full()
            .background(Color::LIGHT_GRAY)
        )
}

fn main() {

    let mut app_state = ApplicationState {
        documents: Default::default(),
        tabs: create_rw_signal(Default::default()),
        show_home_on_startup: false,
    };

    provide_context(Arc::new(app_state));

    floem::launch(app_view);
}
