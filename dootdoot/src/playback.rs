//! Live audio playback for canonical dootdoot buffers.

use std::{error::Error, fmt};

use dootdoot_core::SYNTH_SAMPLE_RATE_HZ;
use rodio::{DeviceSinkBuilder, Player, buffer::SamplesBuffer};

const PLAYBACK_CHANNELS: rodio::ChannelCount = rodio::nz!(1);
const PLAYBACK_SAMPLE_RATE: rodio::SampleRate = rodio::nz!(44_100);
const _: () = assert!(SYNTH_SAMPLE_RATE_HZ == 44_100);

/// Reports why live playback failed.
#[derive(Debug)]
pub struct PlaybackError {
    source: Box<dyn Error + 'static>,
}

impl PlaybackError {
    fn output_device(source: impl Error + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}

impl fmt::Display for PlaybackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "failed to open the default audio output device")
    }
}

impl Error for PlaybackError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Converts canonical signed 16-bit PCM samples into rodio's normalized `f32`
/// samples.
pub fn playback_samples(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|sample| f32::from(*sample) / 32_768.0_f32)
        .collect()
}

/// Plays a canonical mono buffer on the default audio output device.
///
/// # Errors
///
/// Returns an error if the default output device cannot be opened.
pub fn play_buffer(samples: &[i16]) -> Result<(), PlaybackError> {
    let sink = DeviceSinkBuilder::open_default_sink().map_err(PlaybackError::output_device)?;
    let player = Player::connect_new(sink.mixer());
    let source = SamplesBuffer::new(
        PLAYBACK_CHANNELS,
        PLAYBACK_SAMPLE_RATE,
        playback_samples(samples),
    );

    player.append(source);
    player.sleep_until_end();

    Ok(())
}
