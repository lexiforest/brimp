use std::borrow::Cow;

/// A DOMString stored as WTF-8 at the byte-oriented Lexbor boundary.
///
/// Well-formed Unicode has its usual UTF-8 representation. Unpaired UTF-16
/// surrogates use their three-byte encoding; they are never placed in a Rust
/// `str`. The native arena remains the only owner of live DOM attribute and text data.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DomString(Vec<u8>);

impl DomString {
    pub fn from_utf16(units: &[u16]) -> Self {
        let mut bytes = Vec::with_capacity(units.len());
        for point in char::decode_utf16(units.iter().copied()) {
            match point {
                Ok(point) => bytes.extend_from_slice(point.encode_utf8(&mut [0; 4]).as_bytes()),
                Err(error) => {
                    let unit = error.unpaired_surrogate();
                    bytes.extend_from_slice(&[
                        0xe0 | (unit >> 12) as u8,
                        0x80 | ((unit >> 6) & 0x3f) as u8,
                        0x80 | (unit & 0x3f) as u8,
                    ]);
                }
            }
        }
        Self(bytes)
    }

    pub fn to_utf16(&self) -> Vec<u16> {
        let mut units = Vec::with_capacity(self.0.len());
        let mut bytes = self.0.as_slice();
        while !bytes.is_empty() {
            match std::str::from_utf8(bytes) {
                Ok(text) => {
                    units.extend(text.encode_utf16());
                    break;
                }
                Err(error) => {
                    let (valid, rest) = bytes.split_at(error.valid_up_to());
                    units.extend(std::str::from_utf8(valid).unwrap().encode_utf16());
                    if rest.len() >= 3
                        && rest[0] == 0xed
                        && (0xa0..=0xbf).contains(&rest[1])
                        && (0x80..=0xbf).contains(&rest[2])
                    {
                        units.push(
                            (u16::from(rest[0] & 0x0f) << 12)
                                | (u16::from(rest[1] & 0x3f) << 6)
                                | u16::from(rest[2] & 0x3f),
                        );
                        bytes = &rest[3..];
                    } else {
                        // Match Rust's lossy UTF-8 conversion for malformed native
                        // bytes other than the explicitly supported surrogates.
                        units.push(0xfffd);
                        bytes = &rest[error.error_len().unwrap_or(rest.len())..];
                    }
                }
            }
        }
        units
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }

    /// Convert for consumers that require Unicode scalar values, such as URLs.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        match self.as_str() {
            Some(text) => Cow::Borrowed(text),
            None => Cow::Owned(String::from_utf16_lossy(&self.to_utf16())),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&str> for DomString {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl From<&String> for DomString {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<String> for DomString {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::DomString;

    #[test]
    fn round_trip_code_units_and_preserve_utf8_for_scalar_values() {
        let units: Vec<_> = (0..=u16::MAX).flat_map(|unit| [unit, 0]).collect();
        assert_eq!(DomString::from_utf16(&units).to_utf16(), units);
        for units in [
            vec![],
            vec![0xd800, 0xdc00],
            vec![0xdc00, 0xd800],
            vec![0xd800, 0xd800, 0xdc00, 0xdc00],
        ] {
            assert_eq!(DomString::from_utf16(&units).to_utf16(), units);
        }
        let text = "a\0é你好🦀\u{10ffff}";
        let encoded = DomString::from_utf16(&text.encode_utf16().collect::<Vec<_>>());
        assert_eq!(encoded.as_bytes(), text.as_bytes());
        assert_eq!(encoded.as_str(), Some(text));
        assert_eq!(
            DomString::from_utf16(&[0xd800, 0, 0xdc00]).to_string_lossy(),
            "�\0�"
        );
    }

    #[test]
    fn native_invalid_bytes_remain_lossy_except_surrogate_encodings() {
        for bytes in [
            b"\xffx".as_slice(),
            b"\xf0\x90",
            b"\xed\xa0",
            b"\xc0\x80",
            b"\xed\x80\xff",
        ] {
            assert_eq!(
                DomString::from_bytes(bytes).to_string_lossy(),
                String::from_utf8_lossy(bytes)
            );
        }
        assert_eq!(DomString::from_bytes(b"\xed\xa0\x80").to_utf16(), [0xd800]);
    }
}
