pub mod buttons;
pub mod checkbox;
pub mod clipboard;
pub mod context_menu;
pub mod dropdown;
pub mod form;
pub mod images;
pub mod inputs;
pub mod labels;
pub mod lists;
pub mod radio_buttons;
pub mod rich_text;
pub mod slider;

use floem::{
    event::{Event, EventListener, EventPropagation},
    keyboard::{Key, NamedKey},
    kurbo::Point,
    peniko::{Color, Gradient},
    reactive::{create_signal, RwSignal},
    style::{Background, CursorStyle, Transition},
    unit::UnitExt,
    views::{
        button, h_stack, label, scroll, stack, tab, v_stack, virtual_stack, Decorators,
        VirtualDirection, VirtualItemSize,
    },
    IntoView, View,
};

fn app_view() -> impl IntoView {
    let grad_start = RwSignal::new(0.);
    let grad_end = RwSignal::new(1.);
    let tabs: im::Vector<&str> = vec![
        "Label",
        "Button",
        "Checkbox",
        "Radio",
        "Input",
        "List",
        "Menu",
        "RichText",
        "Image",
        "Clipboard",
        "Slider",
        "Dropdown",
    ]
    .into_iter()
    .collect();
    let (tabs, _set_tabs) = create_signal(tabs);

    let (active_tab, set_active_tab) = create_signal(0);

    let list = scroll({
        virtual_stack(
            VirtualDirection::Vertical,
            VirtualItemSize::Fixed(Box::new(|| 36.0)),
            move || tabs.get(),
            move |item| *item,
            move |item| {
                let index = tabs
                    .get_untracked()
                    .iter()
                    .position(|it| *it == item)
                    .unwrap();
                stack((label(move || item).style(|s| s.font_size(18.0)),))
                    .on_click_stop(move |_| {
                        set_active_tab.update(|v: &mut usize| {
                            *v = tabs
                                .get_untracked()
                                .iter()
                                .position(|it| *it == item)
                                .unwrap();
                        });
                    })
                    .on_event(EventListener::KeyDown, move |e| {
                        if let Event::KeyDown(key_event) = e {
                            let active = active_tab.get();
                            if key_event.modifiers.is_empty() {
                                match key_event.key.logical_key {
                                    Key::Named(NamedKey::ArrowUp) => {
                                        if active > 0 {
                                            set_active_tab.update(|v| *v -= 1)
                                        }
                                        EventPropagation::Stop
                                    }
                                    Key::Named(NamedKey::ArrowDown) => {
                                        if active < tabs.get().len() - 1 {
                                            set_active_tab.update(|v| *v += 1)
                                        }
                                        EventPropagation::Stop
                                    }
                                    _ => EventPropagation::Continue,
                                }
                            } else {
                                EventPropagation::Continue
                            }
                        } else {
                            EventPropagation::Continue
                        }
                    })
                    .keyboard_navigatable()
                    .draggable()
                    .style(move |s| {
                        let start_point: Point = (0., 0.).into();
                        let end_point: Point = (140., 0.).into();
                        let grad_start = move || grad_start.get() / 100.;
                        let grad_end = move || grad_end.get() / 100.;
                        s.flex_row()
                            .padding(5.0)
                            .width(100.pct())
                            .height(36.0)
                            .transition(Background, Transition::linear(0.5))
                            .items_center()
                            .border_bottom(1.0)
                            .border_color(Color::LIGHT_GRAY)
                            .background(Gradient::new_linear(start_point, end_point).with_stops([
                                (0., Color::BLUE),
                                (grad_start(), Color::BLUE),
                                (grad_end(), Color::RED),
                            ]))
                            .apply_if(index == active_tab.get(), |s| {
                                s.background(
                                    Gradient::new_linear(start_point, end_point).with_stops([
                                        (0., Color::YELLOW),
                                        (grad_start(), Color::YELLOW),
                                        (grad_end(), Color::GREEN),
                                    ]),
                                )
                            })
                            .hover(|s| {
                                s.background(
                                    Gradient::new_linear(start_point, end_point).with_stops([
                                        (0., Color::RED),
                                        (grad_start(), Color::RED),
                                        (grad_end(), Color::BLUE),
                                    ]),
                                )
                                .apply_if(index == active_tab.get(), |s| s.background(Color::BLUE))
                                .cursor(CursorStyle::Pointer)
                            })
                            .focus_visible(|s| s.border(2.).border_color(Color::BLUE))
                    })
            },
        )
        .style(|s| s.flex_col().width(140.0))
    })
    .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.border(1.).border_color(Color::GRAY));

    let id = list.id();
    let inspector = button(|| "Open Inspector")
        .on_click_stop(move |_| {
            id.inspect();
        })
        .style(|s| s);

    let left = v_stack((list, inspector)).style(|s| s.height_full().column_gap(5.0));

    let tab = tab(
        move || active_tab.get(),
        move || tabs.get(),
        |it| *it,
        move |it| match it {
            "Label" => labels::label_view().debug_name("Label View").into_any(),
            "Button" => buttons::button_view().debug_name("Button View").into_any(),
            "Checkbox" => checkbox::checkbox_view()
                .debug_name("Checkbox View")
                .into_any(),
            "Radio" => radio_buttons::radio_buttons_view()
                .debug_name("Radio Buttons View")
                .into_any(),
            "Input" => inputs::text_input_view()
                .debug_name("Text Input View")
                .into_any(),
            "List" => lists::virt_list_view()
                .debug_name("Virtual List View")
                .into_any(),
            "Menu" => context_menu::menu_view().debug_name("Menu View").into_any(),
            "RichText" => rich_text::rich_text_view()
                .debug_name("Rich Text View")
                .into_any(),
            "Image" => images::img_view().debug_name("Image View").into_any(),
            "Clipboard" => clipboard::clipboard_view()
                .debug_name("Clipboard View")
                .into_any(),
            "Slider" => slider::slider_view(grad_start, grad_end)
                .debug_name("Slider View")
                .into_any(),
            "Dropdown" => dropdown::dropdown_view()
                .debug_name("Dropdown View")
                .into_any(),
            _ => label(|| "Not implemented".to_owned()).into_any(),
        },
    )
    .style(|s| s.flex_col().items_start());

    let tab = scroll(tab).scroll_style(|s| s.shrink_to_fit());

    let view = h_stack((left, tab))
        .style(|s| s.padding(5.0).width_full().height_full().row_gap(5.0))
        .window_title(|| "Widget Gallery".to_owned());

    let id = view.id();
    view.on_event_stop(EventListener::KeyUp, move |e| {
        if let Event::KeyUp(e) = e {
            if e.key.logical_key == Key::Named(NamedKey::F11) {
                id.inspect();
            }
        }
    })
}

fn main() {
    floem::launch(app_view);
}
