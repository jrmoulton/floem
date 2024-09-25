use std::fs;
use std::path::PathBuf;
use floem::event::EventListener;
use floem::peniko::Color;
use floem::reactive::{create_rw_signal, RwSignal, SignalGet};
use floem::style::TextOverflow;
use floem::View;
use floem::views::{Decorators, h_stack, label, static_label, v_stack};

pub struct TextDocument {
    path: PathBuf,
    content: String,
    selection: RwSignal<Option<(usize, usize)>>,
}

impl TextDocument {
    pub fn new(path: PathBuf) -> Self {

        let content = fs::read_to_string(&path).unwrap();

        Self {
            path,
            content,
            selection: create_rw_signal(None),
        }
    }

    pub fn build_view(&self) -> impl View {
        h_stack((
            //
            // info panel
            //
            v_stack((
                {
                    h_stack((
                        static_label("path"),
                        static_label(self.path.to_str().unwrap())
                            .style(|s|s
                                // FIXME doesn't make any difference, path appears truncated
                                .text_overflow(TextOverflow::Wrap)
                            )
                    ))
                },
                {
                    h_stack((
                        static_label("selection"),
                        {
                            // FIXME panics
                            let selection = self.selection.clone();
                            label(move || {
                                selection.get()
                                    .map_or_else(
                                        ||"None".to_string(),
                                        |(offset, length)|{
                                            format!("offset: {}, length: {}",   offset, length)
                                        }
                                    )
                            })
                        }
                    ))
                }
            ))
                .style(|s|s
                    .height_full()
                    .width_pct(20.0)
                ),

            //
            // content
            //
            {
                // FIXME this needs to be reactive
                let content_label = format!("{}", self.content);
                label(move || {
                    // FIXME do we really have to clone here? it's expensive and memory inefficient on a large file!
                    content_label.clone()

                    // FIXME text selection doesn't work properly on this element, there is strange behavior!
                })
                    .on_event_cont(EventListener::PointerUp,|event|{
                        println!("{:?}", event);
                        // TODO somehow see what text is now selected in the label, get and store the selection offset and length
                        //      in `self.selection`
                        let selection = (0,0);

                        // FIXME this doesn't compile: `self` escapes the method body here
                        //self.selection.set(Some(selection));
                    })
            }
                .style(|s|s
                    .height_full()
                    // FIXME if this is 80% or 'full' it still doesn't take up the remaining space.
                    .width_full()
                    .background(Color::DARK_GRAY)
                ),
        ))
            .style(|s|s
                .width_full()
                .height_full()
            )
    }
}
