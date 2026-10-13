//! Text payloads of type-17 text elements and of string linkages.
//!
//! Observed forms (FN-T02, FN-T05, FN-T07, FN-L03):
//! - plain 8-bit bytes (ASCII);
//! - `ff fe 01 00` + 8-bit bytes, read as Windows-1252;
//! - `ff fe` + UTF-16LE, what MicroStation writes for Latvian text;
//! - `ff fd` + UTF-16LE, what ODA writes in string linkages; MicroStation
//!   displays it too.
//!
//! The escaped Windows-1257 form is a rejected experiment (H-T03).

use crate::error::{Result, WriteError};

/// Marker before 8-bit text that MicroStation reads as Windows-1252.
pub const ESCAPED_CODEPAGE_MARKER: [u8; 4] = [0xff, 0xfe, 0x01, 0x00];
/// Marker before UTF-16LE text as MicroStation writes it (FN-T07).
pub const NATIVE_UNICODE_MARKER: [u8; 2] = [0xff, 0xfe];
/// Marker before UTF-16LE strings in ODA-written string linkages (FN-L02).
pub const UTF16_MARKER: [u8; 2] = [0xff, 0xfd];

/// How to store a text string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextEncoding {
    /// Plain ASCII when possible, otherwise MicroStation's `ff fe` +
    /// UTF-16LE (FN-T07).
    #[default]
    Auto,
    /// Plain 7-bit ASCII (FN-T02).
    Ascii,
    /// `ff fe 01 00` + Windows-1252 bytes, as in the GDAL sample (FN-T02).
    EscapedCp1252,
    /// REJECTED experiment H-T03, kept only to rebuild test file T03:
    /// MicroStation shows these bytes as Windows-1252, not Baltic.
    EscapedCp1257,
    /// `ff fd` + UTF-16LE (FN-T05), as tested in T04.
    Utf16,
    /// `ff fe` + UTF-16LE, MicroStation's own form (FN-T07).
    Unicode,
}

/// Encode `text` into the exact bytes stored after the text header.
pub fn encode_text(text: &str, encoding: TextEncoding) -> Result<Vec<u8>> {
    match encoding {
        TextEncoding::Auto => {
            if text.is_ascii() {
                Ok(text.as_bytes().to_vec())
            } else {
                encode_text(text, TextEncoding::Unicode)
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
        TextEncoding::Utf16 => Ok(utf16(&UTF16_MARKER, text)),
        TextEncoding::Unicode => Ok(utf16(&NATIVE_UNICODE_MARKER, text)),
    }
}

/// Decode any of the observed payload forms (FN-L03). Trailing NULs, used
/// as padding in linkages, are dropped.
#[must_use]
pub fn decode_payload(bytes: &[u8]) -> String {
    let text = if let Some(rest) = bytes.strip_prefix(&ESCAPED_CODEPAGE_MARKER) {
        encoding_rs::WINDOWS_1252
            .decode_without_bom_handling(rest)
            .0
            .into_owned()
    } else if let Some(rest) = bytes
        .strip_prefix(&NATIVE_UNICODE_MARKER)
        .or_else(|| bytes.strip_prefix(&UTF16_MARKER))
    {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        encoding_rs::WINDOWS_1252
            .decode_without_bom_handling(bytes)
            .0
            .into_owned()
    };
    text.trim_end_matches('\0').to_owned()
}

fn utf16(marker: &[u8], text: &str) -> Vec<u8> {
    let mut out = marker.to_vec();
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
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
        let bytes = encode_text("myTéxt", TextEncoding::EscapedCp1252).unwrap();
        assert_eq!(
            bytes,
            [0xff, 0xfe, 0x01, 0x00, 0x6d, 0x79, 0x54, 0xe9, 0x78, 0x74]
        );
    }

    #[test]
    fn auto_uses_microstation_unicode_for_latvian_letters() {
        // MicroStation stores "Gl…" of a Latvian text as ff fe 47 00 6c 00 (EXP-0003 E3).
        let bytes = encode_text("Glāž", TextEncoding::Auto).unwrap();
        assert_eq!(
            bytes,
            [0xff, 0xfe, 0x47, 0x00, 0x6c, 0x00, 0x01, 0x01, 0x7e, 0x01]
        );
        assert_eq!(
            encode_text("ā", TextEncoding::Utf16).unwrap(),
            [0xff, 0xfd, 0x01, 0x01]
        );
        assert_eq!(encode_text("ADTI", TextEncoding::Auto).unwrap(), b"ADTI");
        let baltic = encode_text("Rīga", TextEncoding::EscapedCp1257).unwrap();
        assert_eq!(&baltic[4..], &[b'R', 0xee, b'g', b'a']);
    }

    #[test]
    fn decodes_every_observed_form() {
        assert_eq!(decode_payload(b"Default\0"), "Default");
        assert_eq!(
            decode_payload(&[0xff, 0xfe, 0x01, 0x00, b't', b'w', b'o']),
            "two"
        );
        assert_eq!(decode_payload(&[0xff, 0xfe, 0x01, 0x01, 0x7e, 0x01]), "āž");
        assert_eq!(decode_payload(&[0xff, 0xfd, b'm', 0, 0, 0]), "m");
    }
}
