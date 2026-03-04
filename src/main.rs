use dioxus::prelude::*;
mod pages;

use pages::fft_draw::FftDraw;

fn main() {
    launch(App);
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/fft-draw")]
    FftDraw {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Diaudio" }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        h1 { "Diaudio" }
        nav {
            ul {
                li {
                    Link { to: Route::FftDraw {}, "FFT Draw" }
                }
            }
        }
    }
}
