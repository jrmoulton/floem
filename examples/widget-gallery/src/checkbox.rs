use floem::{
    reactive::{RwSignal, SignalGet},
    views::{Checkbox, Decorators},
    IntoView,
};

use crate::form::{form, form_item};

pub fn checkbox_view() -> impl IntoView {
    let width = 160.0;
    let is_checked = RwSignal::new(true);
    form({
        (
            form_item("Checkbox:".to_string(), width, move || {
                Checkbox::new_read_write(is_checked).style(|s| s.margin(5.0))
            }),
            form_item("Disabled Checkbox:".to_string(), width, move || {
                Checkbox::new(move || is_checked.get())
                    .style(|s| s.margin(5.0))
                    .disabled(|| true)
            }),
            form_item("Labelled Checkbox:".to_string(), width, move || {
                Checkbox::new_labeled_read_write(is_checked, || "Check me!")
            }),
            form_item(
                "Disabled Labelled Checkbox:".to_string(),
                width,
                move || {
                    Checkbox::new_labeled(move || is_checked.get(), || "Check me!")
                        .disabled(|| true)
                },
            ),
        )
    })
}
