use dioxus::prelude::*;
mod pages;

use pages::any_note::AnyNote;
use pages::fft_draw::FftDraw;
use pages::real_time::RealTime;

fn main() {
    launch(App);
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/any-note")]
    AnyNote {},
    #[route("/fft-draw")]
    FftDraw {},
    #[route("/real-time")]
    RealTime {},
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
                    Link { to: Route::AnyNote {}, "Any Note" }
                }
                li {
                    Link { to: Route::FftDraw {}, "FFT Draw" }
                }
                li {
                    Link { to: Route::RealTime {}, "Real Time" }
                }
            }
        }
    }
}
