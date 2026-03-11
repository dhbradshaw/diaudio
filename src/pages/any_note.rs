use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NoteName {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl NoteName {
    fn label(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::A => "A",
            Self::B => "B",
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::A => "A",
            Self::B => "B",
        }
    }

    fn from_value(value: &str) -> Option<Self> {
        match value {
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            "E" => Some(Self::E),
            "F" => Some(Self::F),
            "G" => Some(Self::G),
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            _ => None,
        }
    }

    fn semitones_from_c(self) -> i32 {
        match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Accidental {
    Flat,
    Natural,
    Sharp,
}

impl Accidental {
    fn label(self) -> &'static str {
        match self {
            Self::Flat => "♭ (Flat)",
            Self::Natural => "♮ (Natural)",
            Self::Sharp => "♯ (Sharp)",
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Natural => "natural",
            Self::Sharp => "sharp",
        }
    }

    fn from_value(value: &str) -> Option<Self> {
        match value {
            "flat" => Some(Self::Flat),
            "natural" => Some(Self::Natural),
            "sharp" => Some(Self::Sharp),
            _ => None,
        }
    }

    fn semitone_offset(self) -> i32 {
        match self {
            Self::Flat => -1,
            Self::Natural => 0,
            Self::Sharp => 1,
        }
    }
}

fn note_to_midi(note: NoteName, accidental: Accidental, octave: u8) -> u8 {
    let semitones_from_c = note.semitones_from_c() + accidental.semitone_offset();
    let octave_offset = if semitones_from_c < 0 {
        (octave as i32) - 1
    } else if semitones_from_c >= 12 {
        (octave as i32) + 1
    } else {
        octave as i32
    };
    let semitone_offset = semitones_from_c.rem_euclid(12);

    ((octave_offset + 1) * 12 + semitone_offset) as u8
}

fn midi_to_frequency(midi: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

fn get_note_display(note: NoteName, accidental: Accidental, octave: u8) -> String {
    let midi = note_to_midi(note, accidental, octave);
    let frequency = midi_to_frequency(midi);
    format!(
        "{}{}{} ({:.2} Hz)",
        note.label(),
        accidental.label(),
        octave,
        frequency
    )
}

#[component]
pub fn AnyNote() -> Element {
    let mut note = use_signal(|| NoteName::A);
    let mut accidental = use_signal(|| Accidental::Natural);
    let mut octave = use_signal(|| 4u8);
    let mut is_playing = use_signal(|| false);
    let mut status = use_signal(|| "Ready".to_string());

    let display_note = get_note_display(*note.read(), *accidental.read(), *octave.read());

    let handle_play_start = move |_| {
        let current_note = *note.read();
        let current_accidental = *accidental.read();
        let current_octave = *octave.read();
        let midi = note_to_midi(current_note, current_accidental, current_octave);
        let frequency = midi_to_frequency(midi);

        spawn({
            async move {
                if let Err(e) = start_synth(frequency).await {
                    status.set(format!("Error: {}", e));
                } else {
                    is_playing.set(true);
                    status.set(format!("Playing {} Hz", frequency.round()));
                }
            }
        });
    };

    let handle_play_end = move |_| {
        spawn({
            async move {
                let _ = stop_synth().await;
                is_playing.set(false);
                status.set("Ready".to_string());
            }
        });
    };

    rsx! {
        h1 { "Any Note" }
        p { "Select a note and press/hold play to hear it." }

        div { style: "display: grid; gap: 1rem; max-width: 500px;",
            section { style: "border: 1px solid currentColor; border-radius: 8px; padding: 1rem;",
                h2 { "Note Selection" }

                div { style: "display: grid; gap: 1rem;",
                    div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 0.5rem;",
                        div {
                            label { r#for: "note-select", "Note" }
                            select {
                                id: "note-select",
                                value: note.read().value(),
                                onchange: move |event| {
                                    if let Some(new_note) = NoteName::from_value(&event.value()) {
                                        note.set(new_note);
                                    }
                                },
                                option { value: "C", label: "C" }
                                option { value: "D", label: "D" }
                                option { value: "E", label: "E" }
                                option { value: "F", label: "F" }
                                option { value: "G", label: "G" }
                                option { value: "A", label: "A", selected: true }
                                option { value: "B", label: "B" }
                            }
                        }

                        div {
                            label { r#for: "accidental-select", "Accidental" }
                            select {
                                id: "accidental-select",
                                value: accidental.read().value(),
                                onchange: move |event| {
                                    if let Some(new_accidental) = Accidental::from_value(&event.value()) {
                                        accidental.set(new_accidental);
                                    }
                                },
                                option { value: "flat", label: "♭ Flat" }
                                option {
                                    value: "natural",
                                    label: "♮ Natural",
                                    selected: true,
                                }
                                option { value: "sharp", label: "♯ Sharp" }
                            }
                        }

                        div {
                            label { r#for: "octave-select", "Octave" }
                            select {
                                id: "octave-select",
                                value: "{octave.read()}",
                                onchange: move |event| {
                                    if let Ok(new_octave) = event.value().parse::<u8>() {
                                        if (0..=8).contains(&new_octave) {
                                            octave.set(new_octave);
                                        }
                                    }
                                },
                                {
                                    (0..=8)
                                        .map(|oct| {
                                            rsx! {
                                                option { value: "{oct}", label: "{oct}", selected: oct == 4 }
                                            }
                                        })
                                }
                            }
                        }
                    }

                    div { style: "padding: 1rem; background-color: rgba(100, 100, 100, 0.1); border-radius: 4px;",
                        p { style: "font-weight: bold; font-size: 1.2em;", "{display_note}" }
                    }

                    div { style: "display: grid; gap: 0.5rem;",
                        button {
                            r#type: "button",
                            onmousedown: handle_play_start,
                            onmouseup: handle_play_end,
                            onmouseleave: handle_play_end,
                            style: "padding: 1rem; font-size: 1.1em; font-weight: bold; cursor: pointer;",
                            "Press and Hold to Play"
                        }
                    }

                    p { style: "text-align: center; color: grey;", "Status: {status}" }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn start_synth(frequency: f32) -> Result<(), String> {
    use web_sys::{AudioContext, OscillatorType};

    // Stop any existing oscillator first
    let _ = stop_synth().await;

    let context =
        AudioContext::new().map_err(|err| format!("Could not create AudioContext: {:?}", err))?;

    let gain_node = context
        .create_gain()
        .map_err(|err| format!("Could not create gain node: {:?}", err))?;

    gain_node
        .connect_with_audio_node(&context.destination())
        .map_err(|err| format!("Could not connect gain node: {:?}", err))?;

    let oscillator = context
        .create_oscillator()
        .map_err(|err| format!("Could not create oscillator: {:?}", err))?;

    oscillator.frequency().set_value(frequency);
    oscillator.set_type(OscillatorType::Sine);

    oscillator
        .connect_with_audio_node(&gain_node)
        .map_err(|err| format!("Could not connect oscillator: {:?}", err))?;

    gain_node.gain().set_value(0.3);

    oscillator
        .start()
        .map_err(|_| "Could not start oscillator".to_string())?;

    // Store oscillator for cleanup
    let _ = AUDIO_STORE.with(|s| {
        let mut store = s.borrow_mut();
        *store = Some((oscillator, context, gain_node));
    });

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn start_synth(_frequency: f32) -> Result<(), String> {
    Err("Audio not available outside WASM".to_string())
}

#[cfg(target_arch = "wasm32")]
async fn stop_synth() -> Result<(), String> {
    let result = AUDIO_STORE.with(|s| {
        let mut store = s.borrow_mut();
        if let Some((oscillator, _context, _gain)) = store.take() {
            let _ = oscillator.stop();
        }
        Ok::<(), String>(())
    });
    result
}

#[cfg(not(target_arch = "wasm32"))]
async fn stop_synth() -> Result<(), String> {
    Err("Audio not available outside WASM".to_string())
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static AUDIO_STORE: std::cell::RefCell<Option<(web_sys::OscillatorNode, web_sys::AudioContext, web_sys::GainNode)>> = std::cell::RefCell::new(None);
}
