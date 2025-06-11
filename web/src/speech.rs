#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

#[cfg(target_arch = "wasm32")]
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window`")
}

#[cfg(target_arch = "wasm32")]
pub fn speak(text: &str) {
    let win = window();
    if let Some(synth) = win.speech_synthesis() {
        if let Ok(utter) = web_sys::SpeechSynthesisUtterance::new_with_text(text) {
            synth.speak(&utter);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn start_stt<F: FnMut(String) + 'static>(mut callback: F) {
    use web_sys::{
        SpeechRecognition, SpeechRecognitionAlternative, SpeechRecognitionEvent,
        SpeechRecognitionResult,
    };

    if let Ok(rec) = SpeechRecognition::new() {
        rec.set_lang("en-US");
        rec.set_interim_results(false);
        let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
            if let Ok(event) = e.dyn_into::<SpeechRecognitionEvent>() {
                if let Some(res) = event.results().get(0) {
                    let res: SpeechRecognitionResult = res.dyn_into().unwrap();
                    if let Some(alt) = res.get(0) {
                        let alt: SpeechRecognitionAlternative = alt.dyn_into().unwrap();
                        callback(alt.transcript());
                    }
                }
            }
        });
        rec.set_onresult(Some(closure.as_ref().unchecked_ref()));
        rec.start().ok();
        closure.forget();
        std::mem::forget(rec);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn speak(_text: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_stt<F: FnMut(String) + 'static>(_callback: F) {}
