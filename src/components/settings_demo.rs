use crate::hooks::{dismiss_on_outside, use_release, RELEASES_URL};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum Glyph {
    Gear,
    Island,
    Globe,
    Download,
    Refresh,
    Forward,
    Calendar,
    Clock,
    Window,
    Expand,
    Note,
    Bolt,
    Lock,
    Power,
}

#[derive(PartialEq)]
enum Kind {
    Toggle(bool),
    Select(&'static [&'static str]),
}

#[derive(PartialEq)]
struct Row {
    id: &'static str,
    title: &'static str,
    sub: Option<&'static str>,
    color: &'static str,
    glyph: Glyph,
    kind: Kind,
}

const fn row(id: &'static str, title: &'static str, sub: Option<&'static str>, color: &'static str, glyph: Glyph, kind: Kind) -> Row {
    Row { id, title, sub, color, glyph, kind }
}

static STARTUP: [Row; 1] = [row("login", "Launch at login", None, "#9a9a9f", Glyph::Power, Kind::Toggle(true))];
static UPDATE: [Row; 5] = [
    row("auto-check", "Check for updates automatically", None, "#9a9a9f", Glyph::Refresh, Kind::Toggle(true)),
    row("auto-install", "Install updates automatically", Some("Without asking — the app restarts when nothing is playing"), "#bf5af2", Glyph::Download, Kind::Toggle(true)),
    row("skip-small", "Skip small updates", Some("Only updates like 2.3, not 2.2.7 — Check Now still shows every update"), "#32d2e6", Glyph::Forward, Kind::Toggle(false)),
    row("often", "How often", None, "#ff453a", Glyph::Calendar, Kind::Select(&["Every hour", "Every day", "Every 2 days", "Every week"])),
    row("tod", "Time of day", None, "#ff9f0a", Glyph::Clock, Kind::Select(&["Morning (9:00 AM)", "Afternoon (2:00 PM)", "Evening (7:00 PM)"])),
];
static VISIBILITY: [Row; 2] = [
    row("hide-front", "Hide while the playing app is in front", None, "#3b82f6", Glyph::Window, Kind::Toggle(true)),
    row("fullscreen", "Show in fullscreen apps", None, "#40c8e0", Glyph::Expand, Kind::Toggle(true)),
];
static ANIMATIONS: [Row; 2] = [
    row("flip", "Flip the album art on track change", None, "#ff375f", Glyph::Note, Kind::Toggle(true)),
    row("charging", "Show the charging animation", None, "#30d158", Glyph::Bolt, Kind::Toggle(true)),
];
static LOCK: [Row; 1] = [row("lock-icon", "Show a lock icon on the lock screen", None, "#7d7aff", Glyph::Lock, Kind::Toggle(true))];
static LANGUAGE: [Row; 1] = [row("lang", "Language", None, "#3b82f6", Glyph::Globe, Kind::Select(&["System", "English", "Norsk", "עברית"]))];

fn all_rows() -> impl Iterator<Item = &'static Row> {
    STARTUP.iter().chain(&UPDATE).chain(&VISIBILITY).chain(&ANIMATIONS).chain(&LOCK).chain(&LANGUAGE)
}

const PANES: [(&str, &str, &str, Glyph); 3] = [
    ("general", "General", "#9a9a9f", Glyph::Gear),
    ("island", "Island", "#000", Glyph::Island),
    ("language", "Language", "#3b82f6", Glyph::Globe),
];

#[component]
fn GlyphIcon(glyph: Glyph) -> Element {
    rsx! {
        svg {
            view_box: "0 0 20 20", width: "14", height: "14", fill: "none", stroke: "#fff",
            stroke_width: "1.7", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            match glyph {
                Glyph::Gear => rsx! {
                    circle { cx: "10", cy: "10", r: "3" }
                    path { d: "M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" }
                },
                Glyph::Island => rsx! { rect { x: "2", y: "6.5", width: "16", height: "7", rx: "3.5", fill: "#fff", stroke: "none" } },
                Glyph::Globe => rsx! {
                    circle { cx: "10", cy: "10", r: "7.2" }
                    path { d: "M2.8 10h14.4M10 2.8c2.4 2.2 2.4 12.2 0 14.4M10 2.8c-2.4 2.2-2.4 12.2 0 14.4" }
                },
                Glyph::Download => rsx! { path { d: "M10 3.5v9M6.5 9.5 10 13l3.5-3.5M4.5 16h11" } },
                Glyph::Refresh => rsx! { path { d: "M15.5 8A6 6 0 0 0 5 6.5M4.5 12A6 6 0 0 0 15 13.5M5 3.5v3h3M15 16.5v-3h-3" } },
                Glyph::Forward => rsx! { path { d: "M3.5 5v10l6-5zM10.5 5v10l6-5z", fill: "#fff" } },
                Glyph::Calendar => rsx! {
                    rect { x: "3.5", y: "4.5", width: "13", height: "12", rx: "2" }
                    path { d: "M3.5 8h13M7 3v3M13 3v3" }
                },
                Glyph::Clock => rsx! {
                    circle { cx: "10", cy: "10", r: "6.8" }
                    path { d: "M10 6v4l2.5 1.5" }
                },
                Glyph::Window => rsx! {
                    rect { x: "3", y: "4", width: "14", height: "12", rx: "2" }
                    path { d: "M3 7.5h14" }
                },
                Glyph::Expand => rsx! { path { d: "M12 3.5h4.5V8M16.5 3.5 11 9M8 16.5H3.5V12M3.5 16.5 9 11" } },
                Glyph::Note => rsx! {
                    path { d: "M8 14.5V5l7-1.5v9" }
                    circle { cx: "6.5", cy: "14.5", r: "1.8", fill: "#fff" }
                    circle { cx: "13.5", cy: "12.5", r: "1.8", fill: "#fff" }
                },
                Glyph::Bolt => rsx! { path { d: "M11 2.5 5 11h4l-1 6.5L15 8.5h-4z", fill: "#fff" } },
                Glyph::Lock => rsx! {
                    rect { x: "4.5", y: "9", width: "11", height: "8", rx: "2", fill: "#fff", stroke: "none" }
                    path { d: "M7 9V7a3 3 0 0 1 6 0v2" }
                },
                Glyph::Power => rsx! { path { d: "M10 3v6M5.8 5.8a6 6 0 1 0 8.4 0" } },
            }
        }
    }
}

#[component]
fn Icon(color: &'static str, glyph: Glyph) -> Element {
    rsx! {
        span { class: "s-icon", style: "background: {color}", GlyphIcon { glyph } }
    }
}

#[component]
fn SettingRow(
    row: &'static Row,
    toggles: Signal<HashMap<&'static str, bool>>,
    picks: Signal<HashMap<&'static str, usize>>,
    open_menu: Signal<Option<&'static str>>,
) -> Element {
    let mut toggles = toggles;
    let mut picks = picks;
    let mut open_menu = open_menu;
    let id = row.id;

    rsx! {
        div { class: "s-row",
            Icon { color: row.color, glyph: row.glyph }
            div { class: "s-text",
                div { "{row.title}" }
                if let Some(sub) = row.sub {
                    small { "{sub}" }
                }
            }
            match &row.kind {
                Kind::Toggle(_) => {
                    let on = toggles.read().get(id).copied().unwrap_or(false);
                    rsx! {
                        button {
                            class: if on { "s-toggle on" } else { "s-toggle" },
                            role: "switch",
                            "aria-checked": "{on}",
                            "aria-label": row.title,
                            onclick: move |_| {
                                toggles.with_mut(|t| {
                                    let v = t.entry(id).or_insert(false);
                                    *v = !*v;
                                })
                            },
                            span {}
                        }
                    }
                }
                Kind::Select(options) => {
                    let picked = picks.read().get(id).copied().unwrap_or(0);
                    let is_open = open_menu() == Some(id);
                    rsx! {
                        span { class: "s-select-wrap",
                            button {
                                class: "s-select",
                                "aria-haspopup": "menu",
                                "aria-expanded": "{is_open}",
                                onclick: move |_| open_menu.set(if open_menu() == Some(id) { None } else { Some(id) }),
                                "{options[picked]}"
                                span { class: "s-updown", "aria-hidden": "true", "⌃⌄" }
                            }
                            if is_open {
                                div { class: "s-menu", role: "menu",
                                    for (i , o) in options.iter().enumerate() {
                                        button {
                                            key: "{o}",
                                            role: "menuitemradio",
                                            "aria-checked": "{picked == i}",
                                            onclick: move |_| {
                                                picks.with_mut(|p| {
                                                    p.insert(id, i);
                                                });
                                                open_menu.set(None);
                                            },
                                            span { class: "s-check", if picked == i { "✓" } }
                                            "{o}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Sub {
    Main,
    Update,
}

#[component]
pub fn SettingsDemo(on_close: EventHandler<()>, on_drag_start: EventHandler<Event<PointerData>>) -> Element {
    let release = use_release();
    let mut pane = use_signal(|| "general");
    let mut sub = use_signal(|| Sub::Main);
    let mut query = use_signal(String::new);
    let toggles = use_signal(|| {
        all_rows()
            .filter_map(|r| match r.kind {
                Kind::Toggle(on) => Some((r.id, on)),
                _ => None,
            })
            .collect::<HashMap<_, _>>()
    });
    let picks = use_signal(|| {
        all_rows()
            .filter(|r| matches!(r.kind, Kind::Select(_)))
            .map(|r| (r.id, if r.id == "often" || r.id == "lang" { 1 } else { 0 }))
            .collect::<HashMap<_, _>>()
    });
    let mut checked = use_signal(|| false);
    let mut open_menu = use_signal(|| None::<&'static str>);

    // Close an open dropdown on outside click or Escape.
    use_effect(move || {
        if open_menu().is_some() {
            dismiss_on_outside(".s-select-wrap", move || open_menu.set(None));
        }
    });

    let q = query().trim().to_lowercase();
    let searching = !q.is_empty();
    let results: Vec<&'static Row> = if searching {
        all_rows()
            .filter(|r| format!("{} {}", r.title, r.sub.unwrap_or("")).to_lowercase().contains(&q))
            .collect()
    } else {
        vec![]
    };

    let group = move |rows: &'static [Row]| {
        rsx! {
            div { class: "s-group",
                for r in rows.iter() {
                    SettingRow { key: "{r.id}", row: r, toggles, picks, open_menu }
                }
            }
        }
    };

    let version = release().version.unwrap_or_else(|| "2.2.1".to_string());

    let content = if searching {
        if results.is_empty() {
            rsx! { p { class: "s-empty", "No results for “{query}”" } }
        } else {
            rsx! {
                h4 { class: "s-head", "Results" }
                div { class: "s-group",
                    for r in results {
                        SettingRow { key: "{r.id}", row: r, toggles, picks, open_menu }
                    }
                }
            }
        }
    } else if pane() == "general" && sub() == Sub::Main {
        rsx! {
            div { class: "s-group s-about",
                div { class: "s-pill", "aria-hidden": "true",
                    i { class: "s-dot" }
                    span { class: "s-wave",
                        for i in 0..22 {
                            b { key: "{i}", style: "height: {20.0 + (i as f64 * 0.7).sin().abs() * 70.0}%" }
                        }
                    }
                }
                div { class: "s-text",
                    div { class: "s-name", "Hevel" }
                    small { "Version {version}" }
                }
                a { class: "s-link", href: RELEASES_URL, target: "_blank", rel: "noreferrer", "Release Notes" }
            }
            button { class: "s-group s-nav", onclick: move |_| sub.set(Sub::Update),
                div { class: "s-row",
                    Icon { color: "#3b82f6", glyph: Glyph::Download }
                    div { class: "s-text", div { "Software Update" } }
                    span { class: "s-chev", "aria-hidden": "true", "›" }
                }
            }
            h4 { class: "s-head", "Startup" }
            {group(&STARTUP)}
        }
    } else if pane() == "general" {
        rsx! {
            div { class: "s-group",
                div { class: "s-row",
                    Icon { color: "#3b82f6", glyph: Glyph::Download }
                    div { class: "s-text",
                        div { "Software Update" }
                        small {
                            if checked() { "You’re up to date" } else { "Last checked: Today at 1:18 AM · Next: Tomorrow at 9:00 AM" }
                        }
                    }
                    button { class: "s-btn", onclick: move |_| checked.set(true), "Check Now" }
                }
            }
            {group(&UPDATE)}
        }
    } else if pane() == "island" {
        rsx! {
            h4 { class: "s-head", "Visibility" }
            {group(&VISIBILITY)}
            h4 { class: "s-head", "Animations" }
            {group(&ANIMATIONS)}
            h4 { class: "s-head", "Lock Screen" }
            {group(&LOCK)}
        }
    } else {
        group(&LANGUAGE)
    };

    rsx! {
        div { class: "settings", role: "group", "aria-label": "Hevel settings preview",
            aside { class: "s-side",
                div { class: "s-lights draggable-area", onpointerdown: move |e| on_drag_start.call(e),
                    button { "aria-label": "Close settings", onpointerdown: |e| e.stop_propagation(), onclick: move |_| on_close.call(()) }
                    i {}
                    i {}
                }
                label { class: "s-search",
                    svg { view_box: "0 0 20 20", width: "14", height: "14", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", "aria-hidden": "true",
                        circle { cx: "8.5", cy: "8.5", r: "5.5" }
                        path { d: "m13 13 4 4" }
                    }
                    input {
                        value: "{query}",
                        oninput: move |e| query.set(e.value()),
                        placeholder: "Search",
                        "aria-label": "Search settings",
                    }
                }
                nav { class: "s-panes",
                    for (id , title , color , glyph) in PANES {
                        button {
                            key: "{id}",
                            class: if !searching && pane() == id { "on" } else { "" },
                            onclick: move |_| {
                                pane.set(id);
                                sub.set(Sub::Main);
                                query.set(String::new());
                            },
                            Icon { color, glyph }
                            "{title}"
                        }
                    }
                }
            }
            section { class: "s-main",
                header { class: "s-top draggable-area", onpointerdown: move |e| on_drag_start.call(e),
                    div { class: "s-arrows", onpointerdown: |e| e.stop_propagation(),
                        button { "aria-label": "Back", disabled: sub() == Sub::Main || searching, onclick: move |_| sub.set(Sub::Main), "‹" }
                        button { "aria-label": "Forward", disabled: true, "›" }
                    }
                    b { "Hevel" }
                }
                div { class: "s-content", {content} }
            }
        }
    }
}
