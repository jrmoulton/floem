use floem::{
    reactive::RwSignal,
    views::{label, slider, stack, Decorators},
    IntoView,
};

use crate::form::{self, form_item};

pub fn slider_view(grad_start: RwSignal<f32>, grad_end: RwSignal<f32>) -> impl IntoView {
    form::form({
        (
            form_item("Gradient Start:".to_string(), 120.0, move || {
                stack((
                    slider::slider(move || grad_start.get())
                        .style(|s| s.width(200))
                        .on_change_pct(move |val| grad_start.set(val)),
                    label(move || format!("{:.1}%", grad_start.get())),
                ))
                .style(|s| s.gap(10))
            }),
            form_item("Gradient End:".to_string(), 120.0, move || {
                stack((
                    slider::slider(move || grad_end.get())
                        .style(|s| s.width(200))
                        .on_change_pct(move |val| grad_end.set(val)),
                    label(move || format!("{:.1}%", grad_end.get())),
                ))
                .style(|s| s.gap(10))
            }),
        )
    })
}
