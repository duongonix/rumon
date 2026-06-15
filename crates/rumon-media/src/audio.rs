//! Audio metadata extraction.

use crate::metadata::AudioMetadata;

/// Extracts lightweight audio metadata from headers.
#[must_use]
pub fn extract_audio_metadata(mime_type: &str, bytes: &[u8]) -> AudioMetadata {
    match mime_type {
        "audio/wav" => wav_metadata(bytes),
        _ => AudioMetadata {
            duration_ms: None,
            bitrate: None,
            channels: None,
            sample_rate: None,
        },
    }
}

fn wav_metadata(bytes: &[u8]) -> AudioMetadata {
    let channels = read_le_u16(bytes, 22);
    let sample_rate = read_le_u32(bytes, 24);
    let byte_rate = read_le_u32(bytes, 28);
    let data_size = find_data_chunk_size(bytes);
    let duration_ms = data_size
        .zip(byte_rate)
        .and_then(|(size, rate)| (rate > 0).then_some((u64::from(size) * 1000) / u64::from(rate)));
    AudioMetadata {
        duration_ms,
        bitrate: byte_rate.map(|rate| rate.saturating_mul(8)),
        channels,
        sample_rate,
    }
}

fn find_data_chunk_size(bytes: &[u8]) -> Option<u32> {
    bytes
        .windows(4)
        .position(|window| window == b"data")
        .and_then(|position| read_le_u32(bytes, position + 4))
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
mod tests {
    use super::extract_audio_metadata;

    #[test]
    fn extracts_wav_channels_and_rate() {
        let mut bytes = vec![0; 44];
        bytes[0..4].copy_from_slice(b"RIFF");
        bytes[8..12].copy_from_slice(b"WAVE");
        bytes[22..24].copy_from_slice(&2_u16.to_le_bytes());
        bytes[24..28].copy_from_slice(&44_100_u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&176_400_u32.to_le_bytes());
        bytes[36..40].copy_from_slice(b"data");
        bytes[40..44].copy_from_slice(&176_400_u32.to_le_bytes());

        let metadata = extract_audio_metadata("audio/wav", &bytes);

        assert_eq!(metadata.channels, Some(2));
        assert_eq!(metadata.sample_rate, Some(44_100));
        assert_eq!(metadata.duration_ms, Some(1000));
    }
}
