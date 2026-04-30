use img_parts::{jpeg::Jpeg, ImageEXIF, ImageICC};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("input is not a valid JPEG")]
    NotJpeg,

    #[error("failed to decode JPEG: {0}")]
    DecodeFailed(String),

    #[error("failed to encode JPEG: {0}")]
    EncodeFailed(String),
}

// ---------------------------------------------------------------------------
// 4.1 process_jpeg
// ---------------------------------------------------------------------------

/// Strip all metadata (EXIF, IPTC, XMP, comments) from a JPEG and recompress
/// it at the given quality level.
///
/// Requirements: 7.1, 7.2, 7.3
pub fn process_jpeg(input: &[u8], quality: u8) -> Result<Vec<u8>, ProcessError> {
    // --- Step 1: parse with img-parts and strip metadata segments ----------
    let mut jpeg = Jpeg::from_bytes(input.to_vec().into())
        .map_err(|_| ProcessError::NotJpeg)?;

    // Remove EXIF (APP1 with EXIF header) and ICC profile (APP2).
    jpeg.set_exif(None);
    jpeg.set_icc_profile(None);

    // Remove remaining APP1 (XMP), APP13 (IPTC/Photoshop), and COM (comments).
    jpeg.segments_mut().retain(|seg| {
        let marker = seg.marker();
        // APP1 = 0xE1, APP13 = 0xED, COM = 0xFE
        marker != 0xE1 && marker != 0xED && marker != 0xFE
    });

    // Re-serialise the stripped JPEG so mozjpeg can decode it.
    let stripped: Vec<u8> = jpeg.encoder().bytes().to_vec();

    // --- Step 2: decode with mozjpeg ---------------------------------------
    let mut decompress = mozjpeg::Decompress::new_mem(&stripped)
        .map_err(|e| ProcessError::DecodeFailed(e.to_string()))?
        .rgb()
        .map_err(|e| ProcessError::DecodeFailed(e.to_string()))?;

    let width = decompress.width();
    let height = decompress.height();

    // read_scanlines::<u8>() returns flat RGB bytes
    let pixels: Vec<u8> = decompress
        .read_scanlines::<u8>()
        .map_err(|e| ProcessError::DecodeFailed(e.to_string()))?;

    decompress
        .finish()
        .map_err(|e| ProcessError::DecodeFailed(e.to_string()))?;

    // --- Step 3: re-encode with mozjpeg at the requested quality -----------
    let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    compress.set_size(width, height);
    compress.set_quality(quality as f32);

    let output_buf: Vec<u8> = Vec::new();
    let mut started = compress
        .start_compress(output_buf)
        .map_err(|e| ProcessError::EncodeFailed(e.to_string()))?;

    started
        .write_scanlines(&pixels)
        .map_err(|e| ProcessError::EncodeFailed(e.to_string()))?;

    let output = started
        .finish()
        .map_err(|e| ProcessError::EncodeFailed(e.to_string()))?;

    Ok(output)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Minimal valid 1×1 JPEG (no metadata) used as a base for tests.
    // Generated with: convert -size 1x1 xc:red /tmp/red.jpg && xxd -i /tmp/red.jpg
    // -----------------------------------------------------------------------
    fn minimal_jpeg() -> Vec<u8> {
        // A tiny but valid 1×1 RGB JPEG produced by mozjpeg itself.
        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        compress.set_size(1, 1);
        compress.set_quality(75.0);
        let buf: Vec<u8> = Vec::new();
        let mut started = compress.start_compress(buf).unwrap();
        // One RGB pixel: red
        started.write_scanlines(&[255u8, 0, 0]).unwrap();
        started.finish().unwrap()
    }

    // -----------------------------------------------------------------------
    // 4.2 Property 7: JPEG processing produces valid JPEG output
    // Feature: jpeg-optimizer, Property 7: JPEG processing produces valid JPEG output
    // Validates: Requirements 7.1, 7.2, 7.3
    // -----------------------------------------------------------------------
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        #[test]
        fn prop_valid_jpeg_in_valid_jpeg_out(quality in 1u8..=100u8) {
            // Feature: jpeg-optimizer, Property 7: JPEG processing produces valid JPEG output
            // Validates: Requirements 7.1, 7.2, 7.3
            let input = minimal_jpeg();
            let output = process_jpeg(&input, quality)
                .expect("process_jpeg should succeed on a valid JPEG");

            // Valid JPEG starts with FF D8 and ends with FF D9
            prop_assert!(output.starts_with(&[0xFF, 0xD8]),
                "output should start with FF D8 (JPEG SOI marker)");
            prop_assert!(output.ends_with(&[0xFF, 0xD9]),
                "output should end with FF D9 (JPEG EOI marker)");
        }
    }

    // -----------------------------------------------------------------------
    // 4.3 Property 8: Metadata stripping removes EXIF/IPTC/XMP
    // Feature: jpeg-optimizer, Property 8: Metadata stripping removes EXIF/IPTC/XMP
    // Validates: Requirements 7.1
    // -----------------------------------------------------------------------
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        #[test]
        fn prop_metadata_stripped(quality in 1u8..=100u8) {
            // Feature: jpeg-optimizer, Property 8: Metadata stripping removes EXIF/IPTC/XMP
            // Validates: Requirements 7.1
            //
            // Build a JPEG that contains a synthetic APP1 (EXIF) segment.
            let base = minimal_jpeg();
            let mut jpeg = Jpeg::from_bytes(base.into()).unwrap();

            // Inject a fake EXIF APP1 segment (marker 0xE1, "Exif\0\0" header).
            let fake_exif: img_parts::Bytes = b"Exif\0\0fake_exif_data".to_vec().into();
            jpeg.set_exif(Some(fake_exif));
            let with_exif: Vec<u8> = jpeg.encoder().bytes().to_vec();

            // Confirm the EXIF marker is present before processing.
            let has_exif_before = contains_app1_exif(&with_exif);
            prop_assume!(has_exif_before);

            let output = process_jpeg(&with_exif, quality)
                .expect("process_jpeg should succeed");

            // After processing, no APP1 (0xFFE1) segment should remain.
            prop_assert!(!contains_app1_exif(&output),
                "output should not contain an APP1/EXIF marker after stripping");
        }
    }

    // -----------------------------------------------------------------------
    // 4.4 Property 9: Invalid JPEG input returns an error
    // Feature: jpeg-optimizer, Property 9: Invalid JPEG input returns an error
    // Validates: Requirements 7.4
    // -----------------------------------------------------------------------
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        #[test]
        fn prop_invalid_input_returns_error(
            // Generate byte sequences that do NOT start with the JPEG SOI marker FF D8.
            first_byte in prop::num::u8::ANY.prop_filter("not FF", |b| *b != 0xFF),
            rest in prop::collection::vec(prop::num::u8::ANY, 0..64),
        ) {
            // Feature: jpeg-optimizer, Property 9: Invalid JPEG input returns an error
            // Validates: Requirements 7.4
            let mut bad_input = vec![first_byte];
            bad_input.extend_from_slice(&rest);

            let result = process_jpeg(&bad_input, 75);
            prop_assert!(result.is_err(),
                "expected an error for non-JPEG input starting with 0x{:02X}", first_byte);
        }
    }

    // -----------------------------------------------------------------------
    // Helper: scan raw JPEG bytes for an APP1 marker (0xFF 0xE1).
    // -----------------------------------------------------------------------
    fn contains_app1_exif(data: &[u8]) -> bool {
        // Walk the JPEG segment list looking for 0xFF 0xE1.
        let mut i = 0;
        while i + 1 < data.len() {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                if marker == 0xE1 {
                    return true;
                }
                // Skip padding 0xFF bytes
                if marker == 0xFF {
                    i += 1;
                    continue;
                }
                // SOI / EOI have no length field
                if marker == 0xD8 || marker == 0xD9 {
                    i += 2;
                    continue;
                }
                // All other segments have a 2-byte length after the marker
                if i + 3 < data.len() {
                    let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    i += 2 + len;
                } else {
                    break;
                }
            } else {
                i += 1;
            }
        }
        false
    }
}
