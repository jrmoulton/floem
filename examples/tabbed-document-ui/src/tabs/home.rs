use floem::views::{label, v_stack};
use floem::View;
use slotmap::DefaultKey;

#[derive(Clone)]
pub struct HomeTab {}

pub struct HomeContainer {}

impl HomeContainer {
    pub fn build_view(tab_key: DefaultKey) -> impl View {
        v_stack((
            "Home Tab Content",
            label(move || format!("tab_id: {:?}", &tab_key)),
        ))
    }
}
