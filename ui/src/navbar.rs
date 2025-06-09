use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-row space-x-5 text-white", {children} }
    }
}
