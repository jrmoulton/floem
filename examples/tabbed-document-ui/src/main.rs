use crate::config::Config;
use crate::documents::image::ImageDocument;
use crate::documents::text::TextDocument;
use crate::documents::DocumentKind;
use crate::tabs::document::DocumentTab;
use crate::tabs::home::HomeTab;
use crate::tabs::TabKind;
use floem::action::open_file;
use floem::file::{FileDialogOptions, FileInfo, FileSpec};
use floem::peniko::Color;
use floem::reactive::*;
use floem::views::*;
use floem::IntoView;
use slotmap::{new_key_type, SlotMap};
use std::sync::Arc;

pub mod config;
pub mod documents;
pub mod tabs;

fn main() {
    let config = config::load();

    let app_state = ApplicationState {
        documents: create_rw_signal(Default::default()),
        tabs: create_rw_signal(Default::default()),
        active_tab: create_rw_signal(None),
        config,
    };

    if app_state.config.show_home_on_startup {
        show_home_tab(&app_state);
    }

    let app_state_arc = Arc::new(app_state);

    provide_context(app_state_arc.clone());

    floem::launch(app_view);

    config::save(&app_state_arc.config);
}
new_key_type! {
    pub struct DocKey;
}
new_key_type! {
    pub struct TabKey;
}

struct ApplicationState {
    documents: RwSignal<SlotMap<DocKey, DocumentKind>>,
    tabs: RwSignal<SlotMap<TabKey, TabKind>>,
    active_tab: RwSignal<Option<TabKey>>,
    config: Config,
}

fn app_view() -> impl IntoView {
    let toolbar = h_stack((
        button("Add home").action(add_home_pressed),
        button("New").action(new_pressed),
        button("Open").action(open_pressed),
    ))
    .style(|s| s.width_full().background(Color::parse("#eeeeee").unwrap()));

    let tab_bar = dyn_stack(
        move || {
            let app_state: Option<Arc<ApplicationState>> = use_context();

            app_state.unwrap().tabs.get()
        },
        move |(tab_key, _tab_kind)| tab_key.clone(),
        move |(tab_key, tab_kind)| {
            println!("adding tab. tab_id: {:?}", tab_key);

            match tab_kind {
                TabKind::Home(_home_tab) => button("Home")
                    .action(move || {
                        println!("Home tab pressed");
                        let app_state: Option<Arc<ApplicationState>> = use_context();
                        app_state.unwrap().active_tab.set(Some(tab_key))
                    })
                    .into_any(),
                TabKind::Document(_document_tab) => button("Document")
                    .action(move || {
                        println!("Document tab pressed");
                        let app_state: Option<Arc<ApplicationState>> = use_context();
                        app_state.unwrap().active_tab.set(Some(tab_key))
                    })
                    .into_any(),
            }
        },
    )
    .style(|s| s.width_full().background(Color::parse("#dddddd").unwrap()));

    let content = tab(
        move || {
            let app_state: Arc<ApplicationState> = use_context().unwrap();
            let Some(active_tab) = app_state.active_tab.get() else {
                return 0;
            };
            app_state.tabs.with_untracked(|tabs| {
                for (idx, tab_key) in tabs.keys().enumerate() {
                    if tab_key == active_tab {
                        return idx;
                    }
                }
                0
            })
        },
        move || {
            let app_state: Arc<ApplicationState> = use_context().unwrap();
            app_state.tabs.get()
        },
        move |(tab_key, _tabkind)| tab_key.clone(),
        move |(tab_key, tab_kind)| {
            let app_state: Arc<ApplicationState> = use_context().unwrap();
            println!("displaying tab. tab_id: {:?}", &tab_key);

            match tab_kind {
                TabKind::Home(_home_tab) => "Home Tab".into_any(),
                TabKind::Document(doc_tab) => {
                    let documents_signal = app_state.documents;
                    let mut document = None;
                    documents_signal
                        .try_update(|docs| {
                            // because this tab view is keyed and tab state is maintained even when not in view,
                            // this docs.remove will only run the first time that this view is built so this is fine to do
                            document = docs.remove(doc_tab.document_key);
                        })
                        .expect("this failed");

                    match document.unwrap() {
                        DocumentKind::TextDocument(text_doc) => text_doc.build_view().into_any(),
                        DocumentKind::ImageDocument(img_doc) => img_doc.build_view().into_any(),
                    }
                }
            }
        },
    )
    .style(|s| s.width_full().height_full().background(Color::DIM_GRAY));

    (toolbar, tab_bar, content)
        .v_stack()
        .style(|s| s.width_full().height_full().background(Color::LIGHT_GRAY))
}

fn add_home_pressed() {
    println!("Add home pressed");

    let app_state: Arc<ApplicationState> = use_context().unwrap();

    app_state.tabs.update(|tabs| {
        tabs.insert(TabKind::Home(HomeTab {}));
    });
}

fn new_pressed() {
    println!("New pressed");
}

fn open_pressed() {
    println!("Open pressed");

    // changing the code in this way works because the signal that is used in the callback is created here on the main thread.
    // then in the effect (still on the main thread) the text document with its selection signal is created.
    let opened_file: RwSignal<Option<FileInfo>> = RwSignal::new(None);
    create_effect(move |_| {
        let Some(file) = opened_file.get() else {
            return;
        };
        println!("Selected file: {:?}", file.path);

        let app_state: Arc<ApplicationState> = use_context().unwrap();

        let path = file.path.first().unwrap();

        let document = match path.extension().unwrap().to_str().unwrap() {
            "txt" => {
                let text_document = TextDocument::new(path.clone());

                DocumentKind::TextDocument(text_document)
            }
            "bmp" => {
                let image_document = ImageDocument::new(path.clone());

                DocumentKind::ImageDocument(image_document)
            }
            _ => unreachable!(),
        };

        let document_key = app_state
            .documents
            .try_update(|documents| documents.insert(document))
            .unwrap();

        let tab_key = app_state
            .tabs
            .try_update(|tabs| tabs.insert(TabKind::Document(DocumentTab { document_key })))
            .unwrap();
        app_state.active_tab.set(Some(tab_key));
    });

    open_file(
        FileDialogOptions::new()
            .title("Select a file")
            .allowed_types(vec![
                FileSpec {
                    name: "text",
                    extensions: &["txt"],
                },
                FileSpec {
                    name: "image",
                    extensions: &["bmp"],
                },
            ]),
        move |file_info| {
            if let Some(file) = file_info {
                opened_file.set(Some(file));
            }
        },
    );
}

fn show_home_tab(app_state: &ApplicationState) {
    let tab_key = app_state
        .tabs
        .try_update(|tabs| tabs.insert(TabKind::Home(HomeTab {})))
        .unwrap();

    app_state.active_tab.set(Some(tab_key));
}
