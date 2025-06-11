use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[allow(non_snake_case)]
pub fn DarkModeToggle() -> Element {
    let mut theme = use_context::<Signal<Theme>>();
    let checked = matches!(theme(), Theme::Dark);

    rsx! {
        label {
            class: "cursor-pointer select-none flex items-center gap-2",
            input {
                r#type: "checkbox",
                checked: "{checked}",
                onchange: move |e| {
                    let new = if e.value() == "true" { Theme::Dark } else { Theme::Light };
                    theme.set(new);
                }
            }
            span { "Dark-Mode" }
        }
    }
}
