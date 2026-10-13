//! New levels in the seed's level table.
//!
//! A level is a type-95 entry in the table whose type-96 header has table
//! kind 1 (FN-V01). A new entry is a copy of an existing level entry of the
//! same seed, with its own element ID, time, level ID, name and description
//! (FN-V03). It goes into the table in level-ID order, and the header's entry
//! count grows (FN-V03).

use std::io::Write;

use crate::element::{string_linkage_bytes, EncodeContext, IdAllocator, Profile};
use crate::error::{Result, WriteError};
use crate::page::{le_u32, Page, PageHeader, PageObject};
use crate::seed::{
    page_number, read_stream, Compound, LevelEntry, TABLE_ENTRY_LIBRARY_BIT, TABLE_ENTRY_TYPE,
    TABLE_KIND_LEVELS,
};
use crate::text::{ESCAPED_CODEPAGE_MARKER, NATIVE_UNICODE_MARKER};

/// Element type of a table header (FN-V01).
pub const TABLE_HEADER_TYPE: u8 = 96;
/// Offset of the entry count in a table header (FN-V01).
pub const TABLE_COUNT_AT: usize = 0x20;
/// Offset of the level ID in a level entry (FN-V01).
pub const LEVEL_ID_AT: usize = 0x20;
/// Level ID of "Default", the preferred template (FN-V03).
pub const DEFAULT_LEVEL_ID: u32 = 64;

/// A level to add.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NewLevel {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
}

impl NewLevel {
    #[must_use]
    pub fn entry(&self) -> LevelEntry {
        LevelEntry {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

/// String payload as MicroStation writes level names: `ff fe 01 00` + 8-bit
/// for ASCII (EXP-0003 E4), `ff fe` + UTF-16LE otherwise (FN-L03, H-V04).
fn name_payload(text: &str) -> Vec<u8> {
    if text.is_ascii() {
        [&ESCAPED_CODEPAGE_MARKER[..], text.as_bytes()].concat()
    } else {
        let mut out = NATIVE_UNICODE_MARKER.to_vec();
        for unit in text.encode_utf16() {
            out.extend_from_slice(&unit.to_le_bytes());
        }
        out
    }
}

/// One level entry: the template's primary data with new IDs and time, then
/// the name (property 1) and description (property 2) linkages (FN-V03).
pub fn level_entry(
    template: &[u8],
    level: &NewLevel,
    element_id: u64,
    modified_ms: f64,
) -> Result<Vec<u8>> {
    let primary_len = 2 * le_u32(template, 8) as usize; // FN-P03
    if primary_len < LEVEL_ID_AT + 4 || primary_len > template.len() {
        return Err(WriteError::InvalidSeed(
            "level entry template is too short".into(),
        ));
    }
    let mut out = template[..primary_len].to_vec();
    out[0x10..0x18].copy_from_slice(&element_id.to_le_bytes()); // FN-E02
    out[0x18..0x20].copy_from_slice(&modified_ms.to_le_bytes()); // FN-E03a
    out[LEVEL_ID_AT..LEVEL_ID_AT + 4].copy_from_slice(&level.id.to_le_bytes()); // FN-V01
    out.extend(string_linkage_bytes(1, &name_payload(&level.name)));
    if let Some(description) = &level.description {
        out.extend(string_linkage_bytes(2, &name_payload(description)));
    }
    let words = u32::try_from(out.len() / 2)
        .map_err(|_| WriteError::InvalidInput("level entry is too long".into()))?;
    out[4..8].copy_from_slice(&words.to_le_bytes()); // FN-P03
    Ok(out)
}

fn is_level_object(object: &PageObject, element_type: u8) -> bool {
    let bytes = &object.bytes;
    object.element_type() == element_type
        && bytes.len() >= 0x24
        && le_u32(bytes, 0x0c) == TABLE_KIND_LEVELS
        && le_u32(bytes, 0) & TABLE_ENTRY_LIBRARY_BIT == 0
}

/// Add `levels` to the file's level table. Returns the page written.
pub(crate) fn add_levels(
    compound: &mut Compound,
    levels: &[NewLevel],
    ids: &mut IdAllocator,
    context: &EncodeContext,
) -> Result<String> {
    let mut pages: Vec<(u32, String)> = compound
        .walk()
        .filter(|entry| entry.is_stream())
        .map(|entry| entry.path().to_string_lossy().replace('\\', "/"))
        .filter(|path| path.starts_with("/Dgn^Nm/"))
        .filter_map(|path| page_number(&path).map(|number| (number, path)))
        .collect();
    pages.sort();
    for (_, path) in pages {
        let mut page = Page::parse(&read_stream(compound, &path)?)?;
        let Some(header) = page
            .objects
            .iter()
            .position(|object| is_level_object(object, TABLE_HEADER_TYPE))
        else {
            continue;
        };
        let run_end = |page: &Page| {
            page.objects[header + 1..]
                .iter()
                .take_while(|object| is_level_object(object, TABLE_ENTRY_TYPE))
                .count()
                + header
                + 1
        };
        let entries = &page.objects[header + 1..run_end(&page)];
        let template = entries
            .iter()
            .find(|object| le_u32(&object.bytes, LEVEL_ID_AT) == DEFAULT_LEVEL_ID)
            .or_else(|| entries.first())
            .ok_or_else(|| {
                WriteError::UnsupportedSeed("the level table has no entry to copy".into())
            })?
            .bytes
            .clone();
        for level in levels {
            let bytes = level_entry(&template, level, ids.allocate(), context.modified_ms)?;
            // Level-ID order, as MicroStation stores its table (FN-V03).
            let end = run_end(&page);
            let at = (header + 1..end)
                .find(|&index| le_u32(&page.objects[index].bytes, LEVEL_ID_AT) > level.id)
                .unwrap_or(end);
            page.objects.insert(at, PageObject { prefix: 0, bytes });
        }
        let added = levels.len() as u32;
        let header_bytes = &mut page.objects[header].bytes;
        let count = le_u32(header_bytes, TABLE_COUNT_AT) + added;
        header_bytes[TABLE_COUNT_AT..TABLE_COUNT_AT + 4].copy_from_slice(&count.to_le_bytes());
        note_appended(&mut page.header, added, context.profile);
        let mut stream = compound.create_stream(&path)?;
        stream.write_all(&page.encode()?)?;
        return Ok(path);
    }
    Err(WriteError::UnsupportedSeed(
        "the seed has no level table".into(),
    ))
}

/// Page header bookkeeping after adding `added` objects (FN-P01, FN-P05).
pub(crate) fn note_appended(header: &mut PageHeader, added: u32, profile: Profile) {
    header.record_count += added;
    match profile {
        // ODA keeps population equal to the record count (FN-P01).
        Profile::Oda => header.population += added,
        // MicroStation stores page number 0 and population 0 (FN-P05).
        Profile::MicroStation => {
            header.page_number = 0;
            header.population = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_copies_template_and_replaces_ids_and_names() {
        // Shape of the ADTI seed's "Default" entry (EXP-0003 E1): 240 bytes
        // of primary data, then the name linkage.
        let mut template = vec![0u8; 240];
        template[..4].copy_from_slice(&0x4440_005f_u32.to_le_bytes());
        template[8..12].copy_from_slice(&120u32.to_le_bytes());
        template[0x0c] = 1;
        template[0x20] = 64;
        template[0x28..0x30].copy_from_slice(&[6, 0, 0x68, 2, 0xff, 0xff, 0xff, 0xff]);
        template.extend(string_linkage_bytes(1, b"Default"));
        let level = NewLevel {
            id: 1,
            name: "ZZ_A".into(),
            description: Some("Līmenis".into()),
        };
        let entry = level_entry(&template, &level, 1400, 5.0).unwrap();
        assert_eq!(&entry[..4], &template[..4]);
        assert_eq!(le_u32(&entry, 4) as usize * 2, entry.len());
        assert_eq!(le_u32(&entry, 8), 120);
        assert_eq!(&entry[0x10..0x18], &1400u64.to_le_bytes());
        assert_eq!(&entry[0x18..0x20], &5.0f64.to_le_bytes());
        assert_eq!(le_u32(&entry, LEVEL_ID_AT), 1);
        assert_eq!(&entry[0x24..240], &template[0x24..240]);
        // Name as in E4: 4-byte marker + ASCII, in a 24-byte linkage.
        assert_eq!(
            &entry[240..264],
            &[
                0x0b, 0x10, 0xd2, 0x56, 1, 0, 0, 0, 8, 0, 0, 0, 0xff, 0xfe, 1, 0, b'Z', b'Z', b'_',
                b'A', 0, 0, 0, 0
            ]
        );
        assert_eq!(
            crate::seed::string_property(&entry, 2).as_deref(),
            Some("Līmenis")
        );
    }
}
