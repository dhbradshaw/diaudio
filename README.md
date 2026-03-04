# Diaudio

Playing with sound and dioxus.

## Pages

### Home screen

Route: `/`

This is just a pointer to all the different tools, pages, and games.

### FFT Draw

Route: `/fft-draw`

Draw a wave form on the left and see its FFT on the right.
You can also click **Recreate from FFT** to reconstruct the waveform from the current FFT bins (inverse transform).
Use the low-pass cutoff slider to remove higher-frequency bins from the spectrum.

Implementation checklist:

- [x] Add page component and route registration for `/fft-draw`.
- [x] Build a two-panel layout (waveform draw area on left, FFT output on right).
- [x] Implement pointer/mouse drawing on the waveform canvas.
- [x] Normalize drawn points into a fixed-size sample buffer.
- [x] Run FFT on the sample buffer (real input to frequency bins).
- [x] Convert FFT output to magnitudes (linear or dB scale).
- [x] Render FFT bins as a chart/bars on the right panel.
- [x] Add controls for clear/reset and sample-size selection.
- [x] Add inverse reconstruction button to rebuild waveform from FFT.
- [x] Add low-pass cutoff control for spectrum filtering.
- [x] Add axis labels or frequency markers for readability.
- [x] Handle empty/flat input and other edge cases gracefully.
- [x] Verify behavior in wasm/web target and document any limitations.

WASM/Web verification and limitations:

- Verified compile success for web target with `cargo check --target wasm32-unknown-unknown`.
- Empty state is explicit: no waveform means no FFT bins, and reconstruction is disabled.
- Flat input is handled with an explanatory message (expected strong DC component).
- Interaction currently uses mouse events (`onmousedown` / `onmousemove` / `onmouseup`), so touch and pen behavior may vary by browser/device.
- Drawing does not capture pointer outside the waveform SVG; dragging outside ends drawing until pointer returns.

Definition of Done:

- [x] Drawing interaction feels responsive with no visible stutter on normal drag.
- [x] FFT graph updates correctly within one interaction frame after drawing changes.
- [ ] Dominant frequency peaks are plausible for simple test waves (e.g., sine-like input).
- [x] Clear/reset returns both waveform and FFT view to a known empty state.
- [x] Layout remains usable on common desktop widths without overlap/clipping.
- [x] No console errors during normal use of `/fft-draw`.
- [x] Route is discoverable from the home screen.
