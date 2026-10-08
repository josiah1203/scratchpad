const CHECKERBOARD: &str =
    include_str!("../../../tests/fixtures/synthetic-media/checkerboard-4x4.ppm");
const SYNTHETIC_WAV: &[u8] =
    include_bytes!("../../../tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav");
const SYNTHETIC_Y4M: &[u8] = include_bytes!(
    "../../../tests/fixtures/synthetic-media/synthetic-gray-16x16-4frames-24fps.y4m"
);

#[test]
fn synthetic_ppm_fixture_is_well_formed() {
    let tokens: Vec<_> = CHECKERBOARD
        .lines()
        .filter_map(|line| line.split('#').next())
        .flat_map(str::split_whitespace)
        .collect();

    assert_eq!(tokens.first(), Some(&"P3"), "fixture must be an ASCII PPM image");
    let width: usize = tokens[1].parse().expect("width should be an integer");
    let height: usize = tokens[2].parse().expect("height should be an integer");
    let channel_max: u16 = tokens[3].parse().expect("channel maximum should be an integer");
    let samples = &tokens[4..];

    assert!(width > 0 && height > 0);
    assert!(channel_max > 0 && channel_max <= 255);
    assert_eq!(samples.len(), width * height * 3, "each pixel needs three RGB channels");
    assert!(
        samples
            .iter()
            .all(|sample| { sample.parse::<u16>().is_ok_and(|value| value <= channel_max) })
    );
}

#[test]
fn synthetic_ppm_fixture_decodes_with_the_pnm_image_decoder() {
    let decoded =
        image::load_from_memory_with_format(CHECKERBOARD.as_bytes(), image::ImageFormat::Pnm)
            .expect("the synthetic PPM fixture should decode through the PNM codec")
            .to_rgb8();

    assert_eq!(decoded.dimensions(), (4, 4));
    for y in 0..4 {
        for x in 0..4 {
            let expected = if (x + y) % 2 == 0 { [40, 60, 90] } else { [220, 180, 90] };
            assert_eq!(decoded.get_pixel(x, y).0, expected, "pixel ({x}, {y})");
        }
    }
}

#[test]
fn synthetic_pcm_wave_fixture_has_consistent_container_and_samples() {
    let wav = SYNTHETIC_WAV;
    assert!(wav.len() >= 44, "WAV must contain a complete PCM header");
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(&wav[12..16], b"fmt ");
    assert_eq!(&wav[36..40], b"data");

    let riff_size = u32::from_le_bytes(wav[4..8].try_into().expect("four-byte RIFF length"));
    let format_size = u32::from_le_bytes(wav[16..20].try_into().expect("four-byte format length"));
    let channels = u16::from_le_bytes(wav[22..24].try_into().expect("two-byte channel count"));
    let sample_rate = u32::from_le_bytes(wav[24..28].try_into().expect("four-byte sample rate"));
    let byte_rate = u32::from_le_bytes(wav[28..32].try_into().expect("four-byte byte rate"));
    let block_align = u16::from_le_bytes(wav[32..34].try_into().expect("two-byte block alignment"));
    let bits_per_sample = u16::from_le_bytes(wav[34..36].try_into().expect("two-byte bit depth"));
    let data_size = u32::from_le_bytes(wav[40..44].try_into().expect("four-byte data length"));

    assert_eq!(usize::try_from(riff_size).expect("RIFF size should fit usize") + 8, wav.len());
    assert_eq!(format_size, 16, "fixture should use the canonical PCM format chunk");
    assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1, "format should be integer PCM");
    assert_eq!(channels, 1);
    assert_eq!(sample_rate, 8_000);
    assert_eq!(byte_rate, sample_rate * u32::from(block_align));
    assert_eq!(block_align, 2);
    assert_eq!(bits_per_sample, 16);
    assert_eq!(usize::try_from(data_size).expect("data size should fit usize"), wav.len() - 44);
    assert_eq!(data_size % u32::from(block_align), 0);

    let expected_period = [0_i16, 8_192, 11_585, 8_192, 0, -8_192, -11_585, -8_192];
    let samples: Vec<i16> = wav[44..]
        .chunks_exact(2)
        .map(|sample| i16::from_le_bytes([sample[0], sample[1]]))
        .collect();
    assert_eq!(samples.len(), 800, "fixture should contain 100 ms at 8 kHz");
    assert!(samples.chunks_exact(expected_period.len()).all(|period| period == expected_period));
}

#[test]
fn synthetic_y4m_video_fixture_has_consistent_header_and_frames() {
    const WIDTH: usize = 16;
    const HEIGHT: usize = 16;
    const LUMA_BYTES: usize = WIDTH * HEIGHT;
    const CHROMA_BYTES: usize = LUMA_BYTES / 4;
    const FRAME_BYTES: usize = LUMA_BYTES + CHROMA_BYTES * 2;
    const HEADER: &[u8] = b"YUV4MPEG2 W16 H16 F24:1 Ip A1:1 C420jpeg\n";
    const FRAME_MARKER: &[u8] = b"FRAME\n";

    assert!(SYNTHETIC_Y4M.starts_with(HEADER));
    let mut offset = HEADER.len();

    for expected_luma in [32_u8, 96, 160, 224] {
        assert_eq!(&SYNTHETIC_Y4M[offset..offset + FRAME_MARKER.len()], FRAME_MARKER);
        offset += FRAME_MARKER.len();

        let luma = &SYNTHETIC_Y4M[offset..offset + LUMA_BYTES];
        assert!(luma.iter().all(|sample| *sample == expected_luma));
        offset += LUMA_BYTES;

        let chroma = &SYNTHETIC_Y4M[offset..offset + CHROMA_BYTES * 2];
        assert!(chroma.iter().all(|sample| *sample == 128));
        offset += CHROMA_BYTES * 2;
    }

    assert_eq!(SYNTHETIC_Y4M.len(), HEADER.len() + 4 * (FRAME_MARKER.len() + FRAME_BYTES));
    assert_eq!(offset, SYNTHETIC_Y4M.len(), "fixture must not contain trailing data");
}
