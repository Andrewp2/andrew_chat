use dioxus::prelude::*;

mod views;
use views::Chat;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Chat {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND: &str = "https://cdn.tailwindcss.com";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        script { src: TAILWIND }

        Router::<Route> {}
    }
}

