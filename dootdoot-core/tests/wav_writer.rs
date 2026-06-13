//! WAV writer tests.

use dootdoot_core::{SYNTH_SAMPLE_RATE_HZ, wav_bytes};

#[test]
fn wav_writer_serializes_44100hz_16bit_mono_pcm() {
    let samples = [0_i16, i16::MAX, i16::MIN];
    let bytes = wav_bytes(&samples).expect("wav serialization should succeed");

    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");
    assert_eq!(read_u16(&bytes, 20), 1);
    assert_eq!(read_u16(&bytes, 22), 1);
    assert_eq!(read_u32(&bytes, 24), SYNTH_SAMPLE_RATE_HZ);
    assert_eq!(read_u16(&bytes, 34), 16);
    assert_eq!(&bytes[36..40], b"data");
    assert_eq!(read_u32(&bytes, 40), 6);
    assert_eq!(&bytes[44..50], &[0, 0, 255, 127, 0, 128]);
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("u16 field should be present"),
    )
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("u32 field should be present"),
    )
}
