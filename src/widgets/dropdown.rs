use std::{any::Any, rc::Rc};

use floem_reactive::{as_child_of_current_scope, create_effect, create_updater, Scope};
use kurbo::{Point, Rect};
use taffy::geometry::Size;

use crate::{
    action::{add_overlay, remove_overlay},
    id::Id,
    style::{Style, StyleClass, Width},
    style_class,
    unit::PxPctAuto,
    view::{default_compute_layout, default_event, view_children_set_parent_id, View, ViewData},
    views::Decorators,
};

use super::list;

type ChildFn<T> = dyn Fn(T) -> (Box<dyn View>, Scope);

style_class!(pub DropDownClass);

pub struct DropDown<T: 'static> {
    view_data: ViewData,
    main_view: Box<dyn View>,
    main_view_scope: Scope,
    main_fn: Box<ChildFn<T>>,
    list_view: Rc<dyn Fn() -> Box<dyn View>>,
    list_style: Style,
    overlay_id: Option<Id>,
    window_origin: Option<Point>,
    size: Size<f32>,
}

enum DropDownMessage {
    OpenState(bool),
    ActiveElement(Box<dyn Any>),
}

impl<T: 'static> View for DropDown<T> {
    fn view_data(&self) -> &ViewData {
        &self.view_data
    }

    fn view_data_mut(&mut self) -> &mut ViewData {
        &mut self.view_data
    }

    fn for_each_child<'a>(&'a self, for_each: &mut dyn FnMut(&'a dyn View) -> bool) {
        for_each(&self.main_view);
    }

    fn for_each_child_mut<'a>(&'a mut self, for_each: &mut dyn FnMut(&'a mut dyn View) -> bool) {
        for_each(&mut self.main_view);
    }

    fn for_each_child_rev_mut<'a>(
        &'a mut self,
        for_each: &mut dyn FnMut(&'a mut dyn View) -> bool,
    ) {
        for_each(&mut self.main_view);
    }

    fn style(&mut self, cx: &mut crate::context::StyleCx<'_>) {
        cx.save();
        self.list_style =
            Style::new().apply_classes_from_context(&[super::ListClass::class_ref()], &cx.current);
        cx.restore();

        self.for_each_child_mut(&mut |child| {
            cx.style_view(child);
            false
        });
    }

    fn compute_layout(&mut self, cx: &mut crate::context::ComputeLayoutCx) -> Option<Rect> {
        if let Some(layout) = cx.get_layout(self.id()) {
            self.size = layout.size;
            if let PxPctAuto::Pct(pct) = self.list_style.get(Width) {
                self.list_style = self
                    .list_style
                    .clone()
                    .width(self.size.width as f64 * pct / 100.);
            }
        }
        self.window_origin = Some(cx.window_origin);

        default_compute_layout(self, cx)
    }

    fn update(&mut self, cx: &mut crate::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(state) = state.downcast::<DropDownMessage>() {
            match *state {
                DropDownMessage::OpenState(state) => {
                    if state {
                        if self.overlay_id.is_none() {
                            let point = self.window_origin.unwrap_or_default()
                                + (0., self.size.height as f64);
                            let list = self.list_view.clone();
                            let list_style = self.list_style.clone();
                            self.overlay_id = Some(add_overlay(point, move |_| {
                                list().style(move |s| s.apply(list_style.clone()))
                            }));
                        }
                    } else if let Some(id) = self.overlay_id {
                        remove_overlay(id);
                        self.overlay_id = None;
                    }
                }
                DropDownMessage::ActiveElement(val) => {
                    if let Ok(val) = val.downcast::<T>() {
                        let old_child_scope = self.main_view_scope;
                        cx.app_state_mut().remove_view(&mut self.main_view);
                        (self.main_view, self.main_view_scope) = (self.main_fn)(*val);
                        old_child_scope.dispose();
                        self.main_view.id().set_parent(self.id());
                        view_children_set_parent_id(&*self.main_view);
                        cx.request_all(self.id());
                    }
                }
            }
        }
    }

    fn event(
        &mut self,
        cx: &mut crate::context::EventCx,
        id_path: Option<&[Id]>,
        event: crate::event::Event,
    ) -> crate::EventPropagation {
        #[allow(clippy::single_match)]
        match event {
            crate::event::Event::PointerDown(_) => {
                if self.overlay_id.is_some() {
                    self.id().update_state(DropDownMessage::OpenState(false));
                } else {
                    self.id().request_layout();
                    self.id().update_state(DropDownMessage::OpenState(true));
                }
            }
            _ => {}
        }
        default_event(self, cx, id_path, event.clone())
    }
}

pub fn dropdown<MF, I, T, V2, AIF>(main_view: MF, iterator: I, active_item: AIF) -> DropDown<T>
where
    MF: Fn(T) -> Box<dyn View> + 'static,
    I: IntoIterator<Item = V2> + Clone + 'static,
    V2: View + 'static,
    T: Clone + 'static,
    AIF: Fn() -> T + 'static,
{
    let dropdown_id = Id::next();

    let list_view = Rc::new(move || {
        let iterator = iterator.clone();
        Box::new(list(iterator)) as Box<dyn View>
    });

    let initial = create_updater(active_item, move |new_state| {
        dropdown_id.update_state(DropDownMessage::ActiveElement(Box::new(new_state)));
    });

    let main_fn = Box::new(as_child_of_current_scope(main_view));

    let (child, main_view_scope) = main_fn(initial);

    DropDown {
        view_data: ViewData::new(dropdown_id),
        main_view: Box::new(child),
        main_view_scope,
        main_fn,
        list_view,
        list_style: Style::new(),
        overlay_id: None,
        window_origin: None,
        size: Size::ZERO,
    }
    .keyboard_navigatable()
    .class(DropDownClass)
}

impl<T> DropDown<T> {
    pub fn show_list(self, show: impl Fn() -> bool + 'static) -> Self {
        let id = self.id();
        create_effect(move |_| {
            let state = show();
            id.update_state(DropDownMessage::OpenState(state));
        });
        self
    }
}

impl<T> Drop for DropDown<T> {
    fn drop(&mut self) {
        if let Some(id) = self.overlay_id {
            remove_overlay(id)
        }
    }
}
