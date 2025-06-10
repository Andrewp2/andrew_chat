use dioxus::prelude::*;

mod views;
use views::{Chat, ChatShare, Settings, Login};

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
}

const FAVICON: Asset = asset!("/assets/favicon.ico");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Proxima+Vara:wght@100..900&display=swap" }
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css"),
        }
        document::Stylesheet { href: "https://cdn.jsdelivr.net/npm/highlight.js@11.8.0/styles/github-dark.min.css" }
        document::Script { src: "https://cdn.jsdelivr.net/npm/highlight.js@11.8.0/lib/common.min.js" }

        Router::<Route> {}
    }
}
