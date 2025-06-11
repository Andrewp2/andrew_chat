use dioxus::prelude::*;
use web_sys::window;

mod speech;
mod views;
use crate::views::Theme;
use views::{Chat, ChatShare, Login, NotFound, Settings};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Chat {},
    #[route("/chat/:id")]
    ChatShare { id: usize },
    #[route("/settings")]
    Settings {},
    #[route("/login")]
    Login {},
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let theme = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|ls| ls.get_item("theme").ok().flatten())
                .map(|t| match t.as_str() {
                    "dark" => Theme::Dark,
                    "light" => Theme::Light,
                    _ => Theme::System,
                })
                .unwrap_or(Theme::System)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Theme::System
        }
    });

    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(win) = window() {
                if let Some(ls) = win.local_storage().ok().flatten() {
                    let s = match theme() {
                        Theme::Dark => "dark",
                        Theme::Light => "light",
                        Theme::System => "system",
                    };
                    let _ = ls.set_item("theme", s);
                    if let Some(doc) = win.document() {
                        let html = doc.document_element().unwrap();
                        if matches!(theme(), Theme::Dark) {
                            let _ = html.class_list().add_1("dark");
                        } else {
                            let _ = html.class_list().remove_1("dark");
                        }
                    }
                }
            }
        }
    });

    use_context_provider(|| theme);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Proxima+Vara:wght@100..900&display=swap" }
        document::Stylesheet {
            href: asset!("/assets/tailwind.css"),
        }
        document::Stylesheet { href: "https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css" }
        document::Stylesheet { href: "https://cdn.jsdelivr.net/npm/highlight.js@11.8.0/styles/github-dark.min.css" }
        document::Script { src: "https://cdn.jsdelivr.net/npm/highlight.js@11.8.0/lib/common.min.js" }

        Router::<Route> {}
    }
}
