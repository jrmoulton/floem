use floem::peniko::Color;
use floem::style::TextOverflow;
use floem::views::{empty, label, v_stack, Decorators, TupleStackExt};
use floem::View;
use std::path::PathBuf;

pub struct ImageDocument {
    path: PathBuf,
    // TODO content: ImageContent(...)
    coordinate: Option<(usize, usize)>,
}

impl ImageDocument {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            coordinate: None,
        }
    }

    pub fn build_view(&self) -> impl View {
        // FIXME doesn't make any difference, path appears truncated
        // same as in text file
        let path = self
            .path
            .to_str()
            .unwrap()
            .to_owned()
            .style(|s| s.text_overflow(TextOverflow::Wrap));

        let coordinates = {
            // FIXME this needs to be reactive
            let coordinate_label = format!("{:?}", self.coordinate);
            label(move || coordinate_label.clone())
        };

        let info_panel = v_stack((
            ("path", path).h_stack(),
            ("coordinate", coordinates).h_stack(),
        ))
        .style(|s| s.height_full().width_pct(20.0));

        let content = empty().style(|s| {
            s.height_full()
                // FIXME if this is 80% or 'full' it still doesn't take up the remaining space.
                .width_full()
                .background(Color::DARK_GRAY)
        });

        (info_panel, content)
            .h_stack()
            .style(|s| s.width_full().height_full())
    }
}
