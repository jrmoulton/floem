use strum::IntoEnumIterator;

use floem::{
    peniko::Color,
    reactive::create_rw_signal,
    unit::UnitExt,
    view::View,
    views::{container, label, stack, svg, Decorators},
    widgets::dropdown::dropdown,
};

use crate::form::{self, form_item};

#[derive(strum::EnumIter, Debug, PartialEq, Clone, Copy)]
enum Values {
    One,
    Two,
    Three,
    Four,
    Five,
}
impl std::fmt::Display for Values {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

const CHEVRON_DOWN: &str = r#"<?xml version="1.0" encoding="iso-8859-1"?>
<!-- Uploaded to: SVG Repo, www.svgrepo.com, Generator: SVG Repo Mixer Tools -->
<svg height="800px" width="800px" version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
	 viewBox="0 0 185.344 185.344" xml:space="preserve">
<g>
	<g>
		<path style="fill:#010002;" d="M92.672,144.373c-2.752,0-5.493-1.044-7.593-3.138L3.145,59.301c-4.194-4.199-4.194-10.992,0-15.18
			c4.194-4.199,10.987-4.199,15.18,0l74.347,74.341l74.347-74.341c4.194-4.199,10.987-4.199,15.18,0
			c4.194,4.194,4.194,10.981,0,15.18l-81.939,81.934C98.166,143.329,95.419,144.373,92.672,144.373z"/>
	</g>
</g>
</svg>"#;

pub fn dropdown_view() -> impl View {
    let driving_signal = create_rw_signal(Values::Three);

    form::form({
        (form_item("Dropdown".to_string(), 120.0, move || {
            dropdown(
                // main view
                |item| {
                    stack((
                        label(move || item),
                        container(
                            svg(|| String::from(CHEVRON_DOWN))
                                .style(|s| s.size(12, 12).color(Color::BLACK)),
                        )
                        .style(|s| {
                            s.items_center()
                                .padding(3.)
                                .border_radius(7.pct())
                                .hover(move |s| s.background(Color::LIGHT_GRAY))
                        }),
                    ))
                },
                // iterator to build list in dropdown
                Values::iter().map(move |item| {
                    label(move || item)
                        .style(|s| s.size_full())
                        .on_click_stop(move |_| {
                            driving_signal.set(item);
                            println!("Selected {item:?}!")
                        })
                }),
                move || driving_signal.get(),
            )
        }),)
    })
}
