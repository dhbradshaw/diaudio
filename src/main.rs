use dioxus::prelude::*;

const WAVEFORM_WIDTH: f64 = 600.0;
const WAVEFORM_HEIGHT: f64 = 240.0;

fn upsert_waveform_point(points: &mut Vec<(f64, f64)>, x: f64, y: f64) {
    let x = x.round().clamp(0.0, WAVEFORM_WIDTH - 1.0);
    let y = y.clamp(0.0, WAVEFORM_HEIGHT);

    if let Some((_, existing_y)) = points
        .iter_mut()
        .find(|(existing_x, _)| (*existing_x - x).abs() < f64::EPSILON)
    {
        *existing_y = y;
    } else {
        points.push((x, y));
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
}

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
    let mut waveform_points = use_signal(Vec::<(f64, f64)>::new);
    let mut is_drawing = use_signal(|| false);

    let line_points = waveform_points
        .read()
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");

    rsx! {
        h1 { "FFT Draw" }
        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; min-height: 320px;",
            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "Waveform" }
                p { "Click and drag to draw." }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        is_drawing.set(false);
                        waveform_points.set(Vec::new());
                    },
                    "Clear"
                }
                svg {
                    view_box: "0 0 600 240",
                    width: "100%",
                    height: "240",
                    style: "display: block; border: 1px solid currentColor; border-radius: 4px; touch-action: none; cursor: crosshair;",
                    onmousedown: move |event| {
                        is_drawing.set(true);
                        let coordinates = event.element_coordinates();
                        let mut points = waveform_points.write();
                        upsert_waveform_point(&mut points, coordinates.x, coordinates.y);
                    },
                    onmousemove: move |event| {
                        if !*is_drawing.read() {
                            return;
                        }

                        let coordinates = event.element_coordinates();
                        let mut points = waveform_points.write();
                        upsert_waveform_point(&mut points, coordinates.x, coordinates.y);
                    },
                    onmouseup: move |_| {
                        is_drawing.set(false);
                    },
                    onmouseleave: move |_| {
                        is_drawing.set(false);
                    },
                    polyline {
                        points: "{line_points}",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                    }
                }
            }
            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "FFT" }
                p { "Spectrum placeholder" }
            }
        }
        Link { to: Route::Home {}, "Back to Home" }
    }
}
