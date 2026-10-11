use super::{apple_logo::AppleLogo, settings_demo::SettingsDemo};
use crate::hooks::{dismiss_on_outside, use_release};
use crate::web::{
    audio::Player,
    cover::cover_gradient,
    dom::{format_date, is_narrow, rect_of},
};
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use gloo_timers::future::TimeoutFuture;
use gloo_events::EventListener;
use std::{cell::RefCell, rc::Rc};

const SONG: Asset = asset!("/assets/song.mp3");
const COVER: Asset = asset!("/assets/cover.jpg");

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Locked,
    Idle,
    Music,
    Charging,
    Settings,
}

const MODES: [(Mode, &str); 5] = [
    (Mode::Locked, "Lock screen"),
    (Mode::Idle, "Idle"),
    (Mode::Charging, "Charging"),
    (Mode::Music, "Track"),
    (Mode::Settings, "Settings"),
];

const NOTCH_W: u32 = 190;
const SIDE: u32 = 58;

/// Unlock intro: idle -> charging -> track -> settings, then the picker bar appears.
const INTRO: [(Mode, u32); 4] = [
    (Mode::Idle, 2800),
    (Mode::Charging, 4200),
    (Mode::Music, 10000),
    (Mode::Settings, 0),
];

fn battery_color(level: u32) -> &'static str {
    if level >= 80 {
        "#30d158"
    } else if level > 20 {
        "#ffd60a"
    } else {
        "#ff453a"
    }
}

#[component]
fn ChargingRing(level: u32) -> Element {
    let c = 2.0 * std::f64::consts::PI * 10.0;
    let offset = c * (1.0 - level as f64 / 100.0);
    let color = battery_color(level);
    rsx! {
        svg { width: "26", height: "26", view_box: "0 0 26 26", "aria-label": "{level}% charged",
            circle { cx: "13", cy: "13", r: "10", fill: "none", stroke: "rgba(255,255,255,.18)", stroke_width: "2.5" }
            circle {
                cx: "13", cy: "13", r: "10", fill: "none", stroke: color, stroke_width: "2.5", stroke_linecap: "round",
                stroke_dasharray: "{c}", stroke_dashoffset: "{offset}", transform: "rotate(-90 13 13)",
                style: "transition: stroke-dashoffset .6s, stroke .6s",
            }
            path { d: "M14 6.5 9.5 14h3l-1 5.5L16.5 12h-3z", fill: color, style: "transition: fill .6s" }
        }
    }
}

#[component]
fn LockIcon() -> Element {
    rsx! {
        svg { width: "14", height: "16", view_box: "0 0 14 16", "aria-label": "Locked", fill: "#fff",
            rect { x: "1", y: "6.5", width: "12", height: "9", rx: "2.2" }
            path { d: "M3.8 6.5V4.8a3.2 3.2 0 0 1 6.4 0v1.7", fill: "none", stroke: "#fff", stroke_width: "1.7", stroke_linecap: "round" }
        }
    }
}

#[component]
fn Equalizer(playing: bool, a: String, b: String) -> Element {
    rsx! {
        div {
            class: if playing { "eq" } else { "eq paused" },
            style: "--eq-a: {a}; --eq-b: {b}",
            "aria-hidden": "true",
            for i in 0..5 {
                span { key: "{i}", style: "animation-delay: {i as f64 * 0.13}s" }
            }
        }
    }
}

#[derive(Clone, Default)]
struct Clock {
    time: String,
    date: String,
    menu_date: String,
}

impl Clock {
    fn now() -> Self {
        Clock {
            time: format_date("toLocaleTimeString", "en-GB", &[("hour", "2-digit"), ("minute", "2-digit")]),
            date: format_date("toLocaleDateString", "en-US", &[("weekday", "long"), ("day", "numeric"), ("month", "long")]),
            menu_date: format_date("toLocaleDateString", "en-US", &[("weekday", "short"), ("day", "numeric"), ("month", "short")]),
        }
    }
}

fn use_clock() -> Signal<Clock> {
    // Empty on the server and during hydration; filled in (and ticking) once the client is running.
    let mut clock = use_signal(Clock::default);
    use_effect(move || {
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                clock.set(Clock::now());
            }
        });
    });
    clock
}

fn bump(gen: &mut Signal<u32>) -> u32 {
    let next = *gen.peek() + 1;
    gen.set(next);
    next
}

#[component]
pub fn NotchDemo() -> Element {
    let mut mode = use_signal(|| Mode::Locked);
    let mut level = use_signal(|| 76u32);
    let playing = use_signal(|| false);
    let mut eq = use_signal(|| ("#ff8a65".to_string(), "#ffb74d".to_string()));
    let mut intro_running = use_signal(|| false);
    let mut show_tabs = use_signal(|| false);
    let mut intro_gen = use_signal(|| 0u32);
    let mut charge_gen = use_signal(|| 0u32);
    let mut menu_open = use_signal(|| false);
    let mut update_note = use_signal(|| false);
    let mut offset = use_signal(|| (0.0f64, 0.0f64));
    let drag_listeners = use_hook(|| Rc::new(RefCell::new(None::<(EventListener, EventListener)>)));
    let clock = use_clock();
    let release = use_release();

    // Gapless looping player (Web Audio); it reports play/pause back here.
    // Created on the client only: it needs the browser's AudioContext.
    let mut player = use_signal(|| None::<Player>);
    use_effect(move || {
        if player.peek().is_none() {
            player.set(Some(Player::new(&SONG.to_string(), 0.4, move |p| {
                let mut playing = playing;
                playing.set(p);
            })));
        }
    });
    use_drop(move || {
        if let Ok(p) = player.try_peek() {
            if let Some(p) = p.as_ref() {
                p.dispose();
            }
        }
    });

    use_effect(move || {
        spawn(async move {
            if let Some(colors) = cover_gradient(&COVER.to_string()).await {
                eq.set(colors);
            }
        });
    });

    // The battery level climbs while the Charging state is showing.
    use_effect(move || {
        if mode() != Mode::Charging {
            bump(&mut charge_gen);
            return;
        }
        level.set(76);
        let g = bump(&mut charge_gen);
        spawn(async move {
            loop {
                TimeoutFuture::new(600).await;
                if *charge_gen.peek() != g {
                    break;
                }
                level.with_mut(|l| *l = (*l + 1).min(90));
            }
        });
    });

    // Close the app menu on outside click or Escape.
    use_effect(move || {
        if menu_open() {
            dismiss_on_outside(".menu-wrap", move || menu_open.set(false));
        }
    });

    let mut select = move |next: Mode| {
        mode.set(next);
        if let Some(p) = player.peek().as_ref() {
            p.select(next == Mode::Music);
        }
    };

    // A manual choice (menu bar, tabs, window) ends the unlock intro early and reveals the picker.
    let mut choose = move |next: Mode| {
        bump(&mut intro_gen);
        intro_running.set(false);
        show_tabs.set(true);
        select(next);
    };

    let mut unlock = move || {
        if mode() != Mode::Locked || intro_running() {
            return;
        }
        if show_tabs() {
            return select(Mode::Idle);
        }
        // Unlock audio inside this click so the later, timer-driven play() is allowed.
        if let Some(p) = player.peek().as_ref() {
            p.unlock();
        }
        intro_running.set(true);
        let g = bump(&mut intro_gen);
        spawn(async move {
            for (i, (step, ms)) in INTRO.into_iter().enumerate() {
                if *intro_gen.peek() != g {
                    return;
                }
                select(step);
                if i == INTRO.len() - 1 {
                    intro_running.set(false);
                    show_tabs.set(true);
                }
                if ms > 0 {
                    TimeoutFuture::new(ms).await;
                }
            }
        });
    };

    // Drag the settings window by its title areas, keeping it inside the screen.
    let start_drag = move |e: Event<PointerData>| {
        if is_narrow() {
            return;
        }
        let (Some(win), Some(bounds)) = (rect_of(".settings-drag"), rect_of(".settings-window")) else { return };
        e.prevent_default();
        let start = e.client_coordinates();
        let (ox, oy) = *offset.peek();
        let (min_x, max_x) = (ox + bounds.left() - win.left(), ox + bounds.right() - win.right());
        let (min_y, max_y) = (oy + bounds.top() - win.top(), oy + bounds.bottom() - win.bottom());

        let doc = crate::web::dom::document();
        let on_move = EventListener::new(&doc, "pointermove", move |ev| {
            let Some(ev) = ev.dyn_ref::<web_sys::MouseEvent>() else { return };
            let x = (ox + ev.client_x() as f64 - start.x).clamp(min_x, max_x);
            let y = (oy + ev.client_y() as f64 - start.y).clamp(min_y, max_y);
            offset.set((x, y));
        });
        let listeners = drag_listeners.clone();
        let on_up = EventListener::new(&doc, "pointerup", move |_| {
            let listeners = listeners.clone();
            gloo_timers::callback::Timeout::new(0, move || drop(listeners.borrow_mut().take())).forget();
        });
        *drag_listeners.borrow_mut() = Some((on_move, on_up));
    };

    let check_for_updates = move |_| {
        update_note.set(true);
        spawn(async move {
            TimeoutFuture::new(1300).await;
            menu_open.set(false);
            update_note.set(false);
        });
    };

    let m = mode();
    let unlocked = m != Mode::Locked;
    let wide = m != Mode::Idle && m != Mode::Settings;
    let width = if wide { NOTCH_W + SIDE * 2 } else { NOTCH_W };
    let clock = clock();
    let (eq_a, eq_b) = eq();
    let version = release().version.unwrap_or_else(|| "2.2.1".to_string());
    let lvl = level();
    let (k_lock, k_cover, k_pct) = ("lock", "cover", "pct");
    let lvl_color = battery_color(lvl);
    let lock_opacity = if unlocked { 0 } else { 1 };
    let bar_opacity = if unlocked { 1 } else { 0 };
    let bar_events = if unlocked { "auto" } else { "none" };
    let (art_opacity, art_scale) = if playing() { (1.0, 1.0) } else { (0.5, 0.8) };

    rsx! {
        div { class: "demo rise", style: "--d: .4s",
            div {
                class: if unlocked { "screen unlocked" } else { "screen clickable" },
                onclick: move |_| unlock(),
                role: if unlocked { "" } else { "button" },
                tabindex: if unlocked { "-1" } else { "0" },
                "aria-label": if unlocked { "" } else { "Click to unlock" },
                onkeydown: move |e| {
                    if matches!(e.key(), Key::Enter) || e.key() == Key::Character(" ".into()) {
                        unlock();
                    }
                },
                div { class: "lock-layer", style: "opacity: {lock_opacity}",
                    div { class: if clock.time.is_empty() { "lock-clock" } else { "lock-clock ready" },
                        div { class: "lock-date", "{clock.date}" }
                        div { class: "lock-time", "{clock.time}" }
                    }
                    div { class: "lock-hint", "Click to unlock" }
                }
                div {
                    class: "menubar",
                    style: "opacity: {bar_opacity}; pointer-events: {bar_events}",
                    AppleLogo {}
                    b { "Finder" }
                    span { "File" }
                    span { "Edit" }
                    span { "View" }
                    span { class: "spacer" }
                    span { class: "menu-wrap",
                        button {
                            class: if menu_open() { "menubar-app open" } else { "menubar-app" },
                            "aria-label": "Hevel menu",
                            "aria-expanded": "{menu_open()}",
                            onclick: move |e| {
                                e.stop_propagation();
                                menu_open.toggle();
                            },
                            svg { view_box: "0 0 20 12", width: "18", height: "11", "aria-hidden": "true",
                                rect { x: "1", y: "1", width: "18", height: "10", rx: "5", fill: "currentColor" }
                            }
                        }
                        if menu_open() {
                            div { class: "app-menu", role: "menu", onclick: move |e| e.stop_propagation(),
                                div { class: "app-menu-label", "Hevel {version}" }
                                hr {}
                                button {
                                    role: "menuitem",
                                    onclick: move |_| {
                                        menu_open.set(false);
                                        choose(Mode::Settings);
                                    },
                                    svg {
                                        view_box: "0 0 20 20", width: "15", height: "15", fill: "none", stroke: "currentColor",
                                        stroke_width: "1.6", stroke_linecap: "round", "aria-hidden": "true",
                                        circle { cx: "10", cy: "10", r: "3" }
                                        path { d: "M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" }
                                    }
                                    "Settings…"
                                    kbd { "⌘ ," }
                                }
                                button { role: "menuitem", disabled: update_note(), onclick: check_for_updates,
                                    if update_note() { "You’re up to date" } else { "Check for Updates…" }
                                }
                                hr {}
                                button { role: "menuitem", onclick: move |_| menu_open.set(false),
                                    "Quit"
                                    kbd { "⌘ Q" }
                                }
                            }
                        }
                    }
                    span { "{clock.menu_date}" }
                    span { "{clock.time}" }
                }

                div {
                    class: if m == Mode::Settings { "settings-window open" } else { "settings-window" },
                    onclick: move |e| e.stop_propagation(),
                    div { class: "settings-drag", style: "transform: translate({offset().0}px, {offset().1}px)",
                        SettingsDemo { on_close: move |_| choose(Mode::Idle), on_drag_start: start_drag }
                    }
                }

                div { class: "island", style: "width: {width}px",
                    div { class: "island-content", style: "padding-inline: 10px",
                        div { class: "side", style: "width: {SIDE - 10}px",
                            match m {
                                Mode::Locked => rsx! {
                                    span { key: "{k_lock}", class: "fade-in", style: "display: flex", LockIcon {} }
                                },
                                Mode::Music => rsx! {
                                    img {
                                        key: "{k_cover}",
                                        class: "art",
                                        src: COVER,
                                        alt: "",
                                        style: "opacity: {art_opacity}; transform: scale({art_scale})",
                                    }
                                },
                                Mode::Charging => rsx! {
                                    span { key: "{k_pct}", class: "pct fade-in", style: "color: {lvl_color}", "{lvl}%" }
                                },
                                _ => rsx! {},
                            }
                        }
                        div { style: "width: {NOTCH_W}px" }
                        div { class: "side right", style: "width: {SIDE - 10}px",
                            if m == Mode::Music {
                                Equalizer { playing: playing(), a: eq_a, b: eq_b }
                            }
                            if m == Mode::Charging {
                                ChargingRing { level: lvl }
                            }
                        }
                    }
                }
            }

            // Collapsed until the intro ends; opening it slides everything below (the download button) down.
            div { class: if show_tabs() { "tabs-slot open" } else { "tabs-slot" },
                if show_tabs() {
                    div { class: "tabs", role: "tablist", "aria-label": "Demo state",
                        for (id , label) in MODES {
                            button {
                                key: "{label}",
                                role: "tab",
                                "aria-selected": "{m == id}",
                                class: if m == id { "on" } else { "" },
                                onclick: move |_| choose(id),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
