use dioxus::prelude::*;
const HEADER_SVG: Asset = asset!("/assets/header.svg");

#[component]
pub fn Hero() -> Element {
    rsx! {
        div { class: "flex flex-col justify-center items-center",
            img { src: HEADER_SVG, class: "max-w-lg" }
            div { class: "w-96 text-left text-white flex flex-col",
                a {
                    href: "https://dioxuslabs.com/learn/0.6/",
                    class: "my-2 border border-white rounded p-2 hover:bg-gray-800",
                    "📚 Learn Dioxus"
                }
                a {
                    href: "https://dioxuslabs.com/awesome",
                    class: "my-2 border border-white rounded p-2 hover:bg-gray-800",
                    "🚀 Awesome Dioxus"
                }
                a {
                    href: "https://github.com/dioxus-community/",
                    class: "my-2 border border-white rounded p-2 hover:bg-gray-800",
                    "📡 Community Libraries"
                }
                a {
                    href: "https://github.com/DioxusLabs/sdk",
                    class: "my-2 border border-white rounded p-2 hover:bg-gray-800",
                    "⚙️ Dioxus Development Kit"
                }
                a {
                    href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus",
                    class: "my-2 border border-white rounded p-2 hover:bg-gray-800",
                    "💫 VSCode Extension"
                }
                a {
                    href: "https://discord.gg/XgGxMSkvUM",
                    class: "my-2 border border-white rounded p-2 hover:bg-gray-800",
                    "👋 Community Discord"
                }
            }
        }
    }
}
