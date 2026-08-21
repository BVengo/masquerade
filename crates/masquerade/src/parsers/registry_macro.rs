//! Format-registry code generation.

macro_rules! define_formats {
    (
        $(
            $variant:ident => {
                module: $module:ident,
                extension: $extension:literal,
                aliases: [$($alias:literal),* $(,)?]
            }
        ),+ $(,)?
    ) => {
        $(mod $module;)+

        /// A file type understood by Masquerade.
        ///
        /// New variants may be added as more formats are implemented.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum MediaType {
            $($variant),+
        }

        impl MediaType {
            /// Every file type supported by this version of the crate.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[must_use]
            pub fn from_path(path: &std::path::Path) -> Option<Self> {
                path.extension()
                    .and_then(|extension| extension.to_str())
                    .and_then(|extension| extension.parse().ok())
            }

            #[must_use]
            pub const fn extension(self) -> &'static str {
                match self {
                    $(Self::$variant => $extension),+
                }
            }

            /// All recognised extensions for this file type, without a dot.
            #[must_use]
            pub const fn extensions(self) -> &'static [&'static str] {
                match self {
                    $(Self::$variant => &[$extension, $($alias),*]),+
                }
            }

            pub(crate) fn check_signature<R: std::io::Read + std::io::Seek + ?Sized>(
                self,
                reader: &mut R,
                limits: &crate::ValidationLimits,
            ) -> std::io::Result<crate::CheckResult> {
                let valid = match self {
                    $(Self::$variant => $module::signature(reader, limits)),+
                }?;
                Ok(if valid {
                    crate::CheckResult::valid()
                } else {
                    crate::CheckResult::invalid(
                        crate::DiagnosticCode::SignatureMismatch,
                        format!("File signature does not match .{}", self.extension()),
                    )
                })
            }

            pub(crate) fn validate_structure<R: std::io::Read + std::io::Seek + ?Sized>(
                self,
                reader: &mut R,
                limits: &crate::ValidationLimits,
            ) -> std::io::Result<crate::CheckResult> {
                match self {
                    $(Self::$variant => $module::validate_structure(reader, limits)),+
                }
            }
        }

        impl std::str::FromStr for MediaType {
            type Err = UnsupportedMediaType;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let normalized = value.trim_start_matches('.').to_ascii_lowercase();
                match normalized.as_str() {
                    $($extension $(| $alias)* => Ok(Self::$variant)),+,
                    _ => Err(UnsupportedMediaType(value.to_owned())),
                }
            }
        }
    };
}

pub(super) use define_formats;
