use std::path::{Path, PathBuf};

use masquerade::{Inspector, MediaType, ValidationStatus, inspect};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name)
}

#[test]
fn validates_existing_media_corpus() {
    for name in [
        "valid_file.avif",
        "valid_file.avi",
        "valid_file.jpeg",
        "valid_file.jpg",
        "valid_file.mov",
        "valid_file.mp3",
        "valid_file.mp4",
        "valid_file.ogg",
        "valid_file.png",
        "valid_file.wav",
        "valid_file.webp",
    ] {
        let result = inspect(fixture(name)).unwrap();
        assert_eq!(
            result.status(),
            ValidationStatus::Valid,
            "failed to validate {name}: {result:?}"
        );
    }
}

#[test]
fn validates_minimal_bmp() {
    let mut data = Vec::new();
    data.extend_from_slice(b"BM");
    data.extend_from_slice(&58_u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(&54_u32.to_le_bytes());
    data.extend_from_slice(&40_u32.to_le_bytes());
    data.extend_from_slice(&1_i32.to_le_bytes());
    data.extend_from_slice(&1_i32.to_le_bytes());
    data.extend_from_slice(&1_u16.to_le_bytes());
    data.extend_from_slice(&24_u16.to_le_bytes());
    data.extend_from_slice(&[0; 24]);
    data.extend_from_slice(&[0; 4]);

    let result = Inspector::new()
        .inspect_bytes(&data, MediaType::Bmp)
        .unwrap();
    assert_eq!(result.status(), ValidationStatus::Valid, "{result:?}");
}

#[test]
fn rejects_extension_spoofing() {
    let result = inspect(fixture("invalid_file.png")).unwrap();
    assert_eq!(result.signature().status(), ValidationStatus::Invalid);
}

#[test]
fn rejects_malformed_trailing_bmff_data() {
    fn push_box(data: &mut Vec<u8>, kind: [u8; 4], payload: &[u8]) {
        let size = u32::try_from(payload.len() + 8).unwrap();
        data.extend_from_slice(&size.to_be_bytes());
        data.extend_from_slice(&kind);
        data.extend_from_slice(payload);
    }

    let mut data = Vec::new();
    push_box(&mut data, *b"ftyp", b"mp42\0\0\0\0mp42");
    push_box(&mut data, *b"moov", b"");
    push_box(&mut data, *b"mdat", b"");
    data.extend_from_slice(&[0, 0, 0, 8]);

    let result = Inspector::new()
        .inspect_bytes(&data, MediaType::Mp4)
        .unwrap();
    assert_eq!(result.signature().status(), ValidationStatus::Valid);
    assert_eq!(
        result.structure().unwrap().diagnostic().unwrap().code(),
        masquerade::DiagnosticCode::InvalidBoxStructure
    );
}

#[test]
fn rejects_riff_trailing_data() {
    let mut data = b"RIFF\x04\0\0\0WAVE".to_vec();
    data.push(0);

    let result = Inspector::new()
        .inspect_bytes(&data, MediaType::Wav)
        .unwrap();
    assert_eq!(result.signature().status(), ValidationStatus::Valid);
    assert_eq!(
        result.structure().unwrap().diagnostic().unwrap().code(),
        masquerade::DiagnosticCode::RiffSizeMismatch
    );
}
