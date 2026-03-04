use crate::Route;
use dioxus::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};

const WAVEFORM_WIDTH: f64 = 600.0;
const WAVEFORM_HEIGHT: f64 = 240.0;
const DEFAULT_SAMPLE_SIZE: usize = 256;
const FFT_CHART_WIDTH: f64 = 600.0;
const FFT_CHART_HEIGHT: f64 = 240.0;
const SAMPLE_SIZE_OPTIONS: [usize; 4] = [128, 256, 512, 1024];

#[derive(Clone, Copy, PartialEq, Eq)]
enum TestSignalKind {
    SinX,
    SinXPlusSin2X,
    SinXPlusSin16X,
}

impl TestSignalKind {
    fn label(self) -> &'static str {
        match self {
            Self::SinX => "sin(x)",
            Self::SinXPlusSin2X => "sin(x) + sin(2x)",
            Self::SinXPlusSin16X => "sin(x) + sin(16x)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PresetSelection {
    DrawMode,
    Clear,
    SinX,
    SinXPlusSin2X,
    SinXPlusSin16X,
}

impl PresetSelection {
    fn value(self) -> &'static str {
        match self {
            Self::DrawMode => "draw",
            Self::Clear => "clear",
            Self::SinX => "sin_x",
            Self::SinXPlusSin2X => "sin_x_plus_sin_2x",
            Self::SinXPlusSin16X => "sin_x_plus_sin_16x",
        }
    }

    fn from_value(value: &str) -> Option<Self> {
        match value {
            "draw" => Some(Self::DrawMode),
            "clear" => Some(Self::Clear),
            "sin_x" => Some(Self::SinX),
            "sin_x_plus_sin_2x" => Some(Self::SinXPlusSin2X),
            "sin_x_plus_sin_16x" => Some(Self::SinXPlusSin16X),
            _ => None,
        }
    }

    fn signal_kind(self) -> Option<TestSignalKind> {
        match self {
            Self::SinX => Some(TestSignalKind::SinX),
            Self::SinXPlusSin2X => Some(TestSignalKind::SinXPlusSin2X),
            Self::SinXPlusSin16X => Some(TestSignalKind::SinXPlusSin16X),
            Self::DrawMode | Self::Clear => None,
        }
    }
}

fn test_signal_samples(sample_count: usize, signal_kind: TestSignalKind) -> Vec<f64> {
    if sample_count == 0 {
        return Vec::new();
    }

    let denominator = (sample_count.saturating_sub(1)) as f64;
    (0..sample_count)
        .map(|index| {
            let t = if denominator > 0.0 {
                index as f64 / denominator
            } else {
                0.0
            };
            let x = std::f64::consts::TAU * t;
            match signal_kind {
                TestSignalKind::SinX => x.sin(),
                TestSignalKind::SinXPlusSin2X => {
                    ((x.sin() + (2.0 * x).sin()) * 0.5).clamp(-1.0, 1.0)
                }
                TestSignalKind::SinXPlusSin16X => {
                    ((x.sin() + (16.0 * x).sin()) * 0.5).clamp(-1.0, 1.0)
                }
            }
        })
        .collect()
}

fn test_signal_waveform_points(signal_kind: TestSignalKind) -> Vec<(f64, f64)> {
    let sample_count = WAVEFORM_WIDTH as usize;
    let samples = test_signal_samples(sample_count, signal_kind);
    samples_to_waveform_points(&samples)
}

fn clamp_waveform_coordinates(x: f64, y: f64) -> (f64, f64) {
    (
        x.round().clamp(0.0, WAVEFORM_WIDTH - 1.0),
        y.clamp(0.0, WAVEFORM_HEIGHT),
    )
}

fn upsert_waveform_point(points: &mut Vec<(f64, f64)>, x: f64, y: f64) {
    let (x, y) = clamp_waveform_coordinates(x, y);

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

fn draw_waveform_segment(points: &mut Vec<(f64, f64)>, start: (f64, f64), end: (f64, f64)) {
    let (x0, y0) = clamp_waveform_coordinates(start.0, start.1);
    let (x1, y1) = clamp_waveform_coordinates(end.0, end.1);

    if (x1 - x0).abs() < f64::EPSILON {
        upsert_waveform_point(points, x1, y1);
        return;
    }

    let min_x = x0.min(x1);
    let max_x = x0.max(x1);
    let start_x = min_x as i32;
    let end_x = max_x as i32;
    let dx = x1 - x0;

    points.retain(|(x, _)| *x < min_x || *x > max_x);

    for xi in start_x..=end_x {
        let x = xi as f64;
        let alpha = ((x - x0) / dx).clamp(0.0, 1.0);
        let y = y0 + alpha * (y1 - y0);
        points.push((x, y.clamp(0.0, WAVEFORM_HEIGHT)));
    }

    points.sort_by(|a, b| a.0.total_cmp(&b.0));
}

fn normalize_waveform(points: &[(f64, f64)], sample_size: usize) -> Vec<f64> {
    if sample_size == 0 {
        return Vec::new();
    }

    if points.is_empty() {
        return Vec::new();
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

fn run_ifft(fft_bins: &[Complex<f64>]) -> Vec<f64> {
    if fft_bins.is_empty() {
        return Vec::new();
    }

    let mut planner = FftPlanner::<f64>::new();
    let ifft = planner.plan_fft_inverse(fft_bins.len());
    let mut buffer = fft_bins.to_vec();

    ifft.process(&mut buffer);

    let scale = 1.0 / fft_bins.len() as f64;
    buffer
        .iter()
        .map(|sample| (sample.re * scale).clamp(-1.0, 1.0))
        .collect()
}

fn apply_lowpass(fft_bins: &[Complex<f64>], cutoff_percent: f64) -> Vec<Complex<f64>> {
    if fft_bins.is_empty() {
        return Vec::new();
    }

    let n = fft_bins.len();
    let half_len = n / 2;
    let clamped_percent = cutoff_percent.clamp(0.0, 100.0);
    let max_cutoff = half_len.saturating_sub(1);
    let cutoff_bin = ((max_cutoff as f64) * (clamped_percent / 100.0)).round() as usize;

    fft_bins
        .iter()
        .enumerate()
        .map(|(index, bin)| {
            let mirrored_index = if index <= n / 2 { index } else { n - index };
            if mirrored_index <= cutoff_bin {
                *bin
            } else {
                Complex::new(0.0, 0.0)
            }
        })
        .collect()
}

fn samples_to_waveform_points(samples: &[f64]) -> Vec<(f64, f64)> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::with_capacity(samples.len());
    let denominator = (samples.len().saturating_sub(1)) as f64;

    for (index, sample) in samples.iter().enumerate() {
        let normalized_x = if denominator > 0.0 {
            index as f64 / denominator
        } else {
            0.0
        };
        let x = normalized_x * (WAVEFORM_WIDTH - 1.0);
        let y =
            ((1.0 - sample.clamp(-1.0, 1.0)) * 0.5 * WAVEFORM_HEIGHT).clamp(0.0, WAVEFORM_HEIGHT);
        points.push((x, y));
    }

    points
}

fn fft_components(fft_bins: &[Complex<f64>]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    if fft_bins.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let half_len = fft_bins.len() / 2;
    let scale = 1.0 / fft_bins.len() as f64;

    let mut real_bins = Vec::with_capacity(half_len);
    let mut imag_bins = Vec::with_capacity(half_len);
    let mut amplitude_bins = Vec::with_capacity(half_len);

    for bin in fft_bins.iter().take(half_len) {
        real_bins.push(bin.re * scale);
        imag_bins.push(bin.im * scale);
        amplitude_bins.push(bin.norm() * scale);
    }

    (real_bins, imag_bins, amplitude_bins)
}

fn fft_line_points(values: &[f64], min_value: f64, max_value: f64) -> String {
    if values.is_empty() {
        return String::new();
    }

    let width_denominator = (values.len().saturating_sub(1)) as f64;
    let step_x = if width_denominator > 0.0 {
        FFT_CHART_WIDTH / width_denominator
    } else {
        0.0
    };

    let value_range = (max_value - min_value).abs().max(f64::EPSILON);

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = index as f64 * step_x;
            let normalized = ((*value - min_value) / value_range).clamp(0.0, 1.0);
            let y = FFT_CHART_HEIGHT - (normalized * FFT_CHART_HEIGHT);
            format!("{x},{y}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn FftDraw() -> Element {
    let mut waveform_points = use_signal(Vec::<(f64, f64)>::new);
    let mut is_drawing = use_signal(|| false);
    let mut active_test_signal = use_signal(|| None::<TestSignalKind>);
    let mut preset_selection = use_signal(|| PresetSelection::DrawMode);
    let mut last_draw_position = use_signal(|| None::<(f64, f64)>);
    let mut sample_size = use_signal(|| DEFAULT_SAMPLE_SIZE);
    let mut lowpass_cutoff_percent = use_signal(|| 100.0f64);

    let waveform_snapshot = waveform_points.read().clone();

    let waveform_line_points = waveform_snapshot
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");

    let current_sample_size = *sample_size.read();
    let current_lowpass_cutoff_percent = *lowpass_cutoff_percent.read();
    let has_waveform_input = !waveform_snapshot.is_empty();
    let normalized_samples = normalize_waveform(&waveform_snapshot, current_sample_size);
    let normalized_sample_count = normalized_samples.len();
    let (min_sample, max_sample) = if normalized_samples.is_empty() {
        (0.0, 0.0)
    } else {
        normalized_samples.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(min_value, max_value), sample| (min_value.min(*sample), max_value.max(*sample)),
        )
    };
    let sample_span = max_sample - min_sample;
    let is_flat_input = has_waveform_input && sample_span <= 1e-6;
    let first_sample = normalized_samples.first().copied().unwrap_or(0.0);
    let fft_bins = if has_waveform_input {
        run_fft(&normalized_samples)
    } else {
        Vec::new()
    };
    let filtered_fft_bins = apply_lowpass(&fft_bins, current_lowpass_cutoff_percent);
    let fft_bins_for_reconstruction = filtered_fft_bins.clone();
    let (real_bins, imag_bins, amplitude_bins) = fft_components(&filtered_fft_bins);
    let bin_count = amplitude_bins.len();
    let cutoff_bin = if bin_count > 0 {
        ((bin_count - 1) as f64 * (current_lowpass_cutoff_percent / 100.0)).round() as usize
    } else {
        0
    };
    let first_real = real_bins.first().copied().unwrap_or(0.0);
    let first_imag = imag_bins.first().copied().unwrap_or(0.0);
    let first_amplitude = amplitude_bins.first().copied().unwrap_or(0.0);
    let max_complex_component = real_bins
        .iter()
        .chain(imag_bins.iter())
        .map(|value| value.abs())
        .fold(0.0, f64::max)
        .max(1.0);
    let max_amplitude = amplitude_bins.iter().copied().fold(0.0, f64::max).max(1.0);

    let real_points = fft_line_points(&real_bins, -max_complex_component, max_complex_component);
    let imag_points = fft_line_points(&imag_bins, -max_complex_component, max_complex_component);
    let amplitude_points = fft_line_points(&amplitude_bins, 0.0, max_amplitude);
    let zero_line_y = FFT_CHART_HEIGHT / 2.0;
    let frequency_markers = [0.0, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|ratio| {
            let x = ratio * FFT_CHART_WIDTH;
            let bin = ((bin_count.saturating_sub(1)) as f64 * ratio).round() as usize;
            (x, bin, format!("{:.0}%", ratio * 100.0))
        })
        .collect::<Vec<_>>();

    rsx! {
        h1 { "FFT Draw" }
        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; min-height: 320px;",
            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "Waveform" }
                p { "Click and drag to draw." }
                div { style: "display: flex; gap: 0.5rem; align-items: center; margin-bottom: 0.5rem;",
                    label { "Preset signal" }
                    select {
                        value: "{preset_selection.read().value()}",
                        onchange: move |event| {
                            let Some(selection) = PresetSelection::from_value(&event.value()) else {
                                return;
                            };

                            is_drawing.set(false);
                            last_draw_position.set(None);

                            match selection {
                                PresetSelection::DrawMode => {
                                    active_test_signal.set(None);
                                    preset_selection.set(PresetSelection::DrawMode);
                                }
                                PresetSelection::Clear => {
                                    active_test_signal.set(None);
                                    waveform_points.set(Vec::new());
                                    preset_selection.set(PresetSelection::DrawMode);
                                }
                                PresetSelection::SinX
                                | PresetSelection::SinXPlusSin2X
                                | PresetSelection::SinXPlusSin16X => {
                                    if let Some(signal_kind) = selection.signal_kind() {
                                        active_test_signal.set(Some(signal_kind));
                                        waveform_points.set(test_signal_waveform_points(signal_kind));
                                        preset_selection.set(selection);
                                    }
                                }
                            }
                        },
                        option { value: "draw", "Draw mode" }
                        option { value: "clear", "Clear (empty waveform)" }
                        option { value: "sin_x", "sin(x)" }
                        option { value: "sin_x_plus_sin_2x", "sin(x) + sin(2x)" }
                        option { value: "sin_x_plus_sin_16x", "sin(x) + sin(16x)" }
                    }
                    button {
                        r#type: "button",
                        disabled: fft_bins_for_reconstruction.is_empty(),
                        onclick: move |_| {
                            is_drawing.set(false);
                            active_test_signal.set(None);
                            preset_selection.set(PresetSelection::DrawMode);
                            last_draw_position.set(None);
                            let reconstructed_samples = run_ifft(&fft_bins_for_reconstruction);
                            let reconstructed_points = samples_to_waveform_points(&reconstructed_samples);
                            waveform_points.set(reconstructed_points);
                        },
                        "Recreate from FFT"
                    }
                    label { "Sample size" }
                    select {
                        value: "{current_sample_size}",
                        onchange: move |event| {
                            if let Ok(next_size) = event.value().parse::<usize>() {
                                sample_size.set(next_size);
                            }
                        },
                        for option_size in SAMPLE_SIZE_OPTIONS {
                            option { value: "{option_size}", "{option_size}" }
                        }
                    }
                }
                div { style: "display: flex; gap: 0.5rem; align-items: center; margin-bottom: 0.5rem; flex-wrap: wrap;",
                    label { "Low-pass cutoff" }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "100",
                        step: "1",
                        value: "{current_lowpass_cutoff_percent}",
                        oninput: move |event| {
                            if let Ok(next_value) = event.value().parse::<f64>() {
                                lowpass_cutoff_percent.set(next_value.clamp(0.0, 100.0));
                            }
                        },
                    }
                    span { "{current_lowpass_cutoff_percent.round()}% (bin {cutoff_bin})" }
                }
                svg {
                    view_box: "0 0 600 240",
                    width: "600",
                    height: "240",
                    style: "display: block; border: 1px solid currentColor; border-radius: 4px; touch-action: none; cursor: crosshair;",
                    onmousedown: move |event| {
                        is_drawing.set(true);
                        active_test_signal.set(None);
                        preset_selection.set(PresetSelection::DrawMode);
                        let coordinates = event.element_coordinates();
                        let (x, y) = clamp_waveform_coordinates(coordinates.x, coordinates.y);
                        last_draw_position.set(Some((x, y)));
                        let mut points = waveform_points.write();
                        upsert_waveform_point(&mut points, x, y);
                    },
                    onmousemove: move |event| {
                        if !*is_drawing.read() {
                            return;
                        }

                        let coordinates = event.element_coordinates();
                        let current = clamp_waveform_coordinates(coordinates.x, coordinates.y);
                        let previous = *last_draw_position.read();
                        let mut points = waveform_points.write();
                        if let Some(previous) = previous {
                            draw_waveform_segment(&mut points, previous, current);
                        } else {
                            upsert_waveform_point(&mut points, current.0, current.1);
                        }
                        last_draw_position.set(Some(current));
                    },
                    onmouseup: move |_| {
                        is_drawing.set(false);
                        last_draw_position.set(None);
                    },
                    onmouseleave: move |_| {
                        is_drawing.set(false);
                        last_draw_position.set(None);
                    },
                    polyline {
                        points: "{waveform_line_points}",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                    }
                }
            }
            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "FFT" }
                if !has_waveform_input {
                    p { "Draw a waveform to generate FFT data." }
                } else if let Some(signal_kind) = *active_test_signal.read() {
                    p { "Using synthetic test input: {signal_kind.label()}." }
                } else if is_flat_input {
                    p { "Flat waveform detected: most energy is in the DC bin (near b0)." }
                }
                svg {
                    view_box: "0 0 600 240",
                    width: "100%",
                    height: "240",
                    style: "display: block; border: 1px solid currentColor; border-radius: 4px;",
                    for (marker_x , marker_bin , marker_label) in frequency_markers.iter() {
                        line {
                            x1: "{marker_x}",
                            y1: "0",
                            x2: "{marker_x}",
                            y2: "{FFT_CHART_HEIGHT}",
                            stroke: "currentColor",
                            stroke_width: "1",
                            stroke_opacity: "0.15",
                        }
                        text {
                            x: "{marker_x}",
                            y: "{FFT_CHART_HEIGHT - 6.0}",
                            text_anchor: "middle",
                            font_size: "10",
                            fill: "currentColor",
                            fill_opacity: "0.8",
                            "{marker_label} (b{marker_bin})"
                        }
                    }
                    line {
                        x1: "0",
                        y1: "{zero_line_y}",
                        x2: "{FFT_CHART_WIDTH}",
                        y2: "{zero_line_y}",
                        stroke: "currentColor",
                        stroke_width: "1",
                        stroke_opacity: "0.4",
                    }
                    polyline {
                        points: "{real_points}",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                    }
                    polyline {
                        points: "{imag_points}",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_dasharray: "8 4",
                    }
                    polyline {
                        points: "{amplitude_points}",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_dasharray: "2 3",
                    }
                }
                p { "Line styles: real (solid), imaginary (dashed), amplitude (dotted)" }
                p { "Frequency markers: 0% (DC) to 100% (Nyquist)" }
                p { "Normalized samples: {normalized_sample_count}" }
                p { "Selected sample size: {current_sample_size}" }
                p { "Low-pass cutoff: {current_lowpass_cutoff_percent.round()}% (bin {cutoff_bin})" }
                p { "First sample: {first_sample}" }
                p { "FFT bins: {bin_count}" }
                p { "First real: {first_real}" }
                p { "First imaginary: {first_imag}" }
                p { "First amplitude: {first_amplitude}" }
            }
        }
        Link { to: Route::Home {}, "Back to Home" }
    }
}
