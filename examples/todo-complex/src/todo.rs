use std::{hash::Hasher, sync::atomic::AtomicU64, time::Instant};

use floem::{
    action::{debounce_action, exec_after},
    easing::Spring,
    event::{Event, EventListener},
    keyboard::Modifiers,
    kurbo::Stroke,
    menu::{Menu, MenuEntry, MenuItem},
    prelude::*,
    reactive::{create_effect, create_memo, create_updater, Trigger},
    style::{BoxShadowProp, CursorStyle, MinHeight, Transition},
    taffy::AlignItems,
    views::Checkbox,
    AnyView,
};

use crate::{todo_state::TODOS_STATE, AppCommand, OS_MOD};

/// this state macro is unnecessary but convenient. It just produces the original struct and a new struct ({StructName}State) with all of the same fields but wrapped in Signal types.
#[derive(Clone, floem::State)]
#[state_derives(Clone, Copy, Eq, Debug)]
pub struct Todo {
    pub db_id: Option<i64>,
    #[state_skip]
    pub unique_id: u64,
    pub done: bool,
    pub description: String,
}
static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);
impl Todo {
    pub fn new_from_db(db_id: i64, done: bool, description: impl Into<String>) -> Self {
        Self {
            db_id: Some(db_id),
            unique_id: UNIQUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            done,
            description: description.into(),
        }
    }
    pub fn new(done: bool, description: impl Into<String>) -> Self {
        Self {
            db_id: None,
            unique_id: UNIQUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            done,
            description: description.into(),
        }
    }
}
impl IntoView for TodoState {
    type V = AnyView;

    fn into_view(self) -> Self::V {
        // when the done status changes, commit the change to the db
        create_updater(
            move || self.done.track(),
            move |_| AppCommand::UpdateDone(self).execute(),
        );

        // when the description changes, debounce a commit to the db
        debounce_action(self.description, 300.millis(), move || {
            AppCommand::UpdateDescription(self).execute()
        });

        let is_selected =
            create_memo(move |_| TODOS_STATE.with(|s| s.selected.with(|s| s.contains(&self))));
        let is_active = create_memo(move |_| {
            TODOS_STATE.with(|s| s.active.with(|s| s.active.map_or(false, |v| v == self)))
        });

        let todo_action_menu = move || {
            // would be better to have actions that can operate on multiple selections.
            AppCommand::Escape.execute();
            AppCommand::InsertSelected(self).execute();
            let delete = MenuItem::new("Delete").action(move || {
                AppCommand::Delete(&[self]).execute();
            });
            let done = self.done.get();
            let action_name = if done {
                "Mark as Incomplete"
            } else {
                "Mark as Complete"
            };
            let toggle_done = MenuItem::new(action_name).action(move || {
                if done {
                    self.done.set(false);
                } else {
                    self.done.set(true);
                }
            });
            Menu::new("todo action")
                .entry(MenuEntry::Item(toggle_done))
                .entry(MenuEntry::Item(delete))
        };

        let input_focused = Trigger::new();
        let done_check = Checkbox::new_rw(self.done)
            .style(|s| {
                s.flex_shrink(0.)
                    .max_height_pct(70.)
                    .aspect_ratio(1.)
                    .border(
                        Stroke::new(1.)
                            .with_dashes(0.2, [1., 2.])
                            .with_caps(floem::kurbo::Cap::Round),
                    )
                    .class(SvgClass, |s| s.size_pct(50., 50.))
            })
            .on_event_stop(EventListener::PointerDown, move |_| {})
            .on_event_stop(EventListener::FocusGained, move |_| {
                dbg!("here");
                AppCommand::ChangeActive(self).execute();
            })
            .on_event_stop(EventListener::FocusLost, move |_| {
                AppCommand::FocusLost.execute();
            });

        let input = todo_input(self)
            .into_view()
            .on_event_stop(EventListener::FocusGained, move |_| {
                AppCommand::SetActive(self).execute();
                input_focused.notify();
            })
            .on_event_stop(EventListener::FocusLost, move |_| {
                AppCommand::FocusLost.execute();
            });

        // if this todo is being created after the app has already been initialized, focus the input
        let input_id = input.id();
        if TODOS_STATE.with(|s| Instant::now().duration_since(s.time_stated) > 50.millis()) {
            input_id.request_focus();
        }
        create_effect(move |_| {
            if is_active.get() {
                input_id.request_focus();
            }
        });

        let main_controls = (done_check, input)
            .h_stack()
            .debug_name("Todo Checkbox and text input (main controls)")
            .style(|s| s.gap(10).width_full().items_center())
            .container()
            .on_event_stop(EventListener::PointerDown, move |_| {
                AppCommand::ChangeActive(self).execute();
            })
            .on_click_stop(move |e| {
                let Event::PointerUp(e) = e else {
                    return;
                };
                if e.modifiers == OS_MOD {
                    AppCommand::ToggleSelected(self).execute();
                } else if e.modifiers.contains(Modifiers::SHIFT) {
                    AppCommand::SelectRange(self).execute();
                } else {
                    AppCommand::SetSelected(self).execute();
                }
            })
            .on_event_stop(EventListener::DoubleClick, move |_| {
                input_id.request_focus();
            })
            .style(|s| s.width_full().align_items(Some(AlignItems::FlexStart)));

        let container = main_controls.container();
        let final_view_id = container.id();
        create_effect(move |_| {
            input_focused.track();
            // this is a super ugly hack...
            // We should really figure out a way to make sure than an item that is focused
            // can be scrolled to and then kept in view if it has an animation/transition
            exec_after(25.millis(), move |_| final_view_id.scroll_to(None));
            exec_after(50.millis(), move |_| final_view_id.scroll_to(None));
            exec_after(100.millis(), move |_| final_view_id.scroll_to(None));
            exec_after(200.millis(), move |_| final_view_id.scroll_to(None));
            exec_after(300.millis(), move |_| final_view_id.scroll_to(None));
            exec_after(400.millis(), move |_| final_view_id.scroll_to(None));
        });

        container
            .style(move |s| {
                s.width_full()
                    .min_height(0.)
                    .padding(5)
                    .border_radius(5.)
                    .transition(MinHeight, Transition::new(600.millis(), Spring::snappy()))
                    .box_shadow_blur(0.)
                    .box_shadow_color(Color::BLACK.multiply_alpha(0.0))
                    .box_shadow_h_offset(0.)
                    .box_shadow_v_offset(0.)
                    .background(Color::TRANSPARENT)
                    .apply_if(is_selected.get(), |s| {
                        s.background(Color::LIGHT_BLUE.multiply_alpha(0.7))
                    })
                    .apply_if(is_active.get(), |s| {
                        s.min_height(100)
                            .background(Color::WHITE_SMOKE)
                            .box_shadow_blur(2.)
                            .box_shadow_color(Color::BLACK.multiply_alpha(0.7))
                            .box_shadow_h_offset(1.)
                            .box_shadow_v_offset(2.)
                            .transition(
                                BoxShadowProp,
                                Transition::new(600.millis(), Spring::snappy()),
                            )
                    })
            })
            .context_menu(todo_action_menu)
            .into_any()
    }
}
impl IntoView for Todo {
    type V = <TodoState as IntoView>::V;

    fn into_view(self) -> Self::V {
        self.to_state().into_view()
    }
}

impl PartialEq for TodoState {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
}

impl std::hash::Hash for TodoState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state);
    }
}

fn todo_input(todo: TodoState) -> impl IntoView {
    let input = todo.description;

    let is_active = create_memo(move |_| {
        TODOS_STATE.with(|s| s.active.with(|s| s.active.map_or(false, |v| v == todo)))
    });

    text_input(input)
        .disabled(move || !is_active.get())
        .placeholder("New To-Do")
        .style(move |s| {
            s.width_full()
                .apply_if(!is_active.get(), |s| s.cursor(CursorStyle::Default))
                .background(Color::TRANSPARENT)
                .transition_background(Transition::ease_in_out(600.millis()))
                .border(0)
                .hover(|s| s.background(Color::TRANSPARENT))
                .focus(|s| {
                    s.hover(|s| s.background(Color::TRANSPARENT))
                        .border(0.)
                        .border_color(Color::TRANSPARENT)
                })
                .disabled(|s| s.background(Color::TRANSPARENT).color(Color::BLACK))
                .class(PlaceholderTextClass, |s| s.color(Color::GRAY))
        })
        .on_key_down(
            floem::keyboard::Key::Named(floem::keyboard::NamedKey::Enter),
            |m| m.is_empty(),
            move |_| {
                AppCommand::Escape.execute();
            },
        )
}
