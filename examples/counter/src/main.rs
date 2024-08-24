use floem::{
    keyboard::{Key, Modifiers, NamedKey},
    peniko::Color,
    reactive::{RwSignal, SignalUpdate},
    unit::UnitExt,
    views::{button, dyn_view, stack, Decorators, LabelClass, LabelCustomStyle},
    IntoView, View,
};

fn app_view() -> impl IntoView {
    let mut counter = RwSignal::new(0);
    let view = (
        dyn_view(move || format!("Value: {}", counter)),
        counter.style(|s| s.padding(10.0)),
        stack((
            button("Increment")
                .style(|s| {
                    s.border_radius(10.0)
                        .padding(10.0)
                        .background(Color::WHITE)
                        .box_shadow_blur(5.0)
                        .focus_visible(|s| s.outline(2.).outline_color(Color::BLUE))
                        .hover(|s| s.background(Color::LIGHT_GREEN))
                        .active(|s| s.color(Color::WHITE).background(Color::DARK_GREEN))
                })
                .action(move || {
                    counter += 1;
                })
                .keyboard_navigatable(),
            button("Decrement")
                .action(move || {
                    counter -= 1;
                })
                .style(|s| {
                    s.box_shadow_blur(5.0)
                        .background(Color::WHITE)
                        .border_radius(10.0)
                        .padding(10.0)
                        .margin_left(10.0)
                        .focus_visible(|s| s.outline(2.).outline_color(Color::BLUE))
                        .hover(|s| s.background(Color::rgb8(244, 67, 54)))
                        .active(|s| s.color(Color::WHITE).background(Color::RED))
                })
                .keyboard_navigatable(),
            button("Reset to 0")
                .action(move || {
                    println!("Reset counter pressed"); // will not fire if button is disabled
                    counter.set(0);
                })
                .disabled(move || counter == 0)
                .style(|s| {
                    s.box_shadow_blur(5.0)
                        .border_radius(10.0)
                        .padding(10.0)
                        .margin_left(10.0)
                        .background(Color::LIGHT_BLUE)
                        .focus_visible(|s| s.outline(2.).outline_color(Color::BLUE))
                        .disabled(|s| s.background(Color::LIGHT_GRAY))
                        .hover(|s| s.background(Color::LIGHT_YELLOW))
                        .active(|s| s.color(Color::WHITE).background(Color::YELLOW_GREEN))
                })
                .keyboard_navigatable(),
        ))
        .style(|s| {
            s.class(LabelClass, |s| {
                s.apply(LabelCustomStyle::new().selectable(false).style())
            })
        }),
    )
        .style(|s| {
            s.size(100.pct(), 100.pct())
                .flex_col()
                .items_center()
                .justify_center()
        });

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| {
        id.inspect()
    })
}

fn main() {
    floem::launch(app_view);
}
