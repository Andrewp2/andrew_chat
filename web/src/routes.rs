use crate::views::Chat;
use crate::views::ChatShare;
use crate::views::Login;
use crate::views::NotFound;
use crate::views::Settings;
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
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
