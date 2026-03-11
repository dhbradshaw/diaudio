use crate::Route;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

const WAVEFORM_WIDTH: f64 = 600.0;
const WAVEFORM_HEIGHT: f64 = 180.0;
const SPECTRUM_WIDTH: f64 = 600.0;
const SPECTRUM_HEIGHT: f64 = 180.0;
const WAVEFORM_BINS: usize = 160;
const SPECTRUM_BINS: usize = 96;

#[cfg(target_arch = "wasm32")]
mod web_audio {
    use super::{SPECTRUM_BINS, WAVEFORM_BINS};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AnalyserNode, AudioContext, MediaStream, MediaStreamAudioSourceNode};

    pub struct AudioRuntime {
        pub context: AudioContext,
        pub analyser: AnalyserNode,
        pub _stream: MediaStream,
        pub _source: MediaStreamAudioSourceNode,
    }

    pub async fn initialize_audio_runtime() -> Result<AudioRuntime, String> {
        let window = web_sys::window().ok_or_else(|| "Window is unavailable".to_string())?;
        let navigator = window.navigator();
        let media_devices = navigator
            .media_devices()
            .map_err(|err| js_error("Could not access MediaDevices", &err))?;

        let constraints = web_sys::MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::TRUE);

        let media_stream_promise = media_devices
            .get_user_media_with_constraints(&constraints)
            .map_err(|err| js_error("Could not request microphone", &err))?;

        let media_stream_js = JsFuture::from(media_stream_promise)
            .await
            .map_err(|err| js_error("Microphone permission denied", &err))?;
        let media_stream: MediaStream = media_stream_js
            .dyn_into()
            .map_err(|_| "Could not convert microphone stream".to_string())?;

        let context =
            AudioContext::new().map_err(|err| js_error("Could not create AudioContext", &err))?;
        let source = context
            .create_media_stream_source(&media_stream)
            .map_err(|err| js_error("Could not create media source", &err))?;
        let analyser = context
            .create_analyser()
            .map_err(|err| js_error("Could not create analyser", &err))?;

        analyser.set_fft_size(2048);
        analyser.set_smoothing_time_constant(0.8);

        source
            .connect_with_audio_node(&analyser)
            .map_err(|err| js_error("Could not connect audio graph", &err))?;

        Ok(AudioRuntime {
            context,
            analyser,
            _stream: media_stream,
            _source: source,
        })
    }

    pub fn sample_frame(runtime: &AudioRuntime) -> (Vec<f32>, Vec<f32>, f32) {
        let mut waveform = vec![0u8; runtime.analyser.fft_size() as usize];
        runtime.analyser.get_byte_time_domain_data(&mut waveform);

        let mut spectrum = vec![0u8; runtime.analyser.frequency_bin_count() as usize];
        runtime.analyser.get_byte_frequency_data(&mut spectrum);

        let sampled_waveform = resample_u8_to_f32(&waveform, WAVEFORM_BINS, true);
        let sampled_spectrum = resample_u8_to_f32(&spectrum, SPECTRUM_BINS, false);

        let rms = sampled_waveform
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>()
            / sampled_waveform.len().max(1) as f32;
        let level_db = 20.0 * rms.sqrt().max(1e-6).log10();

        (sampled_waveform, sampled_spectrum, level_db)
    }

    pub fn close_runtime(runtime: &AudioRuntime) {
        let _ = runtime.context.close();
    }

    pub fn runtime_info(runtime: &AudioRuntime) -> (f32, usize, usize) {
        (
            runtime.context.sample_rate(),
            runtime.analyser.fft_size() as usize,
            runtime.analyser.frequency_bin_count() as usize,
        )
    }

    fn resample_u8_to_f32(input: &[u8], output_len: usize, centered: bool) -> Vec<f32> {
        if input.is_empty() || output_len == 0 {
            return Vec::new();
        }

        if output_len == 1 {
            return vec![if centered {
                (input[0] as f32 - 128.0) / 128.0
            } else {
                input[0] as f32 / 255.0
            }];
        }

        let input_max_index = (input.len() - 1) as f32;
        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let source_t = i as f32 / (output_len - 1) as f32;
            let source_index = (source_t * input_max_index).round() as usize;
            let byte = input[source_index] as f32;
            let value = if centered {
                (byte - 128.0) / 128.0
            } else {
                byte / 255.0
            };
            output.push(value);
        }

        output
    }

    fn js_error(prefix: &str, err: &JsValue) -> String {
        if let Some(message) = err.as_string() {
            format!("{prefix}: {message}")
        } else {
            format!("{prefix}: unknown JavaScript error")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod web_audio {
    pub struct AudioRuntime;

    pub async fn initialize_audio_runtime() -> Result<AudioRuntime, String> {
        Err("Real-time audio is only available in wasm32/browser builds".to_string())
    }

    pub fn sample_frame(_: &AudioRuntime) -> (Vec<f32>, Vec<f32>, f32) {
        (Vec::new(), Vec::new(), -120.0)
    }

    pub fn close_runtime(_: &AudioRuntime) {}

    pub fn runtime_info(_: &AudioRuntime) -> (f32, usize, usize) {
        (48_000.0, 2048, 1024)
    }
}

fn format_hz_label(hz: f32) -> String {
    if hz >= 1000.0 {
        format!("{:.1} kHz", hz / 1000.0)
    } else {
        format!("{hz:.0} Hz")
    }
}

fn line_points(values: &[f32], width: f64, height: f64, min: f32, max: f32) -> String {
    if values.is_empty() {
        return String::new();
    }

    let x_step = if values.len() > 1 {
        width / (values.len() - 1) as f64
    } else {
        0.0
    };
    let range = (max - min).max(f32::EPSILON);

    values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let x = i as f64 * x_step;
            let normalized = ((*value - min) / range).clamp(0.0, 1.0);
            let y = height - normalized as f64 * height;
            format!("{x},{y}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn bars(values: &[f32], width: f64, height: f64) -> Vec<(f64, f64, f64, f64)> {
    if values.is_empty() {
        return Vec::new();
    }

    let bar_width = (width / values.len() as f64).max(1.0);

    values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let normalized = value.clamp(0.0, 1.0) as f64;
            let h = normalized * height;
            let x = i as f64 * bar_width;
            let y = height - h;
            (x, y, (bar_width - 1.0).max(0.0), h)
        })
        .collect()
}

#[component]
pub fn RealTime() -> Element {
    let mut status = use_signal(|| "Click Start to request microphone access.".to_string());
    let mut runtime = use_signal(|| None::<web_audio::AudioRuntime>);
    let mut is_running = use_signal(|| false);
    let mut waveform = use_signal(|| vec![0.0f32; WAVEFORM_BINS]);
    let mut spectrum = use_signal(|| vec![0.0f32; SPECTRUM_BINS]);
    let mut level_db = use_signal(|| -120.0f32);
    let mut sample_rate_hz = use_signal(|| 48_000.0f32);
    let mut analyser_fft_size = use_signal(|| 2048usize);
    let mut analyser_freq_bins = use_signal(|| 1024usize);

    let _poller = use_future(move || async move {
        loop {
            if *is_running.read() {
                let frame = {
                    let runtime_ref = runtime.read();
                    runtime_ref.as_ref().map(web_audio::sample_frame)
                };

                if let Some((next_waveform, next_spectrum, next_level_db)) = frame {
                    waveform.set(next_waveform);
                    spectrum.set(next_spectrum);
                    level_db.set(next_level_db);
                }

                TimeoutFuture::new(33).await;
            } else {
                TimeoutFuture::new(125).await;
            }
        }
    });

    let waveform_points = line_points(&waveform.read(), WAVEFORM_WIDTH, WAVEFORM_HEIGHT, -1.0, 1.0);
    let spectrum_values = spectrum.read().clone();
    let spectrum_bars = bars(&spectrum_values, SPECTRUM_WIDTH, SPECTRUM_HEIGHT);
    let current_level_db = *level_db.read();
    let mic_active = *is_running.read();
    let has_runtime = runtime.read().is_some();
    let current_sample_rate_hz = *sample_rate_hz.read();
    let current_fft_size = *analyser_fft_size.read();
    let current_freq_bins = *analyser_freq_bins.read();
    let nyquist_hz = (current_sample_rate_hz / 2.0).max(1.0);
    let waveform_total_ms = ((current_fft_size as f32 / current_sample_rate_hz.max(1.0)) * 1000.0).max(0.1);

    let waveform_x_ticks = [0.0_f64, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|ratio| {
            (
                ratio * WAVEFORM_WIDTH,
                waveform_total_ms * (*ratio as f32),
            )
        })
        .collect::<Vec<_>>();

    let spectrum_x_ticks = [0.0_f64, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|ratio| {
            (ratio * SPECTRUM_WIDTH, nyquist_hz * (*ratio as f32))
        })
        .collect::<Vec<_>>();

    let waveform_y_ticks = [(-1.0_f32, "-1.0"), (-0.5_f32, "-0.5"), (0.0_f32, "0.0"), (0.5_f32, "0.5"), (1.0_f32, "1.0")]
        .iter()
        .map(|(value, label)| {
            let normalized = ((*value + 1.0) / 2.0).clamp(0.0, 1.0) as f64;
            let y = WAVEFORM_HEIGHT - normalized * WAVEFORM_HEIGHT;
            (y, *label)
        })
        .collect::<Vec<_>>();

    let spectrum_y_ticks = [(0.0_f32, "0.0"), (0.25_f32, "0.25"), (0.5_f32, "0.5"), (0.75_f32, "0.75"), (1.0_f32, "1.0")]
        .iter()
        .map(|(value, label)| {
            let normalized = value.clamp(0.0, 1.0) as f64;
            let y = SPECTRUM_HEIGHT - normalized * SPECTRUM_HEIGHT;
            (y, *label)
        })
        .collect::<Vec<_>>();

    rsx! {
        h1 { "Real Time Sound Visualizer" }
        p { "Microphone data is processed in your browser via Web Audio." }

        div { style: "display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.75rem;",
            button {
                r#type: "button",
                disabled: mic_active,
                onclick: move |_| {
                    if has_runtime {
                        is_running.set(true);
                        status.set("Microphone stream is running.".to_string());
                        return;
                    }

                    status.set("Requesting microphone permission...".to_string());
                    spawn(async move {
                        match web_audio::initialize_audio_runtime().await {
                            Ok(next_runtime) => {
                                let (next_sample_rate_hz, next_fft_size, next_freq_bins) =
                                    web_audio::runtime_info(&next_runtime);
                                    &next_runtime,
                                );
                                runtime.set(Some(next_runtime));
                                sample_rate_hz.set(next_sample_rate_hz);
                                analyser_fft_size.set(next_fft_size);
                                analyser_freq_bins.set(next_freq_bins);
                                is_running.set(true);
                                status.set("Microphone stream is running.".to_string());
                            }
                            Err(err) => {
                                is_running.set(false);
                                status.set(err);
                            }
                        }
                    });
                },
                "Start"
            }

            button {
                r#type: "button",
                disabled: !mic_active,
                onclick: move |_| {
                    is_running.set(false);
                    status.set("Microphone polling paused.".to_string());
                },
                "Pause"
            }

            button {
                r#type: "button",
                disabled: !has_runtime,
                onclick: move |_| {
                    is_running.set(false);
                    let previous = {
                        let mut write = runtime.write();
                        write.take()
                    };
                    if let Some(existing_runtime) = previous.as_ref() {
                        web_audio::close_runtime(existing_runtime);
                    }
                    waveform.set(vec![0.0; WAVEFORM_BINS]);
                    spectrum.set(vec![0.0; SPECTRUM_BINS]);
                    level_db.set(-120.0);
                    status.set("Microphone released.".to_string());
                },
                "Release Mic"
            }
        }

        p { "Status: {status}" }
        p { "Estimated level: {current_level_db:.1} dBFS" }
        p {
            "Sample rate: {current_sample_rate_hz:.0} Hz | FFT size: {current_fft_size} | Frequency bins: {current_freq_bins}"
        }

        div { style: "display: grid; gap: 1rem; grid-template-columns: 1fr; max-width: 760px;",
            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 0.75rem;",
                h2 { "Waveform" }
                svg {
                    view_box: "0 0 600 180",
                    width: "100%",
                    height: "180",
                    style: "display: block; border: 1px solid currentColor; border-radius: 4px;",
                    for (tick_y , tick_label) in waveform_y_ticks.iter() {
                        line {
                            x1: "0",
                            y1: "{tick_y}",
                            x2: "{WAVEFORM_WIDTH}",
                            y2: "{tick_y}",
                            stroke: "currentColor",
                            stroke_width: "1",
                            stroke_opacity: "0.12",
                        }
                        text {
                            x: "4",
                            y: "{tick_y - 2.0}",
                            text_anchor: "start",
                            font_size: "10",
                            fill: "currentColor",
                            fill_opacity: "0.85",
                            "{tick_label}"
                        }
                    }
                    for (tick_x , tick_ms) in waveform_x_ticks.iter() {
                        line {
                            x1: "{tick_x}",
                            y1: "0",
                            x2: "{tick_x}",
                            y2: "{WAVEFORM_HEIGHT}",
                            stroke: "currentColor",
                            stroke_width: "1",
                            stroke_opacity: "0.12",
                        }
                        text {
                            x: "{tick_x}",
                            y: "{WAVEFORM_HEIGHT - 6.0}",
                            text_anchor: "middle",
                            font_size: "10",
                            fill: "currentColor",
                            fill_opacity: "0.85",
                            "{tick_ms:.1} ms"
                        }
                    }
                    line {
                        x1: "0",
                        y1: "90",
                        x2: "600",
                        y2: "90",
                        stroke: "currentColor",
                        stroke_width: "1",
                        stroke_opacity: "0.35",
                    }
                    polyline {
                        points: "{waveform_points}",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                    }
                }
                p { "Time axis: 0 ms to {waveform_total_ms:.1} ms" }
                p { "Amplitude axis: -1.0 to 1.0" }
            }

            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 0.75rem;",
                h2 { "Spectrum" }
                svg {
                    view_box: "0 0 600 180",
                    width: "100%",
                    height: "180",
                    style: "display: block; border: 1px solid currentColor; border-radius: 4px;",
                    for (tick_y , tick_label) in spectrum_y_ticks.iter() {
                        line {
                            x1: "0",
                            y1: "{tick_y}",
                            x2: "{SPECTRUM_WIDTH}",
                            y2: "{tick_y}",
                            stroke: "currentColor",
                            stroke_width: "1",
                            stroke_opacity: "0.12",
                        }
                        text {
                            x: "4",
                            y: "{tick_y - 2.0}",
                            text_anchor: "start",
                            font_size: "10",
                            fill: "currentColor",
                            fill_opacity: "0.85",
                            "{tick_label}"
                        }
                    }
                    for (tick_x , tick_hz) in spectrum_x_ticks.iter() {
                        line {
                            x1: "{tick_x}",
                            y1: "0",
                            x2: "{tick_x}",
                            y2: "{SPECTRUM_HEIGHT}",
                            stroke: "currentColor",
                            stroke_width: "1",
                            stroke_opacity: "0.12",
                        }
                        text {
                            x: "{tick_x}",
                            y: "{SPECTRUM_HEIGHT - 6.0}",
                            text_anchor: "middle",
                            font_size: "10",
                            fill: "currentColor",
                            fill_opacity: "0.85",
                            "{format_hz_label(*tick_hz)}"
                        }
                    }
                    for (x , y , w , h) in spectrum_bars {
                        rect {
                            x: "{x}",
                            y: "{y}",
                            width: "{w}",
                            height: "{h}",
                            fill: "currentColor",
                            fill_opacity: "0.6",
                        }
                    }
                }
                p { "Frequency axis: 0 Hz to {format_hz_label(nyquist_hz)} (Nyquist)" }
                p { "Magnitude axis: 0.0 to 1.0" }
            }
        }

        Link { to: Route::Home {}, "Back to Home" }
    }
}
