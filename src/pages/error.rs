use crate::Route;
use dioxus::prelude::*;

/// HTTP status codes with a user-facing title and message. `/error/<code>` renders each of them
/// (pre-rendered at build time), so any host can point its error pages at them.
pub const ERROR_CODES: [(u16, &str, &str); 12] = [
    (400, "Bad request", "The request couldn’t be understood. Check the address and try again."),
    (401, "Sign-in required", "You need to be signed in to see this. Hevel has no accounts, so this is probably a mistake."),
    (403, "Forbidden", "You don’t have permission to open this page."),
    (404, "Page not found", "That page doesn’t exist, or it has moved."),
    (405, "Method not allowed", "This address doesn’t accept that kind of request."),
    (408, "Request timeout", "The request took too long. Check your connection and try again."),
    (410, "Gone", "This page has been removed and isn’t coming back."),
    (429, "Too many requests", "You’re going a little fast. Wait a moment and try again."),
    (500, "Something went wrong", "An unexpected error happened on our side. Please try again in a moment."),
    (502, "Bad gateway", "The server got a bad answer from upstream. Try again shortly."),
    (503, "Temporarily unavailable", "Hevel is down for a moment, probably for maintenance. Please check back soon."),
    (504, "Gateway timeout", "The server didn’t answer in time. Try again shortly."),
];

fn describe(code: u16) -> (&'static str, &'static str) {
    ERROR_CODES
        .iter()
        .find(|(c, ..)| *c == code)
        .map(|&(_, title, message)| (title, message))
        .unwrap_or(("Something went wrong", "An unexpected error happened. Please try again."))
}

/// Same page, any status code.
#[component]
pub fn ErrorPage(code: u16) -> Element {
    let (title, message) = describe(code);
    rsx! {
        document::Title { "{code} – {title} · Hevel" }
        document::Meta { name: "robots", content: "noindex" }
        main { class: "page error-page",
            p { class: "error-code rise", "{code}" }
            h1 { class: "error-title rise", style: "--d: .1s", "{title}" }
            p { class: "muted rise", style: "--d: .2s", "{message}" }
            div { class: "rise", style: "--d: .3s",
                Link { to: Route::Home {}, class: "btn btn-md", "Back to Hevel" }
            }
        }
    }
}

/// `/error/<code>`
#[component]
pub fn ErrorRoute(code: u16) -> Element {
    rsx! { ErrorPage { code } }
}

/// Any address that doesn’t match a route. Also answers with a real 404 status when rendered on a server.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let _ = segments;
    use_hook(|| dioxus::fullstack::FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None));
    rsx! { ErrorPage { code: 404u16 } }
}
