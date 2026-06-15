//! Image metadata extraction.

use crate::metadata::ImageMetadata;

/// Extracts image metadata from known image headers.
#[must_use]
pub fn extract_image_metadata(mime_type: &str, bytes: &[u8]) -> ImageMetadata {
    match mime_type {
        "image/png" => png_metadata(bytes),
        "image/gif" => gif_metadata(bytes),
        "image/bmp" => bmp_metadata(bytes),
        "image/jpeg" => jpeg_metadata(bytes),
        "image/svg+xml" => svg_metadata(bytes),
        _ => ImageMetadata {
            width: None,
            height: None,
            color_type: None,
        },
    }
}

fn png_metadata(bytes: &[u8]) -> ImageMetadata {
    let width = read_be_u32(bytes, 16);
    let height = read_be_u32(bytes, 20);
    let color_type = bytes.get(25).map(|kind| match kind {
        0 => "grayscale",
        2 => "truecolor",
        3 => "indexed",
        4 => "grayscale-alpha",
        6 => "truecolor-alpha",
        _ => "unknown",
    });
    ImageMetadata {
        width,
        height,
        color_type: color_type.map(str::to_string),
    }
}

fn gif_metadata(bytes: &[u8]) -> ImageMetadata {
    ImageMetadata {
        width: read_le_u16(bytes, 6).map(u32::from),
        height: read_le_u16(bytes, 8).map(u32::from),
        color_type: Some("indexed".to_string()),
    }
}

fn bmp_metadata(bytes: &[u8]) -> ImageMetadata {
    ImageMetadata {
        width: read_le_u32(bytes, 18),
        height: read_le_u32(bytes, 22),
        color_type: None,
    }
}

fn jpeg_metadata(bytes: &[u8]) -> ImageMetadata {
    let mut index = 2;
    while index + 9 < bytes.len() {
        if bytes[index] != 0xff {
            index += 1;
            continue;
        }
        let marker = bytes[index + 1];
        let length = read_be_u16(bytes, index + 2).map_or(0, usize::from);
        if matches!(marker, 0xc0..=0xc3) {
            return ImageMetadata {
                height: read_be_u16(bytes, index + 5).map(u32::from),
                width: read_be_u16(bytes, index + 7).map(u32::from),
                color_type: Some("jpeg".to_string()),
            };
        }
        if length < 2 {
            break;
        }
        index += 2 + length;
    }
    ImageMetadata {
        width: None,
        height: None,
        color_type: Some("jpeg".to_string()),
    }
}

fn svg_metadata(bytes: &[u8]) -> ImageMetadata {
    let text = String::from_utf8_lossy(bytes);
    ImageMetadata {
        width: extract_svg_number(&text, "width"),
        height: extract_svg_number(&text, "height"),
        color_type: Some("vector".to_string()),
    }
}

fn extract_svg_number(text: &str, attr: &str) -> Option<u32> {
    let needle = format!("{attr}=\"");
    let start = text.find(&needle)? + needle.len();
    let value = text[start..].split('"').next()?;
    value
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([slice[0], slice[1]]))
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
mod tests {
    use super::extract_image_metadata;

    #[test]
    fn extracts_png_dimensions() {
        let mut bytes = b"\x89PNG\r\n\x1a\nxxxxIHDR".to_vec();
        bytes.extend_from_slice(&512_u32.to_be_bytes());
        bytes.extend_from_slice(&256_u32.to_be_bytes());
        bytes.extend_from_slice(&[8, 6]);

        let metadata = extract_image_metadata("image/png", &bytes);

        assert_eq!(metadata.width, Some(512));
        assert_eq!(metadata.height, Some(256));
    }
}
