use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Diaudio" }
        h1 { "Diaudio" }
    }
}
