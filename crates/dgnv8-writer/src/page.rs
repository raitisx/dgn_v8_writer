//! Object pages inside `$N` streams.
//!
//! A page stream is a 16-byte uncompressed header followed by a zlib payload
//! (FN-P01). The inflated payload is a sequence of framed objects, each
//! preceded by a 4-byte prefix (FN-P02). An empty page has no payload at all.

use std::io::{Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::error::{Result, WriteError};

/// Size of the uncompressed page header (FN-P01).
pub const PAGE_HEADER_BYTES: usize = 16;
/// Size of the prefix in front of every object in an inflated page (FN-P02).
pub const OBJECT_PREFIX_BYTES: usize = 4;
/// Smallest object: type word, total words and attribute words (FN-P03).
pub const MIN_OBJECT_BYTES: usize = 12;

/// The four little-endian `u32` fields of a page header (FN-P01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageHeader {
    pub record_count: u32,
    pub format_version: u32,
    pub page_number: u32,
    pub population: u32,
}

impl PageHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < PAGE_HEADER_BYTES {
            return Err(WriteError::InvalidSeed(format!(
                "page stream has {} bytes, header needs {PAGE_HEADER_BYTES}",
                bytes.len()
            )));
        }
        Ok(Self {
            record_count: le_u32(bytes, 0),
            format_version: le_u32(bytes, 4),
            page_number: le_u32(bytes, 8),
            population: le_u32(bytes, 12),
        })
    }

    pub fn to_bytes(self) -> [u8; PAGE_HEADER_BYTES] {
        let mut out = [0u8; PAGE_HEADER_BYTES];
        out[0..4].copy_from_slice(&self.record_count.to_le_bytes());
        out[4..8].copy_from_slice(&self.format_version.to_le_bytes());
        out[8..12].copy_from_slice(&self.page_number.to_le_bytes());
        out[12..16].copy_from_slice(&self.population.to_le_bytes());
        out
    }
}

/// One framed object: its 4-byte prefix and its exact bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageObject {
    pub prefix: u32,
    pub bytes: Vec<u8>,
}

impl PageObject {
    /// Element type in the low byte of the first word (FN-E01).
    pub fn element_type(&self) -> u8 {
        self.bytes[0]
    }

    /// Element ID at 0x10 when the object is long enough (FN-E02).
    pub fn element_id(&self) -> Option<u64> {
        (self.bytes.len() >= 0x18).then(|| le_u64(&self.bytes, 0x10))
    }
}

/// A decoded object page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub header: PageHeader,
    pub objects: Vec<PageObject>,
}

impl Page {
    /// Parse a complete `$N` object-page stream.
    pub fn parse(stream: &[u8]) -> Result<Self> {
        let header = PageHeader::parse(stream)?;
        if stream.len() == PAGE_HEADER_BYTES {
            return Ok(Self {
                header,
                objects: Vec::new(),
            });
        }
        let body = inflate(&stream[PAGE_HEADER_BYTES..])?;
        let objects = split_objects(&body)?;
        Ok(Self { header, objects })
    }

    /// Serialize back to a `$N` stream. An empty page is written without a
    /// payload, as observed in the fixture's empty auxiliary page (FN-P01).
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = self.header.to_bytes().to_vec();
        if self.objects.is_empty() {
            return Ok(out);
        }
        let mut body = Vec::with_capacity(self.inflated_len());
        for object in &self.objects {
            body.extend_from_slice(&object.prefix.to_le_bytes());
            body.extend_from_slice(&object.bytes);
        }
        out.extend_from_slice(&deflate(&body)?);
        Ok(out)
    }

    pub fn inflated_len(&self) -> usize {
        self.objects
            .iter()
            .map(|object| OBJECT_PREFIX_BYTES + object.bytes.len())
            .sum()
    }
}

/// Split an inflated object page into framed objects (FN-P02, FN-P03).
pub fn split_objects(body: &[u8]) -> Result<Vec<PageObject>> {
    let mut objects = Vec::new();
    let mut offset = 0usize;
    while offset < body.len() {
        let object_start = offset + OBJECT_PREFIX_BYTES;
        if object_start + MIN_OBJECT_BYTES > body.len() {
            return Err(WriteError::InvalidSeed(format!(
                "truncated object frame at inflated offset {offset}"
            )));
        }
        let prefix = le_u32(body, offset);
        let words = le_u32(body, object_start + 4) as usize;
        let length = words * 2;
        if length < MIN_OBJECT_BYTES || object_start + length > body.len() {
            return Err(WriteError::InvalidSeed(format!(
                "object at inflated offset {offset} declares {words} words"
            )));
        }
        objects.push(PageObject {
            prefix,
            bytes: body[object_start..object_start + length].to_vec(),
        });
        offset = object_start + length;
    }
    Ok(objects)
}

/// Inflate one complete zlib stream; the whole input must be consumed.
pub fn inflate(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|error| WriteError::InvalidSeed(format!("zlib payload: {error}")))?;
    if decoder.total_in() != data.len() as u64 {
        return Err(WriteError::InvalidSeed(format!(
            "zlib payload leaves {} trailing bytes",
            data.len() as u64 - decoder.total_in()
        )));
    }
    Ok(out)
}

/// Compress with zlib at the default level (FN-P01 streams start `78 9c`).
pub fn deflate(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub(crate) fn le_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

pub(crate) fn le_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

pub(crate) fn le_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

pub(crate) fn le_f64(bytes: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_round_trips_through_zlib() {
        let object = {
            let mut bytes = vec![0u8; 16];
            bytes[0] = 3;
            bytes[4..8].copy_from_slice(&8u32.to_le_bytes());
            bytes
        };
        let page = Page {
            header: PageHeader {
                record_count: 1,
                format_version: 2,
                page_number: 1,
                population: 1,
            },
            objects: vec![PageObject {
                prefix: 0,
                bytes: object,
            }],
        };
        let encoded = page.encode().unwrap();
        assert_eq!(&encoded[16..18], &[0x78, 0x9c]);
        assert_eq!(Page::parse(&encoded).unwrap(), page);
    }

    #[test]
    fn empty_page_has_no_payload() {
        let page = Page {
            header: PageHeader {
                record_count: 0,
                format_version: 2,
                page_number: 0,
                population: 0,
            },
            objects: Vec::new(),
        };
        let encoded = page.encode().unwrap();
        assert_eq!(encoded.len(), PAGE_HEADER_BYTES);
        assert_eq!(Page::parse(&encoded).unwrap(), page);
    }

    #[test]
    fn rejects_trailing_zlib_bytes() {
        let mut data = deflate(b"abc").unwrap();
        data.push(0);
        assert!(inflate(&data).is_err());
    }
}
