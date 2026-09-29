//! Encoding of the text payload stored in type-17 text elements.
//!
//! Only two forms are evidenced (FN-T02): plain ASCII bytes, and the marker
//! `ff fe 01 00` followed by Windows-1252 bytes. Latvian letters such as
//! `ā č ē ģ ī ķ ļ ņ ū` are not in Windows-1252, so the other forms below are
//! experiments (H-T03, H-T04) until a MicroStation-made file confirms one.

use crate::error::{Result, WriteError};

/// Marker observed before Windows-1252 text that contains non-ASCII bytes.
pub const ESCAPED_CODEPAGE_MARKER: [u8; 4] = [0xff, 0xfe, 0x01, 0x00];
/// Marker observed before UTF-16LE strings in string linkages (FN-L02).
pub const UTF16_MARKER: [u8; 2] = [0xff, 0xfd];

/// How to store a text string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextEncoding {
    /// ASCII when possible, else escaped Windows-1252; anything else is an
    /// error until the storage of Latvian text is confirmed.
    #[default]
    Auto,
    /// Plain 7-bit ASCII (FN-T02).
    Ascii,
    /// `ff fe 01 00` + Windows-1252 bytes (FN-T02).
    EscapedCp1252,
    /// EXPERIMENT H-T03: `ff fe 01 00` + Windows-1257 (Baltic) bytes.
    EscapedCp1257,
    /// EXPERIMENT H-T04: `ff fd` + UTF-16LE, as used by string linkages.
    Utf16,
}

/// Encode `text` into the exact bytes stored after the text header.
pub fn encode_text(text: &str, encoding: TextEncoding) -> Result<Vec<u8>> {
    match encoding {
        TextEncoding::Auto => {
            if text.is_ascii() {
                Ok(text.as_bytes().to_vec())
            } else {
                encode_text(text, TextEncoding::EscapedCp1252).map_err(|_| {
                    WriteError::InvalidElement(format!(
                        "text {text:?} is not representable in Windows-1252; choose an \
                         experimental encoding (escaped-cp1257 or utf16) explicitly"
                    ))
                })
            }
        }
        TextEncoding::Ascii => {
            if text.is_ascii() {
                Ok(text.as_bytes().to_vec())
            } else {
                Err(WriteError::InvalidElement(format!(
                    "text {text:?} is not ASCII"
                )))
            }
        }
        TextEncoding::EscapedCp1252 => escaped(text, encoding_rs::WINDOWS_1252),
        TextEncoding::EscapedCp1257 => escaped(text, encoding_rs::WINDOWS_1257),
        TextEncoding::Utf16 => {
            let mut out = UTF16_MARKER.to_vec();
            for unit in text.encode_utf16() {
                out.extend_from_slice(&unit.to_le_bytes());
            }
            Ok(out)
        }
    }
}

fn escaped(text: &str, codepage: &'static encoding_rs::Encoding) -> Result<Vec<u8>> {
    let (bytes, _, unmappable) = codepage.encode(text);
    if unmappable {
        return Err(WriteError::InvalidElement(format!(
            "text {text:?} is not representable in {}",
            codepage.name()
        )));
    }
    let mut out = ESCAPED_CODEPAGE_MARKER.to_vec();
    out.extend_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproduces_fixture_escaped_text() {
        // Fixture text object 40 stores "myTéxt" as ff fe 01 00 6d 79 54 e9 78 74.
        let bytes = encode_text("myTéxt", TextEncoding::Auto).unwrap();
        assert_eq!(
            bytes,
            [0xff, 0xfe, 0x01, 0x00, 0x6d, 0x79, 0x54, 0xe9, 0x78, 0x74]
        );
    }

    #[test]
    fn auto_rejects_latvian_letters() {
        assert!(encode_text("Rīga", TextEncoding::Auto).is_err());
        let baltic = encode_text("Rīga", TextEncoding::EscapedCp1257).unwrap();
        assert_eq!(&baltic[4..], &[b'R', 0xee, b'g', b'a']);
        let utf16 = encode_text("ā", TextEncoding::Utf16).unwrap();
        assert_eq!(utf16, [0xff, 0xfd, 0x01, 0x01]);
    }
}
