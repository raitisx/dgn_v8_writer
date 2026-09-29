//! Seed + append writer.
//!
//! The seed is copied unchanged except for three things: the model's last
//! graphic page receives the new objects, the model header extents grow to
//! cover them, and (only on request) the `Dgn~H` ID counter is raised.

use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::element::{Element, EncodeContext, IdAllocator};
use crate::error::{Result, WriteError};
use crate::geom::{Point2, RangeI64};
use crate::page::{deflate, inflate, Page, PageHeader, PageObject};
use crate::seed::{
    analyse_compound, find_model_header, open_compound, read_stream, Compound, Dimension,
    ModelHeaderLocation, SeedInfo, FILE_HEADER_ID_COUNTER_OFFSET, FILE_HEADER_PREFIX_BYTES,
};

/// Switches for one save.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Last-modified stamp for new objects, in ms since 1970 (default: now).
    pub modified_ms: Option<f64>,
    /// EXPERIMENT H-H02: also write the new highest ID into `Dgn~H`.
    pub update_file_header_counter: bool,
    /// Allow writing 2D elements into a 3D model (test fixtures only).
    pub allow_3d_model: bool,
}

/// A seed plus the elements to append to one of its models.
#[derive(Debug, Clone)]
pub struct Document {
    seed: Vec<u8>,
    info: SeedInfo,
    elements: Vec<Element>,
}

/// Result of a save, for reporting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SaveReport {
    pub page: String,
    pub objects_written: usize,
    pub first_element_id: u64,
    pub last_element_id: u64,
    pub extents_low_uor: [i64; 3],
    pub extents_high_uor: [i64; 3],
    pub file_header_counter_written: Option<u64>,
}

impl Document {
    /// Open a seed held in memory; `model` selects a model storage by index.
    pub fn from_seed_bytes(seed: Vec<u8>, model: Option<usize>) -> Result<Self> {
        let mut compound = open_compound(seed.clone())?;
        let info = analyse_compound(&mut compound, model)?;
        Ok(Self {
            seed,
            info,
            elements: Vec::new(),
        })
    }

    pub fn from_seed_path(path: impl AsRef<Path>, model: Option<usize>) -> Result<Self> {
        Self::from_seed_bytes(std::fs::read(path)?, model)
    }

    #[must_use]
    pub fn info(&self) -> &SeedInfo {
        &self.info
    }

    /// Convert master units to UOR: `uor = master * scale + origin` (FN-M03, FN-M04).
    #[must_use]
    pub fn master_to_uor(&self, x: f64, y: f64) -> Point2 {
        Point2::new(
            x * self.info.uor_per_master + self.info.global_origin_uor[0],
            y * self.info.uor_per_master + self.info.global_origin_uor[1],
        )
    }

    /// Convert a distance in master units to UOR.
    #[must_use]
    pub fn distance_to_uor(&self, distance: f64) -> f64 {
        distance * self.info.uor_per_master
    }

    pub fn add(&mut self, element: Element) {
        self.elements.push(element);
    }

    #[must_use]
    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// Build the output file in memory.
    pub fn to_bytes(&self, options: &WriteOptions) -> Result<(Vec<u8>, SaveReport)> {
        if self.info.dimension == Dimension::Three && !options.allow_3d_model {
            return Err(WriteError::UnsupportedSeed(
                "the selected model is 3D; this writer produces 2D elements only".into(),
            ));
        }
        let context = EncodeContext {
            modified_ms: options.modified_ms.unwrap_or_else(now_ms),
        };
        let mut ids = IdAllocator::starting_at(self.info.max_element_id + 1);
        let first_element_id = self.info.max_element_id + 1;
        let mut new_objects = Vec::new();
        let mut range: Option<RangeI64> = None;
        for element in &self.elements {
            let encoded = element.encode(&mut ids, &context)?;
            range = Some(range.map_or(encoded.range, |r| r.union(encoded.range)));
            new_objects.extend(encoded.objects);
        }

        let mut compound = open_compound(self.seed.clone())?;
        let page_path = self.append_objects(&mut compound, &new_objects)?;

        // Grow the model extents (FN-M05).
        let seed_range = RangeI64 {
            low: self.info.extents_low_uor,
            high: self.info.extents_high_uor,
        };
        let extents = match range {
            Some(new_range) if self.info.graphic_object_count == 0 => new_range,
            Some(new_range) => seed_range.union(new_range),
            None => seed_range,
        };
        self.write_model_extents(&mut compound, extents)?;

        let last_element_id = ids.last_allocated();
        let mut file_header_counter_written = None;
        if options.update_file_header_counter && !new_objects.is_empty() {
            write_file_header_counter(&mut compound, last_element_id)?;
            file_header_counter_written = Some(last_element_id);
        }

        compound.flush()?;
        let bytes = compound.into_inner().into_inner();
        Ok((
            bytes,
            SaveReport {
                page: page_path,
                objects_written: new_objects.len(),
                first_element_id,
                last_element_id,
                extents_low_uor: extents.low,
                extents_high_uor: extents.high,
                file_header_counter_written,
            },
        ))
    }

    pub fn save(&self, path: impl AsRef<Path>, options: &WriteOptions) -> Result<SaveReport> {
        let (bytes, report) = self.to_bytes(options)?;
        std::fs::write(path, bytes)?;
        Ok(report)
    }

    /// Append to the last graphic page, or create page `$1` when the model
    /// has none (FN-P01).
    fn append_objects(&self, compound: &mut Compound, objects: &[Vec<u8>]) -> Result<String> {
        let (path, mut page) = match self.info.graphic_pages.last() {
            Some(path) => (path.clone(), Page::parse(&read_stream(compound, path)?)?),
            None => {
                let storage = format!("{}/Dgn^G", self.info.model_storage);
                if !compound.is_storage(&storage) {
                    compound.create_storage(&storage)?;
                }
                let version = self.info.page_format_versions.first().copied().unwrap_or(2);
                (
                    format!("{storage}/$1"),
                    Page {
                        header: PageHeader {
                            record_count: 0,
                            format_version: version,
                            page_number: 1,
                            population: 0,
                        },
                        objects: Vec::new(),
                    },
                )
            }
        };
        let added = objects.len() as u32;
        page.header.record_count += added;
        page.header.population += added;
        page.objects.extend(objects.iter().map(|bytes| PageObject {
            prefix: 0, // FN-P02: always zero in the evidence
            bytes: bytes.clone(),
        }));
        let encoded = page.encode()?;
        let mut stream = compound.create_stream(&path)?;
        stream.write_all(&encoded)?;
        Ok(path)
    }

    fn write_model_extents(&self, compound: &mut Compound, extents: RangeI64) -> Result<()> {
        let path = format!("{}/Dgn~Mh", self.info.model_storage);
        let mut inflated = inflate(&read_stream(compound, &path)?)?;
        let location = find_model_header(&inflated)?;
        let base = location.offset + ModelHeaderLocation::EXTENTS;
        for (index, value) in extents.low.iter().chain(extents.high.iter()).enumerate() {
            let at = base + index * 8;
            inflated[at..at + 8].copy_from_slice(&value.to_le_bytes());
        }
        let mut stream = compound.create_stream(&path)?;
        stream.write_all(&deflate(&inflated)?)?; // FN-M01: zlib from offset 0
        Ok(())
    }
}

fn write_file_header_counter(compound: &mut Compound, value: u64) -> Result<()> {
    let raw = read_stream(compound, "/Dgn~H")?;
    let prefix = &raw[..FILE_HEADER_PREFIX_BYTES];
    let mut inflated = inflate(&raw[FILE_HEADER_PREFIX_BYTES..])?;
    let at = FILE_HEADER_ID_COUNTER_OFFSET;
    inflated[at..at + 8].copy_from_slice(&value.to_le_bytes());
    let mut out = prefix.to_vec();
    out.extend_from_slice(&deflate(&inflated)?);
    let mut stream = compound.create_stream("/Dgn~H")?;
    stream.write_all(&out)?;
    Ok(())
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}
