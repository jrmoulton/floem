use floem::{
    peniko::{Brush, Color, Gradient},
    reactive::create_signal,
    views::{checkbox, labeled_checkbox, CheckboxClass, Decorators, SvgColor},
    IntoView,
};

use crate::form::{form, form_item};

pub fn checkbox_view() -> impl IntoView {
    let width = 160.0;
    let (is_checked, set_is_checked) = create_signal(true);
    form({
        (
            form_item("Checkbox:".to_string(), width, move || {
                checkbox(move || is_checked.get())
                    .on_update(move |checked| {
                        set_is_checked.set(checked);
                    })
                    .style(|s| {
                        s.margin(5.0).class(CheckboxClass, |s| {
                            s
                                // .background(
                                //     Gradient::new_linear((0., 0.), (20., 0.))
                                //         .with_stops([(0., Color::LIGHT_BLUE), (1., Color::RED)]),
                                // )
                                .set(
                                    SvgColor,
                                    Brush::Gradient(
                                        Gradient::new_linear((0., 0.), (20., 0.)).with_stops([
                                            (0., Color::LIGHT_BLUE),
                                            (1., Color::RED),
                                        ]),
                                    ),
                                )
                        })
                    })
            }),
            form_item("Disabled Checkbox:".to_string(), width, move || {
                checkbox(move || is_checked.get())
                    .style(|s| s.margin(5.0))
                    .disabled(|| true)
            }),
            form_item("Labelled Checkbox:".to_string(), width, move || {
                labeled_checkbox(move || is_checked.get(), || "Check me!")
                    .style(|s| s.class(CheckboxClass, |s| s.color(Color::GREEN)))
                    .on_update(move |checked| {
                        set_is_checked.set(checked);
                    })
            }),
            form_item(
                "Disabled Labelled Checkbox:".to_string(),
                width,
                move || {
                    labeled_checkbox(move || is_checked.get(), || "Check me!").disabled(|| true)
                },
            ),
        )
    })
}
