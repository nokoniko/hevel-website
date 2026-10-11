use crate::web::dom::{document, scroll_to_id, window};
use dioxus::prelude::*;
use gloo_events::EventListener;
use std::{cell::RefCell, rc::Rc};

#[component]
pub fn ScrollCue(target_id: &'static str) -> Element {
    let mut hidden = use_signal(|| false);

    // Client only (effects never run during SSR). The listener lives as long as the component.
    let listener = use_hook(|| Rc::new(RefCell::new(None::<EventListener>)));
    use_effect(move || {
        let mut check = move || {
            if window().scroll_y().unwrap_or(0.0) > 10.0 {
                hidden.set(true);
            }
        };
        check();
        *listener.borrow_mut() = Some(EventListener::new(&document(), "scroll", move |_| check()));
    });

    rsx! {
        button {
            class: if hidden() { "scroll-cue gone" } else { "scroll-cue" },
            "aria-label": "Scroll to features",
            tabindex: if hidden() { "-1" } else { "0" },
            onclick: move |_| scroll_to_id(target_id),
            svg {
                view_box: "0 0 24 24", width: "30", height: "30", fill: "none", stroke: "currentColor",
                stroke_width: "2.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                path { d: "M5 9l7 7 7-7" }
            }
        }
    }
}
