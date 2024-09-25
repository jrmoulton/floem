use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use slotmap::{DefaultKey, SlotMap};
use floem::IntoView;
use floem::peniko::Color;
use floem::reactive::{create_rw_signal, provide_context, RwSignal, SignalGet, SignalUpdate, use_context};
use floem::views::{button, ButtonClass, Decorators, dyn_container, dyn_stack, dyn_view, empty, h_stack, label, v_stack};
use crate::config::Config;
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

struct ApplicationState {
    documents: HashMap<String, DocumentKind>,

    tabs: RwSignal<SlotMap<DefaultKey, TabKind>>,

    active_tab: RwSignal<Option<DefaultKey>>,

    config: Config,
}

fn app_view() -> impl IntoView {

    v_stack((
        //
        // Toolbar
        //
        h_stack((
            button(||"Add home").on_click_stop(move |_event|{

                println!("Add home pressed");

                let app_state: Arc<ApplicationState> = use_context().unwrap();

                app_state.tabs.update(|tabs|{
                    tabs.insert(
                        TabKind::Home(HomeTab {})
                    );
                });

            }),
            button(||"New").on_click_stop(move |_event|{

                println!("New pressed");
            }),
            button(||"Open"),
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
                let app_state: Option<Arc<ApplicationState>> = use_context();
                if let Some(tab_key) = active_tab {
                    println!("displaying tab. tab_id: {:?}", &tab_key);

                    let tabs = app_state.unwrap().tabs.get();
                    let tab = tabs.get(tab_key).unwrap();

                    match tab {
                        TabKind::Home(_) => {
                            v_stack((
                                label(|| "Home Tab Content"),
                                dyn_view(move ||format!("tab_id: {:?}", &tab_key))
                            )).into_any()
                        }
                        TabKind::Document(_) => {
                            label(|| "Document Tab Content").into_any()
                        }
                    }

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
        documents: Default::default(),
        tabs: create_rw_signal(Default::default()),
        active_tab: create_rw_signal(None),
        config,
    };

    if app_state.config.show_home_on_startup {
        app_state.tabs.update(|tabs|{
            let key = tabs.insert(
                TabKind::Home(HomeTab {})
            );

            app_state.active_tab.set(Some(key));
        })
    }

    let app_state_arc = Arc::new(app_state);

    provide_context(app_state_arc.clone());

    floem::launch(app_view);

    let content: String = serde_json::to_string(&app_state_arc.config).unwrap();

    let mut file = File::create(PathBuf::from("config.json")).unwrap();
    file.write(content.as_bytes()).unwrap();
}
