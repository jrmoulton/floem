use std::{collections::VecDeque, sync::Arc};

use crate::ext_event::{create_ext_action, ExtSendTrigger, EXT_EVENT_HANDLER};

/// A reactive resource which receives values from the channel.
///
/// `ChannelSignal` provides a way to reactively receive data from the channel,
/// and does trigger a reactive update for each of the received values.
///
/// The signal will be updated once for every received value in the exact
/// same order as they are being received from the channel, thus
/// there is no guarantee that the `ChannelSignal` state is instantly
/// consistent with the latest value sent to the channel.
///
/// When the `ChannelSignal` is created its state is `None`.
///
/// When the channel is closed the signal shall update up to the last
/// received value and then its state will be set to `None`.
///
/// # Supported channel types
///
/// - `std::sync::mpsc::Receiver`
///
/// do not require additional features
///
/// - `tokio::sync::mpsc::Receiver`
/// - `tokio::sync::mpsc::UnboundedReceiver`
/// - `tokio::sync::broadcast::Receiver`
///
/// require `tokio` feature flag
///
/// - `crossbeam::channel::Receiver`
///
/// require `crossbeam` feature flag
///
/// # Concurrency
///
/// `ChannelSignal` handles received values in the exact same order as they are
/// being _received_ by provided receiver without any further implications.
///
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
impl<T: Send + 'static> ChannelSignal<T> {
    /// Create the `ChannelSignal` from a future, which shall write
    /// values sequentially to the given buffer.
    ///
    /// This is useful for adpoting unsupported channel types, as well as
    /// for any other asynchronously produced sequential data.
    ///
    /// Future must write values to the end of deque and register the trigger
    /// after every write.
    ///
    /// The trigger should be registered after each push to the deque,
    /// however it is not mandatory: trigger register shall lead to a series of
    /// signal updates untill the decue is exhausted.
    ///
    /// # Example
    ///
    /// ```ignore ("requires tokio rt-multi-thread")
    /// # use floem::ext_event::ChannelSignal;
    /// # let seq = vec![1,2,3,4,5];
    /// # let mut sequence = seq.into_iter();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// // Requires tokio runtime context
    /// ChannelSignal::from_async_sequential(move |queue, trigger|
    ///     async move {
    ///         while let Some(value) = sequence.next() {
    ///             queue.lock()
    ///                 .expect("ChannelSignal lock")
    ///                 .push_back(value);
    ///             floem::ext_event::register_ext_trigger(trigger);
    ///         }
    ///     }
    /// );
    /// # });
    /// ```
    pub fn from_async_sequential<Fut>(
        channel: impl FnOnce(Arc<std::sync::Mutex<VecDeque<T>>>, ExtSendTrigger) -> Fut,
    ) -> Self
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        use floem_reactive::*;
        use std::collections::VecDeque;

        let (read, write) = with_scope(Scope::current(), || create_signal(None));

        let cx = Scope::new();
        let trigger = with_scope(cx, ExtSendTrigger::new);
        let channel_closed = cx.create_rw_signal(false);
        let data = std::sync::Arc::new(std::sync::Mutex::new(VecDeque::new()));
        {
            let data = data.clone();
            cx.create_effect(move |_| {
                trigger.track();
                while let Some(value) = data.lock().unwrap().pop_front() {
                    write.set(Some(value));
                }
                if channel_closed.get() {
                    write.set(None);
                    cx.dispose();
                }
            });
        }
        let close = create_ext_action(cx, move |_| {
            channel_closed.set(true);
        });

        let fut = channel(data.clone(), trigger);
        tokio::spawn(async move {
            fut.await;
            close(())
        });

        Self(read)
    }
}

#[cfg(feature = "tokio")]
impl<T: Send + 'static> From<tokio::sync::mpsc::UnboundedReceiver<T>> for ChannelSignal<T> {
    fn from(mut rx: tokio::sync::mpsc::UnboundedReceiver<T>) -> Self {
        Self::from_async_sequential(move |data, trigger| async move {
            while let Some(event) = rx.recv().await {
                data.lock().unwrap().push_back(event);
                crate::ext_event::register_ext_trigger(trigger);
            }
        })
    }
}

#[cfg(feature = "tokio")]
impl<T: Send + 'static> From<tokio::sync::mpsc::Receiver<T>> for ChannelSignal<T> {
    fn from(mut rx: tokio::sync::mpsc::Receiver<T>) -> Self {
        Self::from_async_sequential(move |data, trigger| async move {
            while let Some(event) = rx.recv().await {
                data.lock().unwrap().push_back(event);
                crate::ext_event::register_ext_trigger(trigger);
            }
        })
    }
}

#[cfg(feature = "tokio")]
impl<T: Send + Clone + 'static> From<tokio::sync::broadcast::Receiver<T>>
    for ChannelSignal<Result<T, tokio::sync::broadcast::error::RecvError>>
{
    fn from(mut rx: tokio::sync::broadcast::Receiver<T>) -> Self {
        Self::from_async_sequential(move |data, trigger| async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        data.lock().unwrap().push_back(Ok(event));
                        crate::ext_event::register_ext_trigger(trigger);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    Err(e) => {
                        data.lock().unwrap().push_back(Err(e));
                        crate::ext_event::register_ext_trigger(trigger);
                    }
                }
            }
        })
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
        let data = Arc::new(parking_lot::Mutex::new(VecDeque::new()));
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
        let data = Arc::new(std::sync::Mutex::new(VecDeque::new()));
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
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                data.lock().unwrap().push_back(Some(event));
                EXT_EVENT_HANDLER.add_trigger(trigger);
            }
            send(());
        });
        Self(read)
    }
}

#[cfg(feature = "tokio")]
impl<T> std::ops::Deref for ChannelSignal<T> {
    type Target = floem_reactive::ReadSignal<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
