use floem::{
    peniko::Color,
    reactive::create_rw_signal,
    unit::UnitExt,
    view::View,
    views::{container, label, stack, text, Decorators},
};

fn app_view() -> impl View {
    let file_name = create_rw_signal(String::new());

    let filed_hovered = create_rw_signal(false);

    let drop_zone = container(
        container(stack((text("Drop Here!"), label(move || file_name.get()))))
            .style(move |s| {
                s.size(150, 150)
                    .background(Color::LIGHT_GRAY)
                    .items_center()
                    .justify_center()
                    .apply_if(filed_hovered.get(), |s| s.background(Color::LIGHT_GREEN))
            })
            .on_event_stop(floem::event::EventListener::HoveredFile, move |e| {
                if let floem::event::Event::HoveredFile(path) = e {
                    dbg!("This");
                    file_name.set(
                        path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    )
                }
            }),
    )
    .style(|s| s.absolute().size_full().items_center().justify_center());

    stack((text("File Drop"), drop_zone)).style(|s| s.size_full())
}

fn main() {
    floem::launch(app_view);
}
