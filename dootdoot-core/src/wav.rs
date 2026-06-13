//! WAV serialization for canonical dootdoot buffers.

use crate::{SequenceEvent, sequence_utterance};

/// Gives the fixed scale from normalized `f64` samples to signed 16-bit PCM.
pub const PCM_I16_SCALE: f64 = 32_768.0;

/// Marks the WAV serialization module in the public facade.
#[derive(Debug)]
pub struct WavWriter;

/// Renders the canonical signed 16-bit mono audio buffer for sequenced events.
pub fn render_canonical_buffer(events: &[SequenceEvent]) -> Vec<i16> {
    sequence_utterance(events)
        .samples()
        .iter()
        .copied()
        .map(quantize_sample)
        .collect()
}

/// Quantizes one normalized audio sample to signed 16-bit PCM.
pub fn quantize_sample(sample: f64) -> i16 {
    if sample.is_nan() {
        return 0;
    }

    if sample == f64::INFINITY {
        return i16::MAX;
    }

    if sample == f64::NEG_INFINITY {
        return i16::MIN;
    }

    let scaled = sample * PCM_I16_SCALE;
    let rounded = round_half_to_even_for_i16(scaled);

    integral_f64_to_i16(rounded)
}

fn round_half_to_even_for_i16(value: f64) -> f64 {
    if value >= f64::from(i16::MAX) {
        return f64::from(i16::MAX);
    }

    if value <= f64::from(i16::MIN) + 0.5 {
        return f64::from(i16::MIN);
    }

    let truncated = value.trunc();
    let fraction = (value - truncated).abs();

    if fraction < 0.5 {
        return truncated;
    }

    let direction = if value.is_sign_negative() { -1.0 } else { 1.0 };

    if fraction > 0.5 {
        return truncated + direction;
    }

    if truncated % 2.0 == 0.0 {
        truncated
    } else {
        truncated + direction
    }
}

fn integral_f64_to_i16(value: f64) -> i16 {
    let mut low = i32::from(i16::MIN);
    let mut high = i32::from(i16::MAX);

    while low <= high {
        let midpoint = low + ((high - low) / 2);
        let midpoint_value = f64::from(midpoint);

        if midpoint_value < value {
            low = midpoint + 1;
        } else if midpoint_value > value {
            high = midpoint - 1;
        } else {
            return match i16::try_from(midpoint) {
                Ok(value) => value,
                Err(_) if midpoint.is_negative() => i16::MIN,
                Err(_) => i16::MAX,
            };
        }
    }

    if value.is_sign_negative() {
        i16::MIN
    } else {
        i16::MAX
    }
}
