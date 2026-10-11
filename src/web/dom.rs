use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use js_sys::{Date, Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    Document, DomRect, Element, Event, IntersectionObserver, IntersectionObserverEntry,
    IntersectionObserverInit, KeyboardEvent, Window,
};

pub fn window() -> Window {
    web_sys::window().expect("no window")
}

pub fn document() -> Document {
    window().document().expect("no document")
}

pub fn rect_of(selector: &str) -> Option<DomRect> {
    document().query_selector(selector).ok()?.map(|e| e.get_bounding_client_rect())
}

pub fn is_narrow() -> bool {
    window().inner_width().ok().and_then(|w| w.as_f64()).is_some_and(|w| w <= 680.0)
}

/// Scroll to the top and fade `.reveal` sections in as they enter the viewport.
pub fn page_effects() {
    thread_local! {
        static OBSERVER: RefCell<Option<(IntersectionObserver, Closure<dyn FnMut(js_sys::Array, IntersectionObserver)>)>> =
            const { RefCell::new(None) };
    }

    window().scroll_to_with_x_and_y(0.0, 0.0);

    let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
        |entries: js_sys::Array, observer: IntersectionObserver| {
            for entry in entries.iter() {
                let entry: IntersectionObserverEntry = entry.unchecked_into();
                if entry.is_intersecting() {
                    let target = entry.target();
                    let _ = target.class_list().add_1("in");
                    observer.unobserve(&target);
                }
            }
        },
    );
    let init = IntersectionObserverInit::new();
    init.set_threshold(&JsValue::from_f64(0.15));
    let Ok(observer) = IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &init) else {
        return;
    };
    if let Ok(nodes) = document().query_selector_all(".reveal") {
        for i in 0..nodes.length() {
            if let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                observer.observe(&el);
            }
        }
    }
    OBSERVER.with(|slot| {
        if let Some((old, _)) = slot.borrow_mut().replace((observer, callback)) {
            old.disconnect();
        }
    });
}

fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

/// Smooth-scrolls so the element with `id` sits at the top (instant with reduced motion).
pub fn scroll_to_id(id: &str) {
    let Some(el) = document().get_element_by_id(id) else { return };
    let win = window();
    let target = el.get_bounding_client_rect().top() + win.scroll_y().unwrap_or(0.0);
    let reduced = win
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches());
    if reduced {
        win.scroll_to_with_x_and_y(0.0, target);
        return;
    }

    let start_y = win.scroll_y().unwrap_or(0.0);
    let delta = target - start_y;
    let duration = 1100.0;
    let start = js_sys::Date::now();

    let slot: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::default();
    let next = slot.clone();
    *next.borrow_mut() = Some(Closure::new(move || {
        let t = ((Date::now() - start) / duration).min(1.0);
        window().scroll_to_with_x_and_y(0.0, start_y + delta * ease_in_out_cubic(t));
        if t < 1.0 {
            if let Some(f) = slot.borrow().as_ref() {
                let _ = window().request_animation_frame(f.as_ref().unchecked_ref());
            }
        } else {
            slot.borrow_mut().take();
        }
    }));
    let first_frame = next.borrow();
    if let Some(f) = first_frame.as_ref() {
        let _ = win.request_animation_frame(f.as_ref().unchecked_ref());
    }
}

/// Calls `on_dismiss` once on the next click outside `selector`, or on Escape.
pub fn dismiss_on_outside(selector: &'static str, on_dismiss: impl FnOnce() + 'static) {
    type Listeners = Rc<RefCell<Option<(EventListener, EventListener)>>>;
    let listeners: Listeners = Rc::default();
    let callback: Rc<RefCell<Option<Box<dyn FnOnce()>>>> = Rc::new(RefCell::new(Some(Box::new(on_dismiss))));

    let finish = {
        let listeners = listeners.clone();
        move || {
            let Some(cb) = callback.borrow_mut().take() else { return };
            // Remove the listeners after the current event finishes (not from inside their own callback).
            let listeners = listeners.clone();
            Timeout::new(0, move || drop(listeners.borrow_mut().take())).forget();
            cb();
        }
    };

    let doc = document();
    let on_down = {
        let finish = finish.clone();
        EventListener::new(&doc, "pointerdown", move |e: &Event| {
            let outside = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .map_or(true, |el| el.closest(selector).ok().flatten().is_none());
            if outside {
                finish();
            }
        })
    };
    let on_key = EventListener::new(&doc, "keydown", move |e: &Event| {
        if e.dyn_ref::<KeyboardEvent>().is_some_and(|k| k.key() == "Escape") {
            finish();
        }
    });
    *listeners.borrow_mut() = Some((on_down, on_key));
}

#[cfg(target_arch = "wasm32")]
pub fn current_year() -> u32 {
    Date::new_0().get_full_year()
}

/// Server side: civil-from-days (Howard Hinnant's algorithm) on the system clock.
#[cfg(not(target_arch = "wasm32"))]
pub fn current_year() -> u32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2)) as u32
}

/// `Date#toLocale…String(locale, options)` with a simple string options bag.
pub fn format_date(method: &str, locale: &str, options: &[(&str, &str)]) -> String {
    let date = Date::new_0();
    let opts = Object::new();
    for (k, v) in options {
        let _ = Reflect::set(&opts, &(*k).into(), &(*v).into());
    }
    Reflect::get(&date, &method.into())
        .ok()
        .and_then(|f| f.dyn_into::<Function>().ok())
        .and_then(|f| f.call2(&date, &locale.into(), &opts).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}
