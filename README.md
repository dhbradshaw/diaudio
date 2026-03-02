# Diaudio

Playing with sound and dioxus.

## Pages

### Home screen

Route: `/`

This is just a pointer to all the different tools, pages, and games.

### FFT Draw

Route: `/fft-draw`

Draw a wave form on the left and see its FFT on the right.

Implementation checklist:

- [x] Add page component and route registration for `/fft-draw`.
- [x] Build a two-panel layout (waveform draw area on left, FFT output on right).
- [x] Implement pointer/mouse drawing on the waveform canvas.
- [x] Normalize drawn points into a fixed-size sample buffer.
- [x] Run FFT on the sample buffer (real input to frequency bins).
- [x] Convert FFT output to magnitudes (linear or dB scale).
- [x] Render FFT bins as a chart/bars on the right panel.
- [ ] Add controls for clear/reset and sample-size selection.
- [ ] Add axis labels or frequency markers for readability.
- [ ] Handle empty/flat input and other edge cases gracefully.
- [ ] Verify behavior in wasm/web target and document any limitations.

Definition of Done:

- [ ] Drawing interaction feels responsive with no visible stutter on normal drag.
- [ ] FFT graph updates correctly within one interaction frame after drawing changes.
- [ ] Dominant frequency peaks are plausible for simple test waves (e.g., sine-like input).
- [ ] Clear/reset returns both waveform and FFT view to a known empty state.
- [ ] Layout remains usable on common desktop widths without overlap/clipping.
- [ ] No console errors during normal use of `/fft-draw`.
- [ ] Route is discoverable from the home screen.
