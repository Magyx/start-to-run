use wasm_bindgen::prelude::*;
use web_sys::AudioContext;

/// Play a short beep.  `run == true` → high double-beep; `false` → low single beep.
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
        let cb = Closure::once(move || {
            play_tone(&ctx2, 1100.0, 0.15);
        });
        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                200,
            );
        cb.forget(); // leak is fine for a tiny one-shot
    }
}

fn play_tone(ctx: &AudioContext, freq: f64, dur: f64) {
    let osc = ctx.create_oscillator().unwrap();
    let gain = ctx.create_gain().unwrap();
    let _ = osc.connect_with_audio_node(&gain);
    let _ = gain.connect_with_audio_node(&ctx.destination());
    osc.frequency().set_value(freq as f32);
    gain.gain().set_value(0.3);
    let _ = osc.start();
    let _ = osc.stop_with_when(ctx.current_time() + dur);
}
