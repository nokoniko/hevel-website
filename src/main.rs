mod components;
mod hooks;
mod pages;
mod web;

use components::icons::{FaqIcon, GithubIcon};
use dioxus::prelude::*;
use hooks::{latest_release, REPO};
use pages::{
    error::{ErrorRoute, NotFound},
    faq::Faq,
    home::Home,
};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const FAVICON: Asset = asset!("/assets/favicon.svg");

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Shell)]
        #[route("/")]
        Home {},
        #[route("/faq")]
        Faq {},
        #[route("/error/:code")]
        ErrorRoute { code: u16 },
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}

fn main() {
    dioxus::LaunchBuilder::new()
        // Release builds only (`dx build --ssg`): the server renders each static route once and the
        // incremental renderer writes it to `<public>/<route>/index.html`. In `dx serve` it stays off,
        // otherwise stale pages get cached to disk and the file watcher keeps rebuilding.
        .with_cfg(server_only! {
            {
                let cfg = dioxus::server::ServeConfig::builder();
                if cfg!(debug_assertions) {
                    cfg
                } else {
                    cfg.incremental(
                        dioxus::server::IncrementalRendererConfig::new()
                            .static_dir(std::env::var("DIOXUS_PUBLIC_PATH").unwrap_or_else(|_| "public".into())),
                    )
                }
            }
        })
        .launch(App);
}

/// Lists the routes `dx build --ssg` should pre-render: every route without dynamic segments, plus an
/// error page per status code.
#[post("/api/static_routes")]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
    let mut routes: Vec<String> = Route::static_routes().iter().map(ToString::to_string).collect();
    routes.extend(pages::error::ERROR_CODES.iter().map(|(code, ..)| format!("/error/{code}")));
    Ok(routes)
}

#[component]
fn App() -> Element {
    // Fetched on the server, rendered into the HTML, and handed to the client during hydration.
    let fetched = use_server_future(latest_release)?;
    let initial = fetched.read().as_ref().and_then(|r| r.as_ref().ok()).cloned().unwrap_or_default();
    use_context_provider(|| Signal::new(initial));

    rsx! {
        document::Link { rel: "icon", r#type: "image/svg+xml", href: FAVICON }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        document::Meta { name: "theme-color", content: "#000000" }
        document::Meta {
            name: "description",
            content: "Hevel turns the MacBook notch into a Dynamic Island: now playing, charging and lock screen.",
        }
        document::Title { "Hevel – A Dynamic Island for your Mac" }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}

/// Header + footer around every page. Scrolls to top and wires up `.reveal` on navigation.
#[component]
fn Shell() -> Element {
    let route = use_route::<Route>();
    use_effect(use_reactive!(|route| {
        let _ = route;
        web::dom::page_effects();
    }));

    let year = web::dom::current_year();

    rsx! {
        header { class: "nav rise",
            Link { to: Route::Home {}, class: "logo", "Hevel" }
            nav { class: "nav-links",
                Link { to: Route::Faq {}, class: "muted nav-icon", "aria-label": "FAQ", title: "FAQ", FaqIcon {} }
                a {
                    href: "https://github.com/{REPO}",
                    class: "muted nav-icon",
                    "aria-label": "GitHub",
                    title: "GitHub",
                    target: "_blank",
                    rel: "noreferrer",
                    GithubIcon {}
                }
            }
        }

        Outlet::<Route> {}

        footer { class: "footer muted small",
            span { "© {year} Hevel" }
            span { "Nothing leaves your Mac. No accounts, no tracking." }
        }
    }
}
