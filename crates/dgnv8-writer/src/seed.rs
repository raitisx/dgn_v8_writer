//! Read-only analysis of a seed file: which model to write into, its units,
//! its extents, and the highest element ID already in use.

use std::io::{Cursor, Read};

use serde::Serialize;

use crate::error::{Result, WriteError};
use crate::page::{inflate, le_f64, le_i64, le_u32, le_u64, Page};

/// Streams whose presence identifies a DGN V8 container (FN-S01).
pub const REQUIRED_ROOT_ENTRIES: [&str; 3] = ["/Dgn~H", "/Dgn~S", "/Dgn-Md"];
/// Element type of the model header object in `Dgn~Mh` (FN-M01).
pub const MODEL_HEADER_TYPE: u8 = 66;
/// Model-header type-word bit meaning "2D model" (FN-M02).
pub const MODEL_2D_FLAG: u32 = 0x0080_0000;
/// Size of the uncompressed prefix of `Dgn~H` before its zlib data (FN-H01).
pub const FILE_HEADER_PREFIX_BYTES: usize = 0x14;
/// Offset in inflated `Dgn~H` of a u64 that looks like an element-ID counter
/// (hypothesis H-H02; read for reporting, written only on request).
pub const FILE_HEADER_ID_COUNTER_OFFSET: usize = 0x128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Dimension {
    #[serde(rename = "2D")]
    Two,
    #[serde(rename = "3D")]
    Three,
}

/// What the writer needs to know about the seed.
#[derive(Debug, Clone, Serialize)]
pub struct SeedInfo {
    pub model_storages: Vec<String>,
    pub model_storage: String,
    pub dimension: Dimension,
    pub uor_per_master: f64,
    pub global_origin_uor: [f64; 3],
    pub extents_low_uor: [i64; 3],
    pub extents_high_uor: [i64; 3],
    pub max_element_id: u64,
    pub file_header_id_counter: Option<u64>,
    pub graphic_pages: Vec<String>,
    pub page_format_versions: Vec<u32>,
    pub graphic_object_count: usize,
}

/// Location of the model header object inside inflated `Dgn~Mh`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelHeaderLocation {
    pub offset: usize,
}

impl ModelHeaderLocation {
    pub const EXTENTS: usize = 0x90; // FN-M05: low xyz, high xyz as i64
    pub const GLOBAL_ORIGIN: usize = 0xc8; // FN-M03
    pub const UOR_PER_MASTER: usize = 0xe0; // FN-M04
}

pub(crate) type Compound = cfb::CompoundFile<Cursor<Vec<u8>>>;

/// Analyse `bytes` as a seed and choose model `model` (default: the first).
pub fn analyse(bytes: &[u8], model: Option<usize>) -> Result<SeedInfo> {
    let mut compound = open_compound(bytes.to_vec())?;
    analyse_compound(&mut compound, model)
}

pub(crate) fn open_compound(bytes: Vec<u8>) -> Result<Compound> {
    cfb::CompoundFile::open(Cursor::new(bytes))
        .map_err(|error| WriteError::InvalidSeed(format!("not a compound file: {error}")))
}

pub(crate) fn analyse_compound(compound: &mut Compound, model: Option<usize>) -> Result<SeedInfo> {
    for required in REQUIRED_ROOT_ENTRIES {
        if !compound.exists(required) {
            return Err(WriteError::InvalidSeed(format!("missing {required}")));
        }
    }
    let entries: Vec<(String, bool)> = compound
        .walk()
        .map(|entry| (path_string(entry.path()), entry.is_stream()))
        .collect();

    let mut model_storages: Vec<String> = entries
        .iter()
        .filter(|(path, is_stream)| {
            !is_stream && path.starts_with("/Dgn-Md/#") && path.matches('/').count() == 2
        })
        .map(|(path, _)| path.clone())
        .collect();
    model_storages.sort();
    let index = model.unwrap_or(0);
    let model_storage = model_storages.get(index).cloned().ok_or_else(|| {
        WriteError::InvalidSeed(format!(
            "model {index} requested, seed has {} model storages",
            model_storages.len()
        ))
    })?;

    // Model header (FN-M01..FN-M05).
    let header_bytes = inflate(&read_stream(compound, &format!("{model_storage}/Dgn~Mh"))?)?;
    let location = find_model_header(&header_bytes)?;
    let object = &header_bytes[location.offset..];
    let dimension = if le_u32(object, 0) & MODEL_2D_FLAG != 0 {
        Dimension::Two
    } else {
        Dimension::Three
    };
    let extents_low_uor = read_i64x3(object, ModelHeaderLocation::EXTENTS);
    let extents_high_uor = read_i64x3(object, ModelHeaderLocation::EXTENTS + 24);
    let global_origin_uor = [
        le_f64(object, ModelHeaderLocation::GLOBAL_ORIGIN),
        le_f64(object, ModelHeaderLocation::GLOBAL_ORIGIN + 8),
        le_f64(object, ModelHeaderLocation::GLOBAL_ORIGIN + 16),
    ];
    let uor_per_master = le_f64(object, ModelHeaderLocation::UOR_PER_MASTER);
    if !(uor_per_master.is_finite() && uor_per_master > 0.0) {
        return Err(WriteError::InvalidSeed(format!(
            "model header UOR/master {uor_per_master} is not positive"
        )));
    }
    let mut max_element_id = le_u64(object, 0x10);

    // Highest element ID across every object page of every storage (FN-S02).
    let mut page_format_versions = Vec::new();
    for (path, is_stream) in &entries {
        if !is_stream || page_number(path).is_none() || is_auxiliary_page(path) {
            continue;
        }
        let page = Page::parse(&read_stream(compound, path)?)
            .map_err(|error| WriteError::InvalidSeed(format!("{path}: {error}")))?;
        if !page_format_versions.contains(&page.header.format_version) {
            page_format_versions.push(page.header.format_version);
        }
        for object in &page.objects {
            if let Some(id) = object.element_id() {
                max_element_id = max_element_id.max(id);
            }
        }
    }

    let graphic_pages = graphic_pages(&entries, &model_storage);
    let mut graphic_object_count = 0;
    for path in &graphic_pages {
        graphic_object_count += Page::parse(&read_stream(compound, path)?)?.objects.len();
    }

    Ok(SeedInfo {
        model_storages,
        model_storage,
        dimension,
        uor_per_master,
        global_origin_uor,
        extents_low_uor,
        extents_high_uor,
        max_element_id,
        file_header_id_counter: read_file_header_counter(compound).ok(),
        graphic_pages,
        page_format_versions,
        graphic_object_count,
    })
}

/// Graphic pages `<model>/Dgn^G/$N`, ordered by page number.
fn graphic_pages(entries: &[(String, bool)], model_storage: &str) -> Vec<String> {
    let prefix = format!("{model_storage}/Dgn^G/");
    let mut pages: Vec<(u32, String)> = entries
        .iter()
        .filter(|(path, is_stream)| *is_stream && path.starts_with(&prefix))
        .filter_map(|(path, _)| page_number(path).map(|number| (number, path.clone())))
        .collect();
    pages.sort();
    pages.into_iter().map(|(_, path)| path).collect()
}

/// `$N` stream number, if the last path component is a page stream.
pub(crate) fn page_number(path: &str) -> Option<u32> {
    path.rsplit('/').next()?.strip_prefix('$')?.parse().ok()
}

/// Auxiliary storages end in `A` (`Dgn^GA`, `Dgn^NmA`, ...) and use a
/// different record framing (FN-P04); they hold no new element IDs.
fn is_auxiliary_page(path: &str) -> bool {
    let mut parts = path.rsplit('/');
    parts.next();
    parts.next().is_some_and(|storage| storage.ends_with('A'))
}

/// The model header is one complete type-66 object ending at the end of the
/// inflated stream (FN-M01).
pub(crate) fn find_model_header(bytes: &[u8]) -> Result<ModelHeaderLocation> {
    if bytes.len() < ModelHeaderLocation::UOR_PER_MASTER + 8 {
        return Err(WriteError::InvalidSeed("Dgn~Mh is too short".into()));
    }
    let mut offset = (bytes.len() - 12) & !1;
    loop {
        if bytes[offset] == MODEL_HEADER_TYPE {
            let words = le_u32(bytes, offset + 4) as usize;
            if offset + words * 2 == bytes.len()
                && words * 2 >= ModelHeaderLocation::UOR_PER_MASTER + 8
            {
                return Ok(ModelHeaderLocation { offset });
            }
        }
        if offset == 0 {
            return Err(WriteError::InvalidSeed(
                "no type-66 model header object in Dgn~Mh".into(),
            ));
        }
        offset -= 2;
    }
}

pub(crate) fn read_file_header_counter(compound: &mut Compound) -> Result<u64> {
    let raw = read_stream(compound, "/Dgn~H")?;
    if raw.len() <= FILE_HEADER_PREFIX_BYTES {
        return Err(WriteError::InvalidSeed("Dgn~H is too short".into()));
    }
    let inflated = inflate(&raw[FILE_HEADER_PREFIX_BYTES..])?;
    if inflated.len() < FILE_HEADER_ID_COUNTER_OFFSET + 8 {
        return Err(WriteError::InvalidSeed(
            "inflated Dgn~H is too short".into(),
        ));
    }
    Ok(le_u64(&inflated, FILE_HEADER_ID_COUNTER_OFFSET))
}

pub(crate) fn read_stream(compound: &mut Compound, path: &str) -> Result<Vec<u8>> {
    let mut stream = compound
        .open_stream(path)
        .map_err(|error| WriteError::InvalidSeed(format!("{path}: {error}")))?;
    let mut out = Vec::new();
    stream.read_to_end(&mut out)?;
    Ok(out)
}

fn read_i64x3(bytes: &[u8], offset: usize) -> [i64; 3] {
    [
        le_i64(bytes, offset),
        le_i64(bytes, offset + 8),
        le_i64(bytes, offset + 16),
    ]
}

fn path_string(path: &std::path::Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.starts_with('/') {
        text
    } else {
        format!("/{text}")
    }
}
