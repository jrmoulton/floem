use std::rc::Rc;

use floem_reactive::create_effect;
use kurbo::{Point, Rect};

use crate::{
    action::{add_overlay, remove_overlay},
    id::Id,
    style_class,
    view::{default_compute_layout, default_event, View, ViewData},
    views::{dyn_container, Decorators},
};

use super::list;

style_class!(pub DropDownListClass);
style_class!(pub DropDownClass);

pub struct DropDown {
    view_data: ViewData,
    main_view: Box<dyn View>,
    list_view: Rc<dyn Fn() -> Box<dyn View>>,
    overlay_id: Option<Id>,
    window_origin: Option<Point>,
    layout_rect: Option<Rect>,
}

impl View for DropDown {
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

    fn compute_layout(&mut self, cx: &mut crate::context::ComputeLayoutCx) -> Option<Rect> {
        self.window_origin = Some(cx.window_origin);
        let layout_rect = default_compute_layout(self, cx);
        self.layout_rect = layout_rect;
        layout_rect
    }

    fn update(&mut self, _cx: &mut crate::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(state) = state.downcast::<bool>() {
            if *state {
                if self.overlay_id.is_none() {
                    let layout = self.layout_rect.unwrap_or_default();
                    let point = self.window_origin.unwrap_or_default() + (0., layout.height());
                    let list = self.list_view.clone();
                    self.overlay_id = Some(add_overlay(point, move |_| {
                        list().style(move |s| s.width(layout.width()))
                    }));
                }
            } else if let Some(id) = self.overlay_id {
                remove_overlay(id);
                self.overlay_id = None;
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
                if let Some(id) = self.overlay_id {
                    remove_overlay(id);
                    self.overlay_id = None;
                } else {
                    let layout = self.layout_rect.unwrap();
                    let point = self.window_origin.unwrap_or_default() + (0., layout.height());

                    let list = self.list_view.clone();
                    self.overlay_id = Some(add_overlay(point, move |_| {
                        list().style(move |s| s.width(layout.width()))
                    }));
                }
            }
            _ => {}
        }
        default_event(self, cx, id_path, event.clone())
    }
}

pub fn dropdown<MF, V1, I, T, V2, AIF>(main_view: MF, iterator: I, active_item: AIF) -> DropDown
where
    MF: Fn(T) -> V1 + 'static,
    I: IntoIterator<Item = V2> + Clone + 'static,
    V1: View + 'static,
    V2: View + 'static,
    T: Clone + 'static,
    AIF: Fn() -> T + 'static,
{
    let list_view = Rc::new(move || {
        let iterator = iterator.clone();
        Box::new(list(iterator).class(DropDownListClass)) as Box<dyn View>
    });

    let main_view = dyn_container(active_item, move |item| {
        Box::new(main_view(item).class(DropDownClass))
    })
    .keyboard_navigatable();

    DropDown {
        view_data: ViewData::new(Id::next()),
        main_view: Box::new(main_view),
        list_view,
        overlay_id: None,
        window_origin: None,
        layout_rect: None,
    }
}

impl DropDown {
    pub fn show_list(self, show: impl Fn() -> bool + 'static) -> Self {
        let id = self.id();
        create_effect(move |_| {
            let state = show();
            id.update_state(state);
        });
        self
    }
}

impl Drop for DropDown {
    fn drop(&mut self) {
        if let Some(id) = self.overlay_id {
            remove_overlay(id)
        }
    }
}
