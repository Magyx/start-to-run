use std::cell::Cell;

use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, SpeechSynthesis, SpeechSynthesisUtterance};

const VOICE_LANG: &str = "en-GB";
const SAY_RUN: &str = "Run";
const SAY_WALK: &str = "Walk";
const SAY_DONE: &str = "Finished";

thread_local! {
    static INITIALISED: Cell<bool> = const { Cell::new(false) };
    static VOLUME: Cell<f64> = const { Cell::new(0.7) };
}

pub fn set_volume(volume: f64) {
    VOLUME.with(|v| v.set(volume.clamp(0.0, 1.0)));
}

/// Enable audio from the timer's start tap.
pub fn init() {
    if INITIALISED.with(|i| i.replace(true)) {
        return;
    }
    set_audio_session("transient");
    if let Some(synth) = synth() {
        if let Ok(u) = SpeechSynthesisUtterance::new_with_text(" ") {
            u.set_volume(0.0);
            synth.speak(&u);
        }
    }
}

/// Announce a run/walk transition, falling back to a beep if speech is unavailable.
pub fn cue(run: bool) {
    if !speak(if run { SAY_RUN } else { SAY_WALK }) {
        beep(run);
    }
}

/// Announce that the session is finished.
pub fn finished() {
    if !speak(SAY_DONE) {
        beep(false);
    }
}

fn synth() -> Option<SpeechSynthesis> {
    let window = web_sys::window()?;
    // Older browsers may not support speech synthesis.
    if !js_sys::Reflect::has(&window, &JsValue::from_str("speechSynthesis")).unwrap_or(false) {
        return None;
    }
    window.speech_synthesis().ok()
}

fn speak(text: &str) -> bool {
    let Some(synth) = synth() else { return false };
    let Ok(utterance) = SpeechSynthesisUtterance::new_with_text(text) else {
        return false;
    };
    utterance.set_lang(VOICE_LANG);
    utterance.set_volume(VOLUME.with(Cell::get) as f32);
    // Replace any cue still playing.
    if synth.speaking() || synth.pending() {
        synth.cancel();
    }
    synth.speak(&utterance);
    true
}

/// Set the audio session type when the browser supports it.
fn set_audio_session(kind: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let nav = window.navigator();
    if let Ok(session) = js_sys::Reflect::get(&nav, &JsValue::from_str("audioSession")) {
        if !session.is_undefined() && !session.is_null() {
            let _ = js_sys::Reflect::set(
                &session,
                &JsValue::from_str("type"),
                &JsValue::from_str(kind),
            );
        }
    }
}

/// Play a high double beep for running or a low single beep for walking.
pub fn beep(run: bool) {
    let ctx = match AudioContext::new() {
        Ok(c) => c,
        Err(_) => return,
    };

    play_tone(&ctx, if run { 880.0 } else { 440.0 }, 0.15);

    if run {
        // Schedule a second beep 200 ms later.
        let ctx2 = match AudioContext::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let cb = Closure::once_into_js(move || {
            play_tone(&ctx2, 1100.0, 0.15);
        });
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 200);
    }
}

fn play_tone(ctx: &AudioContext, freq: f64, dur: f64) {
    let osc = ctx.create_oscillator().unwrap();
    let gain = ctx.create_gain().unwrap();
    let _ = osc.connect_with_audio_node(&gain);
    let _ = gain.connect_with_audio_node(&ctx.destination());
    osc.frequency().set_value(freq as f32);
    gain.gain().set_value((0.3 * VOLUME.with(Cell::get)) as f32);
    let _ = osc.start();
    let _ = osc.stop_with_when(ctx.current_time() + dur);
}
