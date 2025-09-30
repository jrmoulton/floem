use parking_lot::Mutex;
use std::{collections::VecDeque, sync::Arc};

use crate::ext_event::{create_ext_action, ExtSendTrigger, EXT_EVENT_HANDLER};

pub struct ChannelSignal<T>(floem_reactive::ReadSignal<Option<T>>);
impl<T: std::clone::Clone> floem_reactive::SignalGet<Option<T>> for ChannelSignal<T> {
    fn id(&self) -> floem_reactive::ReactiveId {
        self.0.id()
    }
}
impl<T> floem_reactive::SignalWith<Option<T>> for ChannelSignal<T> {
    fn id(&self) -> floem_reactive::ReactiveId {
        self.0.id()
    }
}
impl<T> floem_reactive::SignalRead<Option<T>> for ChannelSignal<T> {
    fn id(&self) -> floem_reactive::ReactiveId {
        self.0.id()
    }
}
impl<T> Copy for ChannelSignal<T> {}
impl<T> Clone for ChannelSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Eq for ChannelSignal<T> {}
impl<T> PartialEq for ChannelSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        pub use floem_reactive::SignalWith;
        self.0.id() == other.0.id()
    }
}

#[cfg(feature = "tokio")]
impl<T: Send + 'static> From<tokio::sync::mpsc::UnboundedReceiver<T>> for ChannelSignal<T> {
    fn from(mut rx: tokio::sync::mpsc::UnboundedReceiver<T>) -> Self {
        use floem_reactive::*;
        use std::collections::VecDeque;

        use floem_reactive::{with_scope, Scope};

        let cx = Scope::new();
        let trigger = with_scope(cx, ExtSendTrigger::new);
        let channel_closed = cx.create_rw_signal(false);
        let (read, write) = cx.create_signal(None);
        let data = std::sync::Arc::new(std::sync::Mutex::new(VecDeque::new()));
        {
            let data = data.clone();
            cx.create_effect(move |_| {
                trigger.track();
                while let Some(value) = data.lock().unwrap().pop_front() {
                    write.set(value);
                }
                if channel_closed.get() {
                    cx.dispose();
                }
            });
        }
        let send = create_ext_action(cx, move |_| {
            channel_closed.set(true);
        });
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                data.lock().unwrap().push_back(Some(event));
                crate::ext_event::register_ext_trigger(trigger);
            }
            send(());
        });
        Self(read)
    }
}

#[cfg(feature = "crossbeam")]
impl<T: Send + 'static> From<crossbeam::channel::Receiver<T>> for ChannelSignal<T> {
    fn from(rx: crossbeam::channel::Receiver<T>) -> Self {
        use floem_reactive::*;
        let cx = Scope::new();
        let trigger = with_scope(cx, ExtSendTrigger::new);
        let channel_closed = cx.create_rw_signal(false);
        let (read, write) = cx.create_signal(None);
        let data = Arc::new(Mutex::new(VecDeque::new()));
        {
            let data = data.clone();
            cx.create_effect(move |_| {
                trigger.track();
                while let Some(value) = data.lock().pop_front() {
                    write.set(value);
                }
                if channel_closed.get() {
                    cx.dispose();
                }
            });
        }
        let send = create_ext_action(cx, move |_| {
            channel_closed.set(true);
        });
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                data.lock().push_back(Some(event));
                EXT_EVENT_HANDLER.add_trigger(trigger);
            }
            send(());
        });
        Self(read)
    }
}

impl<T: Send + 'static> From<std::sync::mpsc::Receiver<T>> for ChannelSignal<T> {
    fn from(rx: std::sync::mpsc::Receiver<T>) -> Self {
        use floem_reactive::*;
        let cx = Scope::new();
        let trigger = with_scope(cx, ExtSendTrigger::new);
        let channel_closed = cx.create_rw_signal(false);
        let (read, write) = cx.create_signal(None);
        let data = Arc::new(Mutex::new(VecDeque::new()));
        {
            let data = data.clone();
            cx.create_effect(move |_| {
                trigger.track();
                while let Some(value) = data.lock().pop_front() {
                    write.set(value);
                }
                if channel_closed.get() {
                    cx.dispose();
                }
            });
        }
        let send = create_ext_action(cx, move |_| {
            channel_closed.set(true);
        });
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                data.lock().push_back(Some(event));
                EXT_EVENT_HANDLER.add_trigger(trigger);
            }
            send(());
        });
        Self(read)
    }
}

#[cfg(feature = "tokio")]
impl<T> std::ops::Deref for ChannelSignal<T> {
    type Target = ReadSignal<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
