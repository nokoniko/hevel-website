use dioxus::prelude::*;

const ITEMS: &[(&str, &str)] = &[(
    "Do you collect any data?",
    "No. Hevel has no accounts, no analytics and no tracking. The only thing it ever contacts is GitHub, to check for new versions.",
)];

#[component]
pub fn Faq() -> Element {
    rsx! {
        main { class: "page",
            h1 { class: "page-title rise", "FAQ" }
            for (i , (q , a)) in ITEMS.iter().enumerate() {
                div { key: "{q}", class: "qa rise", style: "--d: {0.15 + i as f64 * 0.1}s",
                    h3 { "{q}" }
                    p { "{a}" }
                }
            }
        }
    }
}
