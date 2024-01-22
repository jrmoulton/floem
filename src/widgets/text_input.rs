use crate::{
    style_class,
    view::View,
    views::{self, Decorators, TextInput, InputStateMsg},
};

use floem_reactive::RwSignal;
use floem_reactive::{create_effect, ReadSignal};

style_class!(pub TextInputClass);
style_class!(pub PlaceholderTextClass);

pub fn text_input(buffer: RwSignal<String>) -> TextInput {
    views::text_input(buffer).class(TextInputClass)
}

pub fn password_input(buffer: RwSignal<String>) -> TextInput {
    views::text_input(buffer).class(TextInputClass)
}

impl TextInput {
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder_buff = Some(text.into());
        self
    }

    pub fn password(self, is_password_fn: impl Fn() -> bool + 'static) -> Self {
        let id = self.id();
        create_effect(move |_| {
            let is_password = is_password_fn();
            id.update_state(InputStateMsg::Password(is_password), false);
        });
        self
    }
}
