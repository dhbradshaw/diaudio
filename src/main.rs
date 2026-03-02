use dioxus::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};

const WAVEFORM_WIDTH: f64 = 600.0;
const WAVEFORM_HEIGHT: f64 = 240.0;
const DEFAULT_SAMPLE_SIZE: usize = 256;
const FFT_CHART_WIDTH: f64 = 600.0;
const FFT_CHART_HEIGHT: f64 = 240.0;

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

fn normalize_waveform(points: &[(f64, f64)], sample_size: usize) -> Vec<f64> {
    if sample_size == 0 {
        return Vec::new();
    }

    if points.is_empty() {
        return vec![0.0; sample_size];
    }

    let mut normalized = Vec::with_capacity(sample_size);
    let mut segment_index = 0usize;

    for i in 0..sample_size {
        let t = if sample_size == 1 {
            0.0
        } else {
            i as f64 / (sample_size - 1) as f64
        };
        let target_x = t * (WAVEFORM_WIDTH - 1.0);

        while segment_index + 1 < points.len() && points[segment_index + 1].0 < target_x {
            segment_index += 1;
        }

        let sampled_y = if segment_index + 1 < points.len() {
            let (x0, y0) = points[segment_index];
            let (x1, y1) = points[segment_index + 1];

            let dx = x1 - x0;
            if dx.abs() < f64::EPSILON {
                y0
            } else {
                let alpha = ((target_x - x0) / dx).clamp(0.0, 1.0);
                y0 + alpha * (y1 - y0)
            }
        } else {
            points[segment_index].1
        };

        let amplitude = (1.0 - (2.0 * sampled_y / WAVEFORM_HEIGHT)).clamp(-1.0, 1.0);
        normalized.push(amplitude);
    }

    normalized
}

fn run_fft(samples: &[f64]) -> Vec<Complex<f64>> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut buffer = samples
        .iter()
        .map(|sample| Complex::new(*sample, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut buffer);
    buffer
}

fn fft_magnitudes(fft_bins: &[Complex<f64>]) -> Vec<f64> {
    if fft_bins.is_empty() {
        return Vec::new();
    }

    let half_len = fft_bins.len() / 2;
    let scale = 1.0 / fft_bins.len() as f64;

    fft_bins
        .iter()
        .take(half_len)
        .map(|bin| bin.norm() * scale)
        .collect()
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

    let waveform_snapshot = waveform_points.read().clone();

    let line_points = waveform_snapshot
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");

    let normalized_samples = normalize_waveform(&waveform_snapshot, DEFAULT_SAMPLE_SIZE);
    let normalized_sample_count = normalized_samples.len();
    let first_sample = normalized_samples.first().copied().unwrap_or(0.0);
    let fft_bins = run_fft(&normalized_samples);
    let magnitude_bins = fft_magnitudes(&fft_bins);
    let magnitude_bin_count = magnitude_bins.len();
    let first_magnitude = magnitude_bins.first().copied().unwrap_or(0.0);
    let max_magnitude = magnitude_bins.iter().copied().fold(0.0, f64::max);

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
                svg {
                    view_box: "0 0 600 240",
                    width: "100%",
                    height: "240",
                    style: "display: block; border: 1px solid currentColor; border-radius: 4px;",
                    for (index , magnitude) in magnitude_bins.iter().enumerate() {
                        {
                            let bar_width = FFT_CHART_WIDTH / magnitude_bin_count as f64;
                            let normalized = if max_magnitude > 0.0 {
                                *magnitude / max_magnitude
                            } else {
                                0.0
                            };
                            let bar_height = (normalized * FFT_CHART_HEIGHT).clamp(0.0, FFT_CHART_HEIGHT);
                            let x = index as f64 * bar_width;
                            let y = FFT_CHART_HEIGHT - bar_height;
                            rsx! {
                                rect {
                                    x: "{x}",
                                    y: "{y}",
                                    width: "{bar_width}",
                                    height: "{bar_height}",
                                    fill: "currentColor",
                                }
                            }
                        }
                    }
                }
                p { "Normalized samples: {normalized_sample_count}" }
                p { "First sample: {first_sample}" }
                p { "Magnitude bins: {magnitude_bin_count}" }
                p { "First magnitude: {first_magnitude}" }
                p { "Max magnitude: {max_magnitude}" }
            }
        }
        Link { to: Route::Home {}, "Back to Home" }
    }
}
