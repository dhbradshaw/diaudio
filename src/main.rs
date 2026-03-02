use dioxus::prelude::*;

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

#[component]
fn FftDraw() -> Element {
    rsx! {
        h1 { "FFT Draw" }
        div {
            style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; min-height: 320px;",
            section {
                style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "Waveform" }
                p { "Draw area placeholder" }
            }
            section {
                style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "FFT" }
                p { "Spectrum placeholder" }
            }
        }
        Link { to: Route::Home {}, "Back to Home" }
    }
}
