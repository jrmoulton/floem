use floem::{
    peniko::{Brush, Color},
    reactive::{RwSignal, SignalGet, SignalUpdate},
    style::{Background, PaddingBottom, PaddingLeft, PaddingRight, PaddingTop, Style, Transition},
    taffy::AlignItems,
    text::Weight,
    unit::DurationUnitExt,
    views::*,
    AnyView, IntoView,
};

use crate::box_shadow;

const FONT_SIZE: f32 = 12.;
const BACKGROUND: Color = Color::rgb8(235, 235, 240);
const SLIDER: Color = Color::rgb8(210, 209, 216);
const ICON: Color = Color::rgb8(120, 120, 127); // medium gray - icons and accent bar and image
const MUSIC_ICON: Color = Color::rgb8(11, 11, 21);
const TEXT_COLOR: Color = Color::rgb8(48, 48, 54);

mod svg;

#[derive(Debug, Clone)]
struct SongInfo {
    title: String,
    artist: String,
}
impl Default for SongInfo {
    fn default() -> Self {
        Self {
            title: "Cool Song Title".to_string(),
            artist: "Artist Name".to_string(),
        }
    }
}
impl IntoView for SongInfo {
    type V = Stack;

    fn into_view(self) -> Self::V {
        let title = self.title.style(|s| s.font_weight(Weight::MEDIUM));

        let artist = self
            .artist
            .style(|s| s.font_size(FONT_SIZE * 0.8).color(Color::GRAY));

        let art_cover = empty().style(|s| s.size(50, 50).border_radius(8).background(ICON));

        let song_artist = (title, artist).v_stack().style(|s| s.gap(5.));

        (art_cover, song_artist)
            .h_stack()
            .style(|s| s.gap(10).items_center())
    }
}

#[derive(Debug, Clone, Copy)]
enum PlayPause {
    Play,
    Pause,
}
impl PlayPause {
    fn toggle(&mut self) {
        *self = match self {
            PlayPause::Play => PlayPause::Pause,
            PlayPause::Pause => PlayPause::Play,
        };
    }
}
impl IntoView for PlayPause {
    type V = AnyView;

    fn into_view(self) -> Self::V {
        match self {
            PlayPause::Play => svg(svg::PLAY).into_any(),
            PlayPause::Pause => svg(svg::PAUSE).into_any(),
        }
        .animation(|a| a.scale_effect().run_on_remove(false))
    }
}

pub fn music_player() -> impl IntoView {
    let song_info = RwSignal::new(SongInfo::default());
    let play_pause_state = RwSignal::new(PlayPause::Play);

    let now_playing = (svg(svg::MUSIC), "Now Playing").h_stack().style(|s| {
        s.gap(5)
            .items_center()
            .font_weight(Weight::MEDIUM)
            .color(MUSIC_ICON)
    });

    let play_pause_button = dyn_container(move || play_pause_state.get(), move |state| state)
        .class(ButtonClass)
        .container()
        .on_click_stop(move |_| play_pause_state.update(|s| s.toggle()));

    let back_button = svg(svg::BACKWARD).container().class(ButtonClass);
    let forward_button = svg(svg::BACKWARD).container().class(ButtonClass);

    let media_buttons = (back_button, play_pause_button, forward_button)
        .h_stack()
        .style(|s| {
            s.align_self(Some(AlignItems::Center))
                .items_center()
                .gap(20)
                .class(SvgClass, |s| s.color(MUSIC_ICON))
        });

    let song_info_view = dyn_container(move || song_info.get(), |info| info);

    let progress_slider = slider::slider(move || 40.)
        .style(|s| s.width_full())
        .slider_style(|s| {
            s.bar_height(3)
                .accent_bar_height(3.)
                .bar_color(SLIDER)
                .accent_bar_color(ICON)
                .handle_color(Brush::Solid(Color::TRANSPARENT))
                .handle_radius(0)
        });

    let card = (now_playing, song_info_view, progress_slider, media_buttons)
        .v_stack()
        .style(|s| {
            s.background(BACKGROUND)
                .size_full()
                .border_radius(8)
                .padding(15)
                .gap(10)
                .width(300)
                .apply(box_shadow())
        });

    let svg_style = |s: Style| {
        s.size(20, 20)
            .inset_top(0.7)
            .inset_right(0.7)
            .items_center()
            .justify_center()
            .flex_shrink(2.)
            .transition_color(Transition::spring(25.millis()))
    };

    let button_style = |s: Style| {
        s.border(0)
            .padding(5)
            .items_center()
            .justify_center()
            .background(Color::TRANSPARENT)
            .transition(PaddingBottom, Transition::spring(25.millis()))
            .transition(PaddingTop, Transition::spring(25.millis()))
            .transition(PaddingLeft, Transition::spring(25.millis()))
            .transition(PaddingRight, Transition::spring(25.millis()))
            .hover(|s| s.background(SLIDER))
            .active(|s| {
                s.set_style_value(Background, floem::style::StyleValue::Unset)
                    .padding(10)
                    .class(SvgClass, |s| s.color(ICON))
            })
    };

    let card_style = move |s: Style| {
        s.size(300, 175)
            .items_center()
            .justify_center()
            .font_size(FONT_SIZE)
            .color(TEXT_COLOR)
            .class(SvgClass, svg_style)
            .class(ButtonClass, button_style)
    };

    card.container().style(card_style)
}
