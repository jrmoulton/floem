use floem::peniko::Color;
use floem::reactive::SignalGet;
use floem::style::TextOverflow;
use floem::views::{h_stack, label, text_editor, v_stack, Decorators, TupleStackExt};
use floem::View;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct TextDocument {
    path: PathBuf,
    content: String,
}

impl TextDocument {
    pub fn new(path: PathBuf) -> Self {
        let content = fs::read_to_string(&path).unwrap();

        Self { path, content }
    }

    pub fn build_view(self) -> impl View {
        let editor = text_editor(self.content);
        let cursor = editor.editor().cursor;
        let info_panel = v_stack((
            h_stack((
                "path",
                // FIXME doesn't make any difference, path appears truncated
                // This isn't wrapping because text doens't have breaks. We need to add a wrap type to floem style. The text layout already supports this so this should be trivial to add
                self.path
                    .to_str()
                    .unwrap()
                    .style(|s| s.text_overflow(TextOverflow::Wrap)),
            )),
            label(move || {
                let mut format = "Selection:".to_string();
                let cursor = cursor.get();
                if let Some(selection) = cursor.get_selection() {
                    if selection.0 != selection.1 {
                        format = format!("{format} (selected {}, {})", selection.0, selection.1);
                    }
                }
                let selection_count = cursor.get_selection_count();
                if selection_count > 1 {
                    format = format!("{format} {selection_count} selections");
                }
                format
            }),
        ))
        .style(|s| s.height_full().width_pct(20.0));

        let content = editor
            .style(|s| s.size_full().background(Color::DARK_GRAY))
            .editor_style(|es| {
                es.gutter_accent_color(Color::WHITE_SMOKE.with_alpha_factor(0.75))
                    .gutter_dim_color(Color::BLACK.with_alpha_factor(0.9))
            });

        (info_panel, content)
            .h_stack()
            .style(|s| s.width_full().height_full())
    }
}
