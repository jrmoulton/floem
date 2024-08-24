//! # Floem
//! Floem is cross-platform GUI library for Rust. It aims to be extremely performant while providing world-class developer ergonomics.
//!
//! ## Counter Example
//! ```rust
//! use floem::{reactive::*, views::*};
//!
//! let mut counter = RwSignal::new(0);
//!
//! v_stack((
//!     label(move || format!("Value: {counter}")),
//!     h_stack((
//!         button("Increment").action(move || counter += 1),
//!         button("Decrement").action(move || counter -= 1),
//!     )),
//! ));
//! ```
//!
//! ## Views
//! Floem models the UI using a tree of [Views](view::View) that is constructed once.
//! Views can be composed together to create complex UIs that are capable of reacting to state changes and events.
//!
//! You can read more about the built-in views and how to compose your UI in the [views module](crate::views).
//!
//! ## State management
//! Floem uses reactivity built on signals and effects for its state management. This design
//! pattern has been popularized by SolidJS in the JavaScript ecosystem and this implementation has been directly
//! inspired Leptos in the Rust ecosystem. Floem uses its own reactive system with an API that
//! is similar to the one in the leptos_reactive crate. To learn more about signals and
//! effects, you may want to explore the Leptos [documentation](https://docs.rs/leptos_reactive/latest/leptos_reactive/index.html)
//! and the [leptos book](https://leptos-rs.github.io/leptos/).
//!
//! #### State
//!
//! You can create a reactive signal anywhere in the program using [`RwSignal::new()`](floem_reactive::RwSignal::new) and [`RwSignal::new_split()`](floem_reactive::RwSignal::new_split) or use a [different signal type](floem_reactive).
//!
//! When you use a signal in a reactive context and call [`get`](floem_reactive::SignalGet::get) or [`with`](floem_reactive::SignalWith::with), (which are also called when you use an operator such as `==`)
//! the runtime will automatically subscribe the correct side effects
//! to changes in that signal, creating reactivity. To the programmer this is transparent. The reactivity
//! "just works" when you access the value where you want to use it.
//!
//! Example:
//! ```
//! # use floem::reactive::*;
//! # use floem::IntoView;
//! # use floem::views::{label, v_stack, text_input, Decorators};
//! #
//! fn app_view() -> impl IntoView {
//!     let text = RwSignal::new("Hello world".to_string());
//!
//!     v_stack((
//!         text_input(text),
//!         label(move || text.get()),
//!     ))
//! }
//! ```
//!
//! In this example, `text` is a signal containing a `String` that can be both read from and written to.
//! The [text input](crate::views::text_input) has direct access to the signal and will mutate the underlying `String` when the user types in the input box.
//! The reactivity will then automatically trigger a re-render of the [label](crate::views::label) with the updated text value.
//!
//! All signal types implement `Copy`, so they can be easily used where needed without needing to manually clone them.
//!
//! ## Customizing appearance
//!
//! Floem has a powerful built-in style system that allows you to customize the appearance of your UI.
//!
//! Example:
//!
//! ```
//! #  use floem::peniko::Color;
//! #  use floem::reactive::*;
//! #  use floem::style::Style;
//! #  use floem::unit::UnitExt;
//! #  use floem::View;
//! #  use floem::views::{text, Decorators};
//! #
//! // Styles can be updated reactively using any signal.
//! // This will apply a gray background color while `active_tab` equals 0.
//!
//! let active_tab = RwSignal::new(0);
//!
//! // The following closure will be automatically re-run any time `active_tab` is set.
//! text("Some text").style(move |s| {
//!     s.width(75)
//!         .font_size(21.)
//!         .border_bottom(1.)
//!         .border_color(Color::LIGHT_GRAY)
//!         .apply_if(active_tab == 0, |s| s.background(Color::GRAY))
//! });
//! ```
//!
//! The View instance is styled by calling the [`style`](crate::views::Decorators::style) method (you'll need to import the
//! [`Decorators`](crate::views::Decorators) trait to use the it). The `style` method takes a closure that takes and returns a
//! [`Style`](crate::style::Style) value using the builder pattern. Through this value, you can access methods that modify a variety
//! of familiar properties such as width, padding, and background. Some `Style` properties
//! such as font size are `inherited` and will apply to all of a view's children until overriden.
//!
// TODO: Add links on these
//! In this same style value, floem supports:
//!     adding custom properties, applying styles conditionally, property transitions, defining styles on different interaction states, themeing with classes, and more.
//!
//! For additional information about styling, [see here](crate::style).
//!
//! ## Animation
//!
//! Floem has a full keyframe animation system that allows you to animate any property that can be interpolated and builds on the capabilities and ergonomics of the style system.
//!
//! Example:
//!
//! ```
//! #  use floem::peniko::Color;
//! #  use floem::reactive::*;
//! #  use floem::style::Style;
//! #  use floem::unit::{UnitExt, DurationUnitExt};
//! #  use floem::View;
//! #  use floem::views::*;
//! #
//! empty()
//!     .style(|s| s.background(Color::RED).size(500, 100))
//!     .animation(move |a| {
//!         a.duration(5.seconds())
//!             .keyframe(50, |kf| {
//!                 kf.style(|s| s.background(Color::BLACK).size(30, 30))
//!                     .ease_in()
//!             })
//!             .keyframe(100, |kf| {
//!                 kf.style(|s| s.background(Color::AQUAMARINE).size(10, 300))
//!                     .ease_out()
//!             })
//!             .repeat(true)
//!             .auto_reverse(true)
//!     });
//!
//! ```
//!
//! You can add aninimations to a View instance by calling the [`animation`](crate::views::Decorators::animation) method.
//! The `animation` method takes a closure that takes and returns an [`Animation`](crate::animate::Animation) value using the builder pattern.
//!
//! In this same style value floem supports:
//!     adding custom properties, applying styles conditionally, property transitions, defining styles on different interaction states, themeing with classes, and more.
//!
//! For additional information about styling, [see here](crate::style).
//!
//!
//! ## Additional reading
//!
//!
pub mod action;
pub mod animate;
mod app;
mod app_handle;
pub(crate) mod app_state;
mod clipboard;
pub mod context;
pub mod dropped_file;
pub mod event;
pub mod ext_event;
pub mod file;
#[cfg(any(feature = "rfd-async-std", feature = "rfd-tokio"))]
pub mod file_action;
pub(crate) mod id;
mod inspector;
pub mod keyboard;
pub mod menu;
mod nav;
pub mod pointer;
mod profiler;
pub mod renderer;
pub mod responsive;
mod screen_layout;
pub mod style;
pub(crate) mod theme;
pub mod unit;
mod update;
pub(crate) mod view;
pub(crate) mod view_state;
pub(crate) mod view_storage;
pub mod view_tuple;
pub mod views;
pub mod window;
mod window_handle;
mod window_id;
mod window_tracking;

pub use app::{launch, quit_app, AppEvent, Application};
pub use app_state::AppState;
pub use clipboard::{Clipboard, ClipboardError};
pub use floem_reactive as reactive;
pub use floem_renderer::text;
pub use floem_renderer::Renderer;
pub use id::ViewId;
pub use peniko;
pub use peniko::kurbo;
pub use screen_layout::ScreenLayout;
pub use taffy;
pub use view::{recursively_layout_view, AnyView, IntoView, View};
pub use window::{close_window, new_window};
pub use window_id::{Urgency, WindowIdExt};
