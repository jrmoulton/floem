use floem::{reactive::RwSignal, views::*, IntoView};

fn app_view() -> impl IntoView {
    let mut counter = RwSignal::new(0);

    v_stack((
        label(move || format!("Value: {counter}")),
        h_stack((
            button("Increment").action(move || counter += 1),
            button("Decrement").action(move || counter -= 1),
        )),
    ))
    .style(|s| s.items_center().justify_center())
}

fn main() {
    floem::launch(app_view);
}
