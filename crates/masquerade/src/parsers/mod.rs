//! Format registry and parser dispatch.

use std::fmt;

mod registry_macro;

registry_macro::define_formats! {
    Avif => { module: avif, extension: "avif", aliases: [] },
    Avi => { module: avi, extension: "avi", aliases: [] },
    Bmp => { module: bmp, extension: "bmp", aliases: [] },
    Jpeg => { module: jpeg, extension: "jpeg", aliases: ["jpg"] },
    M4a => { module: m4a, extension: "m4a", aliases: [] },
    Mov => { module: mov, extension: "mov", aliases: [] },
    Mp3 => { module: mp3, extension: "mp3", aliases: [] },
    Mp4 => { module: mp4, extension: "mp4", aliases: [] },
    Ogg => { module: ogg, extension: "ogg", aliases: [] },
    Png => { module: png, extension: "png", aliases: [] },
    Vtt => { module: vtt, extension: "vtt", aliases: [] },
    Wav => { module: wav, extension: "wav", aliases: [] },
    Webp => { module: webp, extension: "webp", aliases: [] },
}

impl fmt::Display for MediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.extension())
    }
}

/// Error returned when parsing an unsupported file type name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedMediaType(String);

impl fmt::Display for UnsupportedMediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unsupported file type: {}", self.0)
    }
}

impl std::error::Error for UnsupportedMediaType {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_extensions_case_insensitively() {
        assert_eq!(".JPG".parse(), Ok(MediaType::Jpeg));
    }

    #[test]
    fn registry_extensions_round_trip() {
        for media_type in MediaType::ALL {
            for extension in media_type.extensions() {
                assert_eq!(extension.parse(), Ok(*media_type));
            }
        }
    }

    #[test]
    fn registry_extensions_are_unique() {
        let mut extensions = std::collections::HashSet::new();
        for media_type in MediaType::ALL {
            for extension in media_type.extensions() {
                assert!(
                    extensions.insert(extension),
                    "duplicate extension: {extension}"
                );
            }
        }
    }
}
