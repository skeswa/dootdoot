//! WAV serialization for canonical dootdoot buffers.

use std::io::{Cursor, Seek, Write};

use thiserror::Error;

use crate::{SYNTH_SAMPLE_RATE_HZ, SequenceEvent, sequence_utterance};

/// Gives the fixed scale from normalized `f64` samples to signed 16-bit PCM.
pub const PCM_I16_SCALE: f64 = 32_768.0;

/// Marks the WAV serialization module in the public facade.
#[derive(Debug)]
pub struct WavWriter;

/// Reports why WAV serialization failed.
#[derive(Debug, Error)]
pub enum WavError {
    /// Wraps errors from the WAV encoder.
    #[error("failed to write WAV data: {0}")]
    Encoder(#[from] hound::Error),
}

/// Renders the canonical signed 16-bit mono audio buffer for sequenced events.
pub fn render_canonical_buffer(events: &[SequenceEvent]) -> Vec<i16> {
    sequence_utterance(events)
        .samples()
        .iter()
        .copied()
        .map(quantize_sample)
        .collect()
}

/// Serializes canonical samples to an in-memory WAV byte vector.
///
/// # Errors
///
/// Returns an error if the WAV encoder rejects the stream.
pub fn wav_bytes(samples: &[i16]) -> Result<Vec<u8>, WavError> {
    let mut cursor = Cursor::new(Vec::new());

    write_wav(&mut cursor, samples)?;

    Ok(cursor.into_inner())
}

/// Writes canonical samples as 44.1 kHz, 16-bit, mono PCM WAV.
///
/// # Errors
///
/// Returns an error if the WAV encoder rejects the stream or writer.
pub fn write_wav<W>(writer: W, samples: &[i16]) -> Result<(), WavError>
where
    W: Write + Seek,
{
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SYNTH_SAMPLE_RATE_HZ,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav_writer = hound::WavWriter::new(writer, spec)?;

    for sample in samples {
        wav_writer.write_sample(*sample)?;
    }

    wav_writer.finalize()?;

    Ok(())
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
