use floem::{
    peniko::{Brush, Color},
    reactive::{RwSignal, SignalGet, SignalUpdate},
    style::{ScaleX, ScaleY, Style, Transition},
    text::Weight,
    unit::{DurationUnitExt, UnitExt},
    views::{
        container, dyn_container, empty, h_stack, slider, svg, v_stack, ButtonClass, Decorators,
        Stack, SvgClass,
    },
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

    let button_style = |s: Style| {
        s.border(0)
            .padding(5)
            .items_center()
            .justify_center()
            .background(Color::TRANSPARENT)
            .hover(|s| s.background(SLIDER))
            .active(|s| {
                s.class(SvgClass, |s| {
                    s.color(ICON).scale_x(50.pct()).scale_y(50.pct())
                })
            })
    };

    container(card).style(move |s| {
        s.size(300, 175)
            .items_center()
            .justify_center()
            .font_size(FONT_SIZE)
            .color(TEXT_COLOR)
            .class(SvgClass, |s| {
                s.size(20, 20)
                    .items_center()
                    .justify_center()
                    .scale(100.pct())
                    .transition(ScaleX, Transition::spring(50.millis()))
                    .transition(ScaleY, Transition::spring(50.millis()))
                    .transition_color(Transition::linear(50.millis()))
            })
            .class(ButtonClass, button_style)
    })
}
