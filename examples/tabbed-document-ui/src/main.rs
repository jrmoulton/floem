use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use slotmap::{DefaultKey, SlotMap};
use floem::action::open_file;
use floem::event::Event;
use floem::file::{FileDialogOptions, FileSpec};
use floem::IntoView;
use floem::peniko::Color;
use floem::reactive::{create_rw_signal, provide_context, RwSignal, SignalGet, SignalUpdate, SignalWith, use_context};
use floem::views::{button, Decorators, dyn_container, dyn_stack, dyn_view, empty, h_stack, label, v_stack};
use crate::config::Config;
use crate::documents::image::ImageDocument;
use crate::documents::text::TextDocument;

pub mod documents {
    pub mod text {
        use std::fs;
        use std::path::PathBuf;

        pub struct TextDocument {
            path: PathBuf,
            content: String,
            selection: (usize, usize),
        }

        impl TextDocument {
            pub fn new(path: PathBuf) -> Self {

                let content = fs::read_to_string(&path).unwrap();

                Self {
                    path,
                    content,
                    selection: (0, 0),
                }
            }
        }
    }

    pub mod image {
        use std::path::PathBuf;

        pub struct ImageDocument {
            path: PathBuf,
            // TODO content: ImageContent(...)
            coordinate: Option<(usize, usize)>
        }

        impl ImageDocument {
            pub fn new(path: PathBuf) -> Self {
                Self {
                    path,
                    coordinate: None,
                }
            }
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
    document_key: DefaultKey,
}

#[derive(Clone)]
enum TabKind {
    Home(HomeTab),
    Document(DocumentTab),
}

struct ApplicationState {
    documents: RwSignal<SlotMap<DefaultKey, DocumentKind>>,

    tabs: RwSignal<SlotMap<DefaultKey, TabKind>>,

    active_tab: RwSignal<Option<DefaultKey>>,

    config: Config,
}

fn add_home_pressed(_event: &Event) {
    println!("Add home pressed");

    let app_state: Arc<ApplicationState> = use_context().unwrap();

    app_state.tabs.update(|tabs|{
        tabs.insert(
            TabKind::Home(HomeTab {})
        );
    });
}

fn new_pressed(_event: &Event) {
    println!("New pressed");
}

fn open_pressed(_event: &Event) {
    println!("Open pressed");

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
                }
            ]),
        move |file_info| {
            if let Some(file) = file_info {
                println!("Selected file: {:?}", file.path);


                let app_state: Arc<ApplicationState> = use_context().unwrap();

                let path = file.path.first().unwrap();

                let document = match path.extension().unwrap().to_str().unwrap() {
                    "txt" => {
                        let text_document = TextDocument::new(path.clone());

                        DocumentKind::TextDocument(text_document)
                    },
                    "bmp" => {
                        let image_document = ImageDocument::new(path.clone());

                        DocumentKind::ImageDocument(image_document)
                    },
                    _ => unreachable!()
                };

                let document_key= app_state.documents.try_update(|documents|{
                    documents.insert(document)
                }).unwrap();

                app_state.tabs.update(|tabs|{
                    let tab_key = tabs.insert(
                        TabKind::Document(DocumentTab { document_key })
                    );

                    app_state.active_tab.set(Some(tab_key));
                });
            }
        },
    );
}

fn app_view() -> impl IntoView {

    v_stack((
        //
        // Toolbar
        //
        h_stack((
            button(||"Add home").on_click_stop(add_home_pressed),
            button(||"New").on_click_stop(new_pressed),
            button(||"Open").on_click_stop(open_pressed),
        ))
            .style(|s| s
                .width_full()
                .background(Color::parse("#eeeeee").unwrap())
            ),

        //
        // Tab bar
        //
        dyn_stack(
            move || {
                let app_state: Option<Arc<ApplicationState>> = use_context();

                app_state.unwrap().tabs.get()
            },
            move |(tab_key, _tab_kind)| tab_key.clone(),
            move |(tab_key, tab_kind)| {
                println!("adding tab. tab_id: {:?}", tab_key);

                match tab_kind {
                    TabKind::Home(_home_tab) => {
                        button(||"Home")
                            .on_click_stop(move |_event|{
                                println!("Home tab pressed");
                                let app_state: Option<Arc<ApplicationState>> = use_context();
                                app_state.unwrap().active_tab.set(Some(tab_key))
                            }).into_any()

                    }
                    TabKind::Document(_document_tab) => {
                        button(||"Document")
                            .on_click_stop(move |_event|{
                                println!("Document tab pressed");
                                let app_state: Option<Arc<ApplicationState>> = use_context();
                                app_state.unwrap().active_tab.set(Some(tab_key))
                            }).into_any()
                    }
                }
            }
        ),
        //
        // Content
        //
        dyn_container(
            move || {
                let app_state: Option<Arc<ApplicationState>> = use_context();
                app_state.unwrap().active_tab.get()
            },
            move |active_tab| {
                let app_state: Arc<ApplicationState> = use_context().unwrap();
                if let Some(tab_key) = active_tab {
                    println!("displaying tab. tab_id: {:?}", &tab_key);

                    let tabs_signal = app_state.tabs;
                    tabs_signal.with_untracked(|tabs|{
                        let tab = tabs.get(tab_key).unwrap().clone();

                        match tab {
                            TabKind::Home(_) => {
                                v_stack((
                                    label(|| "Home Tab Content"),
                                    dyn_view(move ||format!("tab_id: {:?}", &tab_key))
                                )).into_any()
                            }
                            TabKind::Document(document_tab) => {
                                dyn_container(move ||{
                                    document_tab.document_key
                                },
                                move |document_key|{
                                    app_state.documents.with(|documents| {
                                        let document_kind = documents.get(document_key).unwrap();
                                        match document_kind {
                                            DocumentKind::TextDocument(_) => {
                                                label(|| "Text document").into_any()
                                            },
                                            DocumentKind::ImageDocument(_) => {
                                                label(|| "Image document").into_any()
                                            },
                                        }
                                    })
                                }).into_any()
                            }
                        }
                    })
                } else {
                    empty().into_any()
                }
            }
        ),

    ))
        .style(|s| s
            .width_full()
            .height_full()
            .background(Color::LIGHT_GRAY)
        )
}

pub mod config {

    #[derive(Default, serde::Serialize, serde::Deserialize)]
    pub struct Config {
        pub show_home_on_startup: bool,
    }
}

fn main() {

    let file = File::open(PathBuf::from("config.json"));
    let config: Config = match file {
        Ok(file) => {
            serde_json::from_reader(file).unwrap()
        }
        Err(_) => {
            Config::default()
        }
    };

    let app_state = ApplicationState {
        documents: create_rw_signal(Default::default()),
        tabs: create_rw_signal(Default::default()),
        active_tab: create_rw_signal(None),
        config,
    };

    if app_state.config.show_home_on_startup {
        app_state.tabs.update(|tabs|{
            let tab_key = tabs.insert(
                TabKind::Home(HomeTab {})
            );

            app_state.active_tab.set(Some(tab_key));
        })
    }

    let app_state_arc = Arc::new(app_state);

    provide_context(app_state_arc.clone());

    floem::launch(app_view);

    let content: String = serde_json::to_string(&app_state_arc.config).unwrap();

    let mut file = File::create(PathBuf::from("config.json")).unwrap();
    file.write(content.as_bytes()).unwrap();
}
