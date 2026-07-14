//! Browser bindings for the deterministic renderer.

use wasm_bindgen::prelude::{JsError, wasm_bindgen};

use crate::{render_text_canonical_buffer, wav_bytes};

/// Renders arbitrary text as a complete `VOICE_V12` PCM WAV file.
///
/// # Errors
///
/// Returns a JavaScript error when the embedded tokenizer, semantic mapping, or
/// WAV serializer cannot process the input.
#[wasm_bindgen]
pub fn render_wav(text: &str) -> Result<Vec<u8>, JsError> {
    let samples =
        render_text_canonical_buffer(text).map_err(|error| JsError::new(&error.to_string()))?;
    wav_bytes(&samples).map_err(|error| JsError::new(&error.to_string()))
}
