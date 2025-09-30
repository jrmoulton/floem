use floem::{ext_event::ChannelSignal, reactive::*};
use floem::{prelude::*, taffy::FlexDirection};

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");

    let (mpsc_tx, mpsc_rx) = tokio::sync::mpsc::channel::<i32>(16);
    let (mpsc_unbound_tx, mpsc_unbound_rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
    let (broad_tx, broad_rx) = tokio::sync::broadcast::channel::<i32>(16);

    // We must run floem app in context of tokio runtime, so that
    // things like `tokio::spawn` will work.
    // It is imperative to make sure that floem runs on the main thread.
    runtime.block_on(async {
        {
            // channels will be closed after loop reaches its end

            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            tokio::spawn(async move {
                for i in 0..=8 {
                    interval.tick().await;
                    broad_tx.send(i).expect("send");
                    mpsc_tx.send(i).await.expect("send");
                    mpsc_unbound_tx.send(i).expect("send");
                }
            });
        }

        tokio::task::block_in_place(|| {
            floem::launch(move || {
                app_view(AppViewOpts {
                    broad_rx,
                    mpsc_rx,
                    mpsc_unbound_rx,
                })
            })
        })
    });
}

struct AppViewOpts {
    broad_rx: tokio::sync::broadcast::Receiver<i32>,
    mpsc_rx: tokio::sync::mpsc::Receiver<i32>,
    mpsc_unbound_rx: tokio::sync::mpsc::UnboundedReceiver<i32>,
}

fn app_view(opts: AppViewOpts) -> impl IntoView {
    v_stack((
        view_broadcast(opts.broad_rx),
        view_mpsc(opts.mpsc_rx),
        view_mpsc_unbound(opts.mpsc_unbound_rx),
    ))
    .style(|s| {
        s.size_full()
            .flex_direction(FlexDirection::Column)
            .items_stretch()
            .gap(30)
            .padding(10)
    })
}

fn view_broadcast(rx: tokio::sync::broadcast::Receiver<i32>) -> impl IntoView {
    let signal0 = ChannelSignal::from(rx.resubscribe());
    let signal1 = ChannelSignal::from(rx);

    v_stack((
        text("tokio broadcast receiver:").style(|s| s.font_bold()),
        label(move || format!("orig  rx: {:?}", signal0.get())),
        label(move || format!("resub rx: {:?}", signal1.get())),
    ))
    .style(|s| s.gap(10))
}

fn view_mpsc(rx: tokio::sync::mpsc::Receiver<i32>) -> impl IntoView {
    let signal = ChannelSignal::from(rx);

    v_stack((
        text("tokio mpsc receiver:").style(|s| s.font_bold()),
        label(move || format!("rx: {:?}", signal.get())),
    ))
    .style(|s| s.gap(10))
}

fn view_mpsc_unbound(rx: tokio::sync::mpsc::UnboundedReceiver<i32>) -> impl IntoView {
    let signal = ChannelSignal::from(rx);

    v_stack((
        text("tokio unbounded mpsc receiver:").style(|s| s.font_bold()),
        label(move || format!("{:?}", signal.get())),
    ))
    .style(|s| s.gap(10))
}
